//! Bitmaps > Remove Background: makes the background of a picture
//! transparent on this computer, without an AI service.
//!
//! 1. The background colours are learnt from the picture's border (a
//!    small k-means in Lab, so a gradient or a two-tone backdrop gives
//!    several colours) or taken from the colours the user picked.
//! 2. Every pixel gets its colour distance (CIE76 Delta E) to the
//!    nearest background colour.
//! 3. With Contiguous on, the background is what a flood fill from the
//!    border reaches through pixels within the tolerance; it may cross a
//!    few edge pixels (the anti-aliased rim) but never the subject, so
//!    background-coloured parts inside the subject stay. Off, every pixel
//!    near a background colour goes.
//! 4. Opacity ramps over a band past the tolerance (soft, anti-aliased
//!    edges), small loose specks are removed, and Edge softness feathers
//!    the edge.
//! 5. Clean edge colours takes the background's colour out of the
//!    half-transparent edge pixels, so no halo of the old backdrop shows
//!    on a new one.
//!
//! See docs/behavior/remove-background.md.

use image::RgbaImage;
use std::collections::VecDeque;

/// The dialog's settings.
#[derive(Debug, Clone, PartialEq)]
pub struct BgSettings {
    /// How far (Delta E) a colour may be from the background's and still
    /// be background.
    pub tolerance: f32,
    /// The feather of the edge, in pixels of the picture.
    pub softness: f32,
    /// Only the background connected to the picture's border goes.
    pub contiguous: bool,
    /// Loose specks left in the background go too.
    pub despeckle: bool,
    /// The backdrop's colour is taken out of half-transparent edge pixels.
    pub clean_edges: bool,
    /// The background colours the user picked; empty: learnt from the
    /// border.
    pub colors: Vec<[u8; 3]>,
}

impl Default for BgSettings {
    fn default() -> Self {
        BgSettings {
            tolerance: 14.0,
            softness: 1.0,
            contiguous: true,
            despeckle: true,
            clean_edges: true,
            colors: Vec::new(),
        }
    }
}

type Lab = [f32; 3];

fn lab(c: [u8; 3]) -> Lab {
    let (l, a, b) = tracedraw_core::color::rgb_to_lab(
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
    );
    [l, a, b]
}

fn dist2(a: &Lab, b: &Lab) -> f32 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

/// A background colour: in Lab for distances, in RGB for cleaning edges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BgColor {
    pub lab: Lab,
    pub rgb: [f32; 3],
}

impl BgColor {
    fn of(c: [u8; 3]) -> Self {
        BgColor {
            lab: lab(c),
            rgb: [c[0] as f32, c[1] as f32, c[2] as f32],
        }
    }
}

/// The pixels of the picture's border, two pixels deep (opaque ones).
fn border(img: &RgbaImage) -> Vec<[u8; 3]> {
    let (w, h) = img.dimensions();
    let depth = 2.min(w).min(h);
    let mut out = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let edge = x < depth || y < depth || x + depth >= w || y + depth >= h;
            if !edge {
                continue;
            }
            let p = img.get_pixel(x, y).0;
            if p[3] >= 128 {
                out.push([p[0], p[1], p[2]]);
            }
        }
    }
    out
}

