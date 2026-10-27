//! The fonts of a version 16+ package: `font/fontTable.dat`, listing every
//! face the text uses, and `embed/embedding<n>`, the faces embedded. See
//! docs/cdr-format.md, Embedded fonts.

use super::Buf;
use crate::eot::{self, EotNames};
use std::sync::Arc;

/// A face the text uses, with its font file when one was found.
#[derive(Debug, Clone)]
pub struct FontFace {
    pub family: String,
    pub bold: bool,
    pub italic: bool,
    /// The TrueType or OpenType file (a single font, not a collection).
    pub data: Option<Arc<Vec<u8>>>,
}

/// What the font table and an embedding header say about a face.
struct FaceInfo {
    flags: u32,
    panose: [u8; 10],
    kind: u32,
    postscript: String,
    version: String,
    style: String,
    /// The face may be embedded (OS/2 `fsType` allows it).
    embeddable: bool,
    eot: Option<EotNames>,
}

/// The style flags of a face in font records: 0x40 regular, 0x1000 bold,
/// 0x80 italic, 0x2000 bold italic.
fn face_flags(bold: bool, italic: bool) -> u32 {
    match (bold, italic) {
        (false, false) => 0x40,
        (true, false) => 0x1000,
        (false, true) => 0x80,
        (true, true) => 0x2000,
    }
}

fn be16(b: &[u8], at: usize) -> u16 {
    b.get(at..at + 2)
        .map_or(0, |s| u16::from_be_bytes([s[0], s[1]]))
}

fn be32(b: &[u8], at: usize) -> u32 {
    b.get(at..at + 4)
        .map_or(0, |s| u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
}

fn info(face: &FontFace) -> FaceInfo {
    let style_name = match (face.bold, face.italic) {
        (false, false) => "Regular",
        (true, false) => "Bold",
        (false, true) => "Italic",
        (true, true) => "Bold Italic",
    };
    let mut out = FaceInfo {
        flags: face_flags(face.bold, face.italic),
        panose: [0; 10],
        kind: 4,
        postscript: face.family.replace(' ', ""),
        version: String::new(),
        style: style_name.to_string(),
        embeddable: false,
        eot: None,
    };
    let Some(data) = &face.data else {
        return out;
    };
    let Ok(f) = ttf_parser::Face::parse(data, 0) else {
        return out;
    };
    let name = |id: u16| {
        f.names()
            .into_iter()
            .filter(|n| n.name_id == id)
            .find_map(|n| n.to_string())
            .unwrap_or_default()
    };
    let raw = f.raw_face();
    let os2 = raw
        .table(ttf_parser::Tag::from_bytes(b"OS/2"))
        .unwrap_or(&[]);
    let head = raw
        .table(ttf_parser::Tag::from_bytes(b"head"))
        .unwrap_or(&[]);
    if let Some(p) = os2.get(32..42) {
        out.panose.copy_from_slice(p);
    }
    let fs_type = be16(os2, 8);
    // Restricted licence (2) or bitmaps only (0x200): not embedded.
    out.embeddable = !os2.is_empty() && fs_type & 0x000f != 0x0002 && fs_type & 0x0200 == 0;
    // 5 for PostScript outlines, 4 for TrueType ones, as files have them.
    if raw.table(ttf_parser::Tag::from_bytes(b"CFF ")).is_some() {
        out.kind = 5;
    }
    let ps = name(6);
    if !ps.is_empty() {
        out.postscript = ps;
    }
    out.version = name(5);
    let style = name(2);
    if !style.is_empty() {
        out.style = style;
    }
    let mut family = name(1);
    if family.is_empty() {
        family = face.family.clone();
    }
    out.eot = Some(EotNames {
        panose: out.panose,
        italic: face.italic,
        weight: u32::from(be16(os2, 4)).max(if face.bold { 700 } else { 400 }),
        fs_type,
        unicode_ranges: [be32(os2, 42), be32(os2, 46), be32(os2, 50), be32(os2, 54)],
        code_page_ranges: if be16(os2, 0) >= 1 {
            [be32(os2, 78), be32(os2, 82)]
        } else {
            [0; 2]
        },
        checksum_adjustment: be32(head, 8),
        family,
        style: out.style.clone(),
        version: out.version.clone(),
        full: name(4),
    });
    out
}

/// The bytes the faces that may be embedded add to a file (their font
/// files), 0 when none may be.
pub fn embedded_size(faces: &[FontFace]) -> u64 {
    faces
        .iter()
        .filter_map(|f| f.data.as_ref().filter(|_| info(f).embeddable))
        .map(|d| d.len() as u64)
        .sum()
}

/// A string as the font table stores it: its length in UTF-16 units with
/// the closing zero, then the units.
fn string(b: &mut Buf, s: &str) {
    let units: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
    b.u32(units.len() as u32);
    for u in units {
        b.u16(u);
    }
}

/// The fonts members of a package: the font table, and an embedding per
/// face that may be embedded when `embed` is set (in table order).
pub fn members(faces: &[FontFace], embed: bool) -> Vec<(String, Vec<u8>)> {
    if faces.is_empty() {
        return Vec::new();
    }
    let infos: Vec<FaceInfo> = faces.iter().map(info).collect();
    let mut table = Buf::default();
    table.u32(0).u32(0).u32(faces.len() as u32);
    let mut embeddings = Vec::new();
    for (i, (face, inf)) in faces.iter().zip(&infos).enumerate() {
        let embedded = embed && inf.embeddable && face.data.is_some();
        let mut e = Buf::default();
        // The face's number, then the character set (Western).
        e.u16(i as u16 + 1).u16(0).u32(inf.flags);
        for p in inf.panose {
            e.u8(p);
        }
        string(&mut e, &face.family);
        e.u8(0);
        string(&mut e, "Western");
        e.u32(u32::from(embedded)).u32(inf.kind);
        string(&mut e, &inf.postscript);
        string(&mut e, &inf.version);
        string(&mut e, &inf.style);
        let e = e.done();
        table.u32(0).u32(e.len() as u32);
        table.bytes(&e);
        if embedded {
            if let (Some(data), Some(names)) = (&face.data, &inf.eot) {
                embeddings.push(embedding(inf.kind, &inf.version, data, names));
            }
        }
    }
    let mut table = table.done();
    let size = (table.len() - 8) as u32;
    table[4..8].copy_from_slice(&size.to_le_bytes());
    let mut out = vec![("font/fontTable.dat".to_string(), table)];
    for (n, m) in embeddings.into_iter().enumerate() {
        out.push((format!("embed/embedding{n}"), m));
    }
    out
}

/// An `embed/` member: u32 1, u32 0x803, the face's kind byte, its
/// version string in UTF-16 with a closing zero, zeros up to 2705 bytes,
/// then the EOT structure.
const EMBEDDING_HEADER: usize = 2705;

fn embedding(kind: u32, version: &str, data: &[u8], names: &EotNames) -> Vec<u8> {
    let mut m = Vec::with_capacity(EMBEDDING_HEADER + data.len() + 256);
    m.extend_from_slice(&1u32.to_le_bytes());
    m.extend_from_slice(&0x803u32.to_le_bytes());
    m.push(kind as u8);
    for u in version.encode_utf16().chain(std::iter::once(0)) {
        m.extend_from_slice(&u.to_le_bytes());
    }
    m.truncate(EMBEDDING_HEADER);
    m.resize(EMBEDDING_HEADER, 0);
    m.extend(eot::encode(data, names));
    m
}