/// The background colours learnt from the border: up to four k-means
/// clusters in Lab, each kept when it covers at least a tenth of the
/// border. None when the border is mostly transparent already.
pub fn learn_background(img: &RgbaImage) -> Vec<BgColor> {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return Vec::new();
    }
    let px = border(img);
    let ring = (2 * (w as usize + h as usize)).max(1);
    if px.len() * 3 < ring {
        return Vec::new();
    }
    // A sample of at most 4096 pixels, evenly spread.
    let step = (px.len() / 4096).max(1);
    let pts: Vec<([u8; 3], Lab)> = px.iter().step_by(step).map(|c| (*c, lab(*c))).collect();
    // Farthest-point seeds: the first point, then the one farthest from
    // every centre so far, while it is far enough to be another colour.
    let mut centres: Vec<Lab> = vec![pts[0].1];
    while centres.len() < 6 {
        let (far, d) = pts
            .iter()
            .map(|(_, l)| {
                let d = centres.iter().map(|c| dist2(l, c)).fold(f32::MAX, f32::min);
                (*l, d)
            })
            .fold(([0.0; 3], -1.0), |a, b| if b.1 > a.1 { b } else { a });
        if d < 10.0f32.powi(2) {
            break;
        }
        centres.push(far);
    }
    let mut members = vec![0usize; pts.len()];
    for _ in 0..10 {
        let mut sum = vec![[0.0f64; 6]; centres.len()];
        let mut n = vec![0usize; centres.len()];
        for (i, (rgb, l)) in pts.iter().enumerate() {
            let k = nearest(&centres, l).0;
            members[i] = k;
            n[k] += 1;
            for j in 0..3 {
                sum[k][j] += l[j] as f64;
                sum[k][3 + j] += rgb[j] as f64;
            }
        }
        for (k, c) in centres.iter_mut().enumerate() {
            if n[k] > 0 {
                for j in 0..3 {
                    c[j] = (sum[k][j] / n[k] as f64) as f32;
                }
            }
        }
    }
    let mut out = Vec::new();
    for (k, centre) in centres.iter().enumerate() {
        let mine: Vec<&[u8; 3]> = pts
            .iter()
            .zip(&members)
            .filter(|(_, m)| **m == k)
            .map(|((rgb, _), _)| rgb)
            .collect();
        if mine.len() * 15 < pts.len() {
            continue;
        }
        let mut rgb = [0.0f32; 3];
        for c in &mine {
            for j in 0..3 {
                rgb[j] += c[j] as f32;
            }
        }
        for v in &mut rgb {
            *v /= mine.len() as f32;
        }
        out.push(BgColor { lab: *centre, rgb });
    }
    out
}

fn nearest(centres: &[Lab], l: &Lab) -> (usize, f32) {
    centres
        .iter()
        .enumerate()
        .map(|(i, c)| (i, dist2(l, c)))
        .fold((0, f32::MAX), |a, b| if b.1 < a.1 { b } else { a })
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The picture with its background transparent. `scale` is the size of
/// `img` against the bitmap (previews are smaller), so the softness keeps
/// its look.
pub fn remove_background(img: &RgbaImage, s: &BgSettings, scale: f32) -> RgbaImage {
    let (w, h) = img.dimensions();
    let bg: Vec<BgColor> = if s.colors.is_empty() {
        learn_background(img)
    } else {
        s.colors.iter().map(|c| BgColor::of(*c)).collect()
    };
    if bg.is_empty() || w == 0 || h == 0 {
        return img.clone();
    }
    let (wu, hu) = (w as usize, h as usize);
    let n = wu * hu;
    let centres: Vec<Lab> = bg.iter().map(|b| b.lab).collect();
    // Each pixel in Lab, its distance to the nearest background colour,
    // and which one.
    let mut labs = vec![[0.0f32; 3]; n];
    let mut dist = vec![0.0f32; n];
    let mut which = vec![0u8; n];
    for (i, p) in img.pixels().enumerate() {
        if p.0[3] == 0 {
            continue;
        }
        let l = lab([p.0[0], p.0[1], p.0[2]]);
        let (k, d2) = nearest(&centres, &l);
        labs[i] = l;
        dist[i] = d2.sqrt();
        which[i] = k as u8;
    }
    let tol = s.tolerance.max(0.0);
    // The ramp past the tolerance where edge pixels are partly opaque.
    let band = (tol * 0.6).max(4.0);
    let ramp = |d: f32| smoothstep(tol, tol + band, d);
    // Opacity of the subject, 1 where it stays.
    let mut keep = vec![1.0f32; n];
    if s.contiguous {
        // Flood fill from the border: through background pixels freely,
        // and at most `rim` pixels into the ramp, so the fill takes the
        // anti-aliased rim and stops at the subject. It also walks on
        // from a background pixel to a neighbour of nearly the same
        // colour (a step under `step`), while that neighbour is within
        // `reach` of a background colour: so a backdrop that drifts
        // (texture, a gradient the border did not show, a soft shadow)
        // goes whole, while the subject's edge, a sharp step, stops it.
        let rim = 2 + s.softness.max(0.0).ceil() as u8;
        let step2 = (tol / 3.0).max(3.0).powi(2);
        let reach = (tol * 3.0).max(tol + band);
        let mut depth = vec![u8::MAX; n];
        // Reached by such steps: background, whatever its distance.
        let mut drifted = vec![false; n];
        let mut queue = VecDeque::new();
        let seed = |i: usize, depth: &mut Vec<u8>, queue: &mut VecDeque<usize>| {
            if depth[i] == u8::MAX && dist[i] < tol + band {
                depth[i] = if dist[i] <= tol { 0 } else { 1 };
                if depth[i] <= rim {
                    queue.push_back(i);
                }
            }
        };
        for x in 0..wu {
            seed(x, &mut depth, &mut queue);
            seed((hu - 1) * wu + x, &mut depth, &mut queue);
        }
        for y in 0..hu {
            seed(y * wu, &mut depth, &mut queue);
            seed(y * wu + wu - 1, &mut depth, &mut queue);
        }
        while let Some(i) = queue.pop_front() {
            let (x, y) = (i % wu, i / wu);
            let here = depth[i];
            let mut visit = |j: usize| {
                if depth[j] != u8::MAX {
                    return;
                }
                let near = dist[j] < tol + band;
                // A step from solid background only, never from the rim.
                let drift = here == 0
                    && dist[j] < reach
                    && img.get_pixel((j % wu) as u32, (j / wu) as u32).0[3] > 0
                    && dist2(&labs[i], &labs[j]) <= step2;
                if !near && !drift {
                    return;
                }
                let d = if dist[j] <= tol || drift { 0 } else { here + 1 };
                if d > rim {
                    return;
                }
                depth[j] = d;
                drifted[j] = drift && dist[j] > tol;
                queue.push_back(j);
            };
            if x > 0 {
                visit(i - 1);
            }
            if x + 1 < wu {
                visit(i + 1);
            }
            if y > 0 {
                visit(i - wu);
            }
            if y + 1 < hu {
                visit(i + wu);
            }
        }
        for i in 0..n {
            if drifted[i] {
                keep[i] = 0.0;
            } else if depth[i] != u8::MAX {
                keep[i] = ramp(dist[i]);
            }
        }
    } else {
        for i in 0..n {
            keep[i] = ramp(dist[i]);
        }
    }
    if s.despeckle {
        despeckle(&mut keep, wu, hu);
    }
    let radius = s.softness.max(0.0) * scale;
    if radius >= 0.5 {
        let r = radius.round() as usize;
        // Two box passes approach a Gaussian; only the edge changes,
        // since flat areas stay flat.
        box_blur(&mut keep, wu, hu, r);
        box_blur(&mut keep, wu, hu, r);
    }
    let mut out = img.clone();
    for (i, p) in out.pixels_mut().enumerate() {
        let a = keep[i];
        let orig = p.0[3] as f32;
        if a >= 0.999 {
            continue;
        }
        if s.clean_edges && a > 0.05 {
            // The pixel was a * subject + (1 - a) * backdrop: solve for
            // the subject's colour.
            let b = bg[which[i] as usize].rgb;
            for (ch, back) in p.0.iter_mut().zip(b) {
                let c = *ch as f32;
                *ch = ((c - (1.0 - a) * back) / a).round().clamp(0.0, 255.0) as u8;
            }
        }
        p.0[3] = (orig * a).round().clamp(0.0, 255.0) as u8;
    }
    out
}

/// Loose specks: kept parts not touching anything larger, smaller than
/// a two-thousandth of the picture (at least 12 pixels), go.
fn despeckle(keep: &mut [f32], w: usize, h: usize) {
    let n = w * h;
    let limit = (n / 2000).max(12);
    let mut seen = vec![false; n];
    let mut part = Vec::new();
    let mut stack = Vec::new();
    for start in 0..n {
        if seen[start] || keep[start] < 0.5 {
            continue;
        }
        part.clear();
        stack.push(start);
        seen[start] = true;
        while let Some(i) = stack.pop() {
            part.push(i);
            let (x, y) = (i % w, i / w);
            let mut go = |j: usize| {
                if !seen[j] && keep[j] >= 0.5 {
                    seen[j] = true;
                    stack.push(j);
                }
            };
            if x > 0 {
                go(i - 1);
            }
            if x + 1 < w {
                go(i + 1);
            }
            if y > 0 {
                go(i - w);
            }
            if y + 1 < h {
                go(i + w);
            }
        }
        if part.len() < limit && part.len() * 4 < n {
            for &i in &part {
                keep[i] = 0.0;
            }
        }
    }
}

/// A box blur of radius `r` across rows and then columns.
fn box_blur(v: &mut [f32], w: usize, h: usize, r: usize) {
    if r == 0 {
        return;
    }
    let mut tmp = vec![0.0f32; v.len()];
    let pass = |src: &[f32],
                dst: &mut [f32],
                len: usize,
                count: usize,
                at: &dyn Fn(usize, usize) -> usize| {
        for line in 0..count {
            let mut sum = 0.0f32;
            let mut k = 0usize;
            // Running sum over the window, edges clamped.
            let get = |i: isize| src[at(line, i.clamp(0, len as isize - 1) as usize)];
            for i in -(r as isize)..=(r as isize) {
                sum += get(i);
                k += 1;
            }
            for i in 0..len {
                dst[at(line, i)] = sum / k as f32;
                sum += get(i as isize + r as isize + 1) - get(i as isize - r as isize);
            }
        }
    };
    pass(v, &mut tmp, w, h, &|line, i| line * w + i);
    pass(&tmp, v, h, w, &|line, i| i * w + line);
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    /// A red disc on a backdrop that goes from light grey to white, with
    /// a white dot inside the disc and one dark speck in the backdrop.
    fn picture() -> RgbaImage {
        RgbaImage::from_fn(80, 60, |x, y| {
            let (dx, dy) = (x as f32 - 40.0, y as f32 - 30.0);
            let r = (dx * dx + dy * dy).sqrt();
            if (38..40).contains(&x) && (28..30).contains(&y) {
                Rgba([255, 255, 255, 255])
            } else if r < 18.0 {
                Rgba([200, 30, 30, 255])
            } else if r < 19.0 {
                // The anti-aliased rim: half disc, half backdrop.
                Rgba([226, 140, 140, 255])
            } else if (5..7).contains(&x) && (5..7).contains(&y) {
                Rgba([40, 40, 40, 255])
            } else {
                let g = 225 + (x * 30 / 80) as u8;
                Rgba([g, g, g, 255])
            }
        })
    }

    #[test]
    fn the_backdrop_goes_and_the_subject_stays() {
        let img = picture();
        let out = remove_background(&img, &BgSettings::default(), 1.0);
        // Backdrop at both ends of the gradient: gone.
        assert_eq!(out.get_pixel(2, 50).0[3], 0);
        assert_eq!(out.get_pixel(77, 2).0[3], 0);
        // The subject: opaque and its colour unchanged.
        assert_eq!(out.get_pixel(40, 40).0, [200, 30, 30, 255]);
        // The white dot inside the subject is not reached by the fill.
        assert_eq!(out.get_pixel(38, 28).0[3], 255);
        // The loose dark speck in the backdrop is removed.
        assert!(out.get_pixel(5, 5).0[3] < 40, "{:?}", out.get_pixel(5, 5));
    }

    #[test]
    fn everywhere_mode_takes_background_coloured_parts_inside_too() {
        let img = picture();
        // No feather, which would blur the tiny hole partly back.
        let s = BgSettings {
            contiguous: false,
            softness: 0.0,
            ..BgSettings::default()
        };
        let out = remove_background(&img, &s, 1.0);
        assert_eq!(out.get_pixel(38, 28).0[3], 0);
        assert_eq!(out.get_pixel(40, 40).0[3], 255);
    }

    #[test]
    fn edge_pixels_lose_the_backdrops_colour() {
        let img = picture();
        let s = BgSettings {
            softness: 0.0,
            ..BgSettings::default()
        };
        let out = remove_background(&img, &s, 1.0);
        // A rim pixel: partly transparent, its colour pulled toward the
        // disc's red rather than the backdrop's pink.
        let rim = (0..80)
            .map(|x| *out.get_pixel(x, 30))
            .find(|p| p.0[3] > 10 && p.0[3] < 245);
        if let Some(p) = rim {
            assert!(p.0[0] > p.0[1] + 60, "{p:?}");
        }
        let plain = remove_background(
            &img,
            &BgSettings {
                clean_edges: false,
                ..s
            },
            1.0,
        );
        // Without cleaning, colours stay as they were.
        for (a, b) in img.pixels().zip(plain.pixels()) {
            assert_eq!(a.0[..3], b.0[..3]);
        }
    }

    #[test]
    fn learnt_colours_cover_a_two_tone_backdrop() {
        let img = RgbaImage::from_fn(40, 40, |x, y| {
            if (12..28).contains(&x) && (12..28).contains(&y) {
                Rgba([20, 160, 40, 255])
            } else if x < 20 {
                Rgba([250, 250, 250, 255])
            } else {
                Rgba([30, 60, 200, 255])
            }
        });
        assert_eq!(learn_background(&img).len(), 2);
        let out = remove_background(&img, &BgSettings::default(), 1.0);
        assert_eq!(out.get_pixel(1, 20).0[3], 0);
        assert_eq!(out.get_pixel(38, 20).0[3], 0);
        assert_eq!(out.get_pixel(20, 20).0[3], 255);
    }

    #[test]
    fn picked_colours_and_tolerance() {
        let img = picture();
        // A picked colour far from the backdrop: nothing goes.
        let s = BgSettings {
            colors: vec![[0, 0, 255]],
            ..BgSettings::default()
        };
        let out = remove_background(&img, &s, 1.0);
        assert!(out.pixels().all(|p| p.0[3] == 255));
        // Tolerance 0 on a gradient keeps most of it.
        let s = BgSettings {
            tolerance: 0.0,
            despeckle: false,
            softness: 0.0,
            ..BgSettings::default()
        };
        let out = remove_background(&img, &s, 1.0);
        let gone = out.pixels().filter(|p| p.0[3] == 0).count();
        let loose = remove_background(&img, &BgSettings::default(), 1.0)
            .pixels()
            .filter(|p| p.0[3] == 0)
            .count();
        assert!(gone < loose, "{gone} {loose}");
    }

    #[test]
    fn a_transparent_picture_is_left_alone() {
        let img = RgbaImage::from_fn(20, 20, |x, y| {
            if (5..15).contains(&x) && (5..15).contains(&y) {
                Rgba([10, 10, 10, 255])
            } else {
                Rgba([0, 0, 0, 0])
            }
        });
        assert_eq!(remove_background(&img, &BgSettings::default(), 1.0), img);
        let empty = RgbaImage::new(0, 0);
        assert_eq!(
            remove_background(&empty, &BgSettings::default(), 1.0),
            empty
        );
    }

    #[test]
    fn feathering_softens_only_the_edge() {
        let img = picture();
        let s = BgSettings {
            softness: 3.0,
            ..BgSettings::default()
        };
        let out = remove_background(&img, &s, 1.0);
        assert_eq!(out.get_pixel(40, 40).0[3], 255);
        assert_eq!(out.get_pixel(2, 50).0[3], 0);
        let partial = (0..80)
            .filter(|x| {
                let a = out.get_pixel(*x, 30).0[3];
                a > 0 && a < 255
            })
            .count();
        assert!(partial >= 4, "{partial}");
    }

    /// A backdrop that drifts away from the colours its border shows (a
    /// soft shadow under the subject, a texture) goes whole: the fill
    /// walks on through small steps of colour, while the subject's edge,
    /// a sharp step, stops it (#231).
    #[test]
    fn a_drifting_backdrop_goes_and_the_subject_stays() {
        // A white backdrop with a grey shadow fading in over 30 px under
        // a red disc, and a faint texture all over.
        let img = RgbaImage::from_fn(120, 90, |x, y| {
            let (dx, dy) = (x as f32 - 60.0, y as f32 - 40.0);
            if (dx * dx + dy * dy).sqrt() < 20.0 {
                return Rgba([200, 30, 30, 255]);
            }
            let (sx, sy) = (x as f32 - 60.0, y as f32 - 68.0);
            let shade = (1.0 - ((sx / 40.0).powi(2) + (sy / 10.0).powi(2)).sqrt()).max(0.0);
            let noise = ((x * 7 + y * 13) % 5) as i32 - 2;
            let v = (250.0 - 90.0 * shade) as i32 + noise;
            let v = v.clamp(0, 255) as u8;
            Rgba([v, v, v.saturating_sub(4), 255])
        });
        let out = remove_background(&img, &BgSettings::default(), 1.0);
        // The darkest of the shadow is 36 Delta E from white, past the
        // tolerance and its band, yet gone.
        assert_eq!(out.get_pixel(60, 68).0[3], 0, "{:?}", out.get_pixel(60, 68));
        assert_eq!(out.get_pixel(3, 3).0[3], 0);
        // The subject keeps its colour, to its edge.
        assert_eq!(out.get_pixel(60, 40).0, [200, 30, 30, 255]);
        assert_eq!(out.get_pixel(60, 24).0[3], 255);
        // (Counted at half opacity: the feather softens the rim.)
        let kept = out.pixels().filter(|p| p.0[3] >= 128).count();
        let disc = (std::f32::consts::PI * 20.0 * 20.0) as usize;
        assert!(
            kept >= disc * 9 / 10 && kept <= disc * 11 / 10,
            "{kept} vs {disc}"
        );
        // Everywhere mode (no fill) still goes by colour alone: the
        // shadow's dark middle stays.
        let s = BgSettings {
            contiguous: false,
            ..BgSettings::default()
        };
        assert_eq!(remove_background(&img, &s, 1.0).get_pixel(60, 68).0[3], 255);
    }
}
