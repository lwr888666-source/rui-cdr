//! Exact, topology-preserving path inspection and coordinate patches.
//! Indices are zero-based BezPath element indices, stable while topology stays.

use crate::geometry::{Affine, BezPath, PathEl, Point, Rect, Shape as _};
use crate::{Document, Error, Result, ShapeId, ShapeKind};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};

pub const MAX_COORDINATE_MM: f64 = 1_000_000_000.0;
pub const MAX_ELEMENTS: usize = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl From<Point> for Coordinate {
    fn from(p: Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}
impl Coordinate {
    fn point(self) -> Point {
        Point::new(self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateSpace {
    Local,
    Page,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeEdit {
    pub node_index: usize,
    #[serde(default)]
    pub position: Option<Coordinate>,
    #[serde(default)]
    pub in_handle: Option<Coordinate>,
    #[serde(default)]
    pub out_handle: Option<Coordinate>,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathNodePatch {
    pub shape_id: ShapeId,
    pub geometry_token: String,
    pub coordinate_space: CoordinateSpace,
    pub edits: Vec<NodeEdit>,
    /// AI edits keep page-space bounds. User scripts may explicitly opt out.
    #[serde(default = "yes")]
    pub preserve_bounds: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeGeometry {
    pub node_index: usize,
    pub subpath_index: usize,
    pub position: Coordinate,
    pub in_handle: Option<Coordinate>,
    pub out_handle: Option<Coordinate>,
    pub page_position: Coordinate,
    pub page_in_handle: Option<Coordinate>,
    pub page_out_handle: Option<Coordinate>,
    pub node_type: crate::nodes::NodeType,
}

#[derive(Debug, Clone, Serialize)]
pub struct SegmentGeometry {
    pub element_index: usize,
    pub kind: &'static str,
    pub from: Coordinate,
    pub to: Coordinate,
    pub controls: Vec<Coordinate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubpathGeometry {
    pub subpath_index: usize,
    pub closed: bool,
    pub node_indices: Vec<usize>,
    pub segments: Vec<SegmentGeometry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VectorGeometry {
    pub shape_id: ShapeId,
    pub object_type: &'static str,
    pub selected: bool,
    pub units: &'static str,
    pub geometry_token: String,
    pub local_to_page: [f64; 6],
    pub page_to_local: [f64; 6],
    pub closed: bool,
    pub nodes: Vec<NodeGeometry>,
    pub subpaths: Vec<SubpathGeometry>,
}

fn bad(message: impl Into<String>) -> Error {
    Error::PathNodeEdit(message.into())
}
fn check_point(p: Point) -> Result<()> {
    if !p.is_finite() || p.x.abs() > MAX_COORDINATE_MM || p.y.abs() > MAX_COORDINATE_MM {
        return Err(bad(
            "coordinates must be finite millimetres within +/-1000000000",
        ));
    }
    Ok(())
}

fn end(el: &PathEl) -> Option<Point> {
    match *el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) | PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => {
            Some(p)
        }
        PathEl::ClosePath => None,
    }
}

/// Validate without normalizing or elevating quadratics.
fn layout(path: &BezPath) -> Result<Vec<SubpathGeometry>> {
    layout_elements(path.elements())
}

fn layout_elements(elements: &[PathEl]) -> Result<Vec<SubpathGeometry>> {
    if elements.len() > MAX_ELEMENTS {
        return Err(bad("path exceeds the node-edit limit"));
    }
    let mut subs: Vec<SubpathGeometry> = Vec::new();
    let mut last = Point::ZERO;
    let mut start = Point::ZERO;
    for (index, el) in elements.iter().enumerate() {
        if let PathEl::MoveTo(p) = *el {
            check_point(p)?;
            subs.push(SubpathGeometry {
                subpath_index: subs.len(),
                closed: false,
                node_indices: Vec::new(),
                segments: Vec::new(),
            });
            start = p;
            last = p;
            continue;
        }
        let sub = subs
            .last_mut()
            .ok_or_else(|| bad("path must start with MoveTo"))?;
        if sub.closed {
            return Err(bad("a closed subpath must be followed by MoveTo"));
        }
        let (kind, to, controls): (&str, Point, Vec<Coordinate>) = match *el {
            PathEl::LineTo(p) => ("line", p, Vec::new()),
            PathEl::QuadTo(c, p) => ("quadratic", p, vec![c.into()]),
            PathEl::CurveTo(a, b, p) => ("cubic", p, vec![a.into(), b.into()]),
            PathEl::ClosePath => {
                sub.closed = true;
                ("close", start, Vec::new())
            }
            PathEl::MoveTo(_) => continue,
        };
        check_point(to)?;
        for p in &controls {
            check_point(p.point())?;
        }
        sub.segments.push(SegmentGeometry {
            element_index: index,
            kind,
            from: last.into(),
            to: to.into(),
            controls,
        });
        last = to;
    }
    if subs.is_empty() || subs.iter().any(|s| s.segments.is_empty()) {
        return Err(bad("path has an empty subpath"));
    }
    Ok(subs)
}

fn transform(doc: &Document, id: ShapeId) -> Result<Affine> {
    let shape = doc.find_shape(id).ok_or(Error::ShapeNotFound(id))?;
    let t = doc.parent_transform(id).ok_or(Error::ShapeNotFound(id))? * shape.transform;
    if t.as_coeffs().iter().any(|v| !v.is_finite())
        || !t.determinant().is_finite()
        || t.determinant().abs() < 1e-15
    {
        return Err(bad("the object's transform is not invertible"));
    }
    Ok(t)
}

fn token(id: ShapeId, path: &BezPath, closed: bool, t: Affine) -> String {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hash);
    closed.hash(&mut hash);
    for v in t.as_coeffs() {
        v.to_bits().hash(&mut hash);
    }
    for el in path.elements() {
        let (kind, points): (u8, Vec<Point>) = match *el {
            PathEl::MoveTo(p) => (0, vec![p]),
            PathEl::LineTo(p) => (1, vec![p]),
            PathEl::QuadTo(c, p) => (2, vec![c, p]),
            PathEl::CurveTo(a, b, p) => (3, vec![a, b, p]),
            PathEl::ClosePath => (4, Vec::new()),
        };
        kind.hash(&mut hash);
        for p in points {
            p.x.to_bits().hash(&mut hash);
            p.y.to_bits().hash(&mut hash);
        }
    }
    format!("path-v1-{:016x}", hash.finish())
}

pub fn geometry(doc: &Document, id: ShapeId, selected: bool) -> Result<VectorGeometry> {
    let shape = doc.find_shape(id).ok_or(Error::ShapeNotFound(id))?;
    let ShapeKind::Path { path, closed } = &shape.kind else {
        return Err(bad(
            "object is not a native path; no automatic conversion is performed",
        ));
    };
    if shape.spline.is_some() {
        return Err(bad(
            "B-spline outlines must be edited through their spline control points",
        ));
    }
    let t = transform(doc, id)?;
    let mut subpaths = layout(path)?;
    let slots = coordinate_slots(path);
    let mut nodes = Vec::with_capacity(slots.len());
    let mut subpath_index = 0;
    for (index, el) in path.elements().iter().enumerate() {
        if matches!(el, PathEl::MoveTo(_)) && index > 0 {
            subpath_index += 1;
        }
        let Some(slot) = slots.get(&index) else {
            continue;
        };
        let Some(pos) = end(el) else {
            continue;
        };
        let ctrl_in = slot.incoming.and_then(|s| slot_point(path, s));
        let ctrl_out = slot.outgoing.and_then(|s| slot_point(path, s));
        for p in [Some(pos), ctrl_in, ctrl_out].into_iter().flatten() {
            check_point(t * p)?;
        }
        if let Some(sub) = subpaths.get_mut(subpath_index) {
            sub.node_indices.push(index);
        }
        let n = crate::nodes::Node {
            index,
            pos,
            ctrl_in,
            ctrl_out,
            is_start: matches!(el, PathEl::MoveTo(_)),
        };
        nodes.push(NodeGeometry {
            node_index: index,
            subpath_index,
            position: pos.into(),
            in_handle: ctrl_in.map(Into::into),
            out_handle: ctrl_out.map(Into::into),
            page_position: (t * pos).into(),
            page_in_handle: ctrl_in.map(|p| (t * p).into()),
            page_out_handle: ctrl_out.map(|p| (t * p).into()),
            node_type: crate::nodes::node_type_of_with(&n, &shape.node_types),
        });
    }
    Ok(VectorGeometry {
        shape_id: id,
        object_type: "path",
        selected,
        units: "mm",
        geometry_token: token(id, path, *closed, t),
        local_to_page: t.as_coeffs(),
        page_to_local: t.inverse().as_coeffs(),
        closed: *closed,
        nodes,
        subpaths,
    })
}

fn locked(doc: &Document, id: ShapeId) -> bool {
    let mut ids = doc.ancestors(id);
    ids.push(id);
    ids.iter()
        .any(|id| doc.find_shape(*id).is_some_and(|s| s.locked))
        || ids
            .first()
            .and_then(|id| doc.shape(*id).ok())
            .is_some_and(|(l, _)| l.locked)
}

// Slots are coordinates of the original elements, never formatted path text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Slot {
    Anchor(usize),
    First(usize),
    Second(usize),
    Quadratic(usize),
}

fn control_slot(path: &BezPath, at: usize, incoming: bool) -> Option<Slot> {
    match path.elements().get(at)? {
        PathEl::CurveTo(..) => Some(if incoming {
            Slot::Second(at)
        } else {
            Slot::First(at)
        }),
        PathEl::QuadTo(..) => Some(Slot::Quadratic(at)),
        _ => None,
    }
}

struct NodeSlots {
    incoming: Option<Slot>,
    outgoing: Option<Slot>,
    twin: Option<usize>,
}

/// Index aliases once, avoiding full-path scans for every supplied handle.
fn coordinate_slots(path: &BezPath) -> BTreeMap<usize, NodeSlots> {
    let mut twins = BTreeMap::new();
    let mut aliases = BTreeSet::new();
    let mut start = 0;
    for (i, el) in path.elements().iter().enumerate() {
        if matches!(el, PathEl::MoveTo(_)) {
            start = i;
        }
        if matches!(el, PathEl::ClosePath) {
            if let Some(last) = i.checked_sub(1).filter(|last| *last > start) {
                if let (Some(a), Some(b)) = (
                    path.elements().get(start).and_then(end),
                    path.elements().get(last).and_then(end),
                ) {
                    if (a - b).hypot() < 1e-9 {
                        twins.insert(start, last);
                        aliases.insert(last);
                    }
                }
            }
        }
    }
    path.elements()
        .iter()
        .enumerate()
        .filter_map(|(index, el)| {
            if end(el).is_none() || aliases.contains(&index) {
                return None;
            }
            let twin = twins.get(&index).copied();
            Some((
                index,
                NodeSlots {
                    incoming: control_slot(path, twin.unwrap_or(index), true),
                    outgoing: index
                        .checked_add(1)
                        .and_then(|i| control_slot(path, i, false)),
                    twin,
                },
            ))
        })
        .collect()
}

fn slot_point(path: &BezPath, slot: Slot) -> Option<Point> {
    let index = match slot {
        Slot::Anchor(i) | Slot::First(i) | Slot::Second(i) | Slot::Quadratic(i) => i,
    };
    match (slot, path.elements().get(index)?) {
        (Slot::Anchor(_), el) => end(el),
        (Slot::First(_), PathEl::CurveTo(p, _, _))
        | (Slot::Second(_), PathEl::CurveTo(_, p, _))
        | (Slot::Quadratic(_), PathEl::QuadTo(p, _)) => Some(*p),
        _ => None,
    }
}

fn bounds_equal(a: Rect, b: Rect) -> bool {
    let tolerance = 1e-9 * a.width().abs().max(a.height().abs()).max(1.0);
    [a.x0 - b.x0, a.y0 - b.y0, a.x1 - b.x1, a.y1 - b.y1]
        .iter()
        .all(|v| v.abs() <= tolerance)
}

/// A validated batch, still separate from the document until every patch passes.
pub struct PreparedEdits {
    pub changed_nodes: usize,
    paths: Vec<(ShapeId, BezPath)>,
}

pub fn prepare(doc: &Document, patches: &[PathNodePatch]) -> Result<PreparedEdits> {
    if patches.len() > 256 {
        return Err(bad("too many objects in one node edit"));
    }
    let mut paths = Vec::new();
    let mut changed_nodes = 0;
    let mut ids = BTreeSet::new();
    for patch in patches {
        if !ids.insert(patch.shape_id) {
            return Err(bad("duplicate object in the edit batch"));
        }
        if locked(doc, patch.shape_id) {
            return Err(bad("the object or its container is locked"));
        }
        let info = geometry(doc, patch.shape_id, true)?;
        if info.geometry_token != patch.geometry_token {
            return Err(bad(
                "stale geometry token; read the selected geometry again",
            ));
        }
        if patch.edits.len() > 4096 {
            return Err(bad("too many node edits"));
        }
        let shape = doc
            .find_shape(patch.shape_id)
            .ok_or(Error::ShapeNotFound(patch.shape_id))?;
        let ShapeKind::Path { path, .. } = &shape.kind else {
            return Err(bad("object is not a path"));
        };
        let slots = coordinate_slots(path);
        let t = Affine::new(info.local_to_page);
        let mut writes = BTreeMap::<Slot, Point>::new();
        let mut explicit = BTreeSet::new();
        let mut touched = BTreeSet::new();
        for edit in &patch.edits {
            if !explicit.insert(edit.node_index) {
                return Err(bad("duplicate node index"));
            }
            let node_slots = slots
                .get(&edit.node_index)
                .ok_or_else(|| bad(format!("no canonical node {}", edit.node_index)))?;
            let mut changed = false;
            for (incoming, value) in [
                (None, edit.position),
                (Some(true), edit.in_handle),
                (Some(false), edit.out_handle),
            ] {
                let Some(value) = value else {
                    continue;
                };
                check_point(value.point())?;
                let slot = match incoming {
                    None => Slot::Anchor(edit.node_index),
                    Some(inside) => (if inside {
                        node_slots.incoming
                    } else {
                        node_slots.outgoing
                    })
                    .ok_or_else(|| {
                        bad("the specified handle does not exist; segment types cannot be changed")
                    })?,
                };
                let old = slot_point(path, slot).ok_or_else(|| bad("invalid coordinate slot"))?;
                // Preserve exact old bits for page-space round trips without edits.
                let old_in_space = if patch.coordinate_space == CoordinateSpace::Page {
                    t * old
                } else {
                    old
                };
                let value = if value.point() == old_in_space {
                    old
                } else if patch.coordinate_space == CoordinateSpace::Page {
                    t.inverse() * value.point()
                } else {
                    value.point()
                };
                check_point(value)?;
                check_point(t * value)?;
                if let Some(previous) = writes.insert(slot, value) {
                    if previous != value {
                        return Err(bad("conflicting edits to a shared control point"));
                    }
                }
                if old != value {
                    changed = true;
                }
                if incoming.is_none() {
                    if let Some(twin) = node_slots.twin {
                        writes.insert(Slot::Anchor(twin), value);
                    }
                }
            }
            if changed {
                touched.insert(edit.node_index);
            }
        }
        // A quadratic has one physical control point shared by two nodes.
        // Both aliases must be explicitly addressed so no omitted handle moves.
        let supplied: BTreeMap<_, _> = patch.edits.iter().map(|e| (e.node_index, e)).collect();
        for (index, node_slots) in &slots {
            for incoming in [true, false] {
                let slot = if incoming {
                    node_slots.incoming
                } else {
                    node_slots.outgoing
                };
                if let Some(slot @ Slot::Quadratic(_)) = slot {
                    if writes.contains_key(&slot)
                        && !supplied.get(index).is_some_and(|e| {
                            if incoming {
                                e.in_handle.is_some()
                            } else {
                                e.out_handle.is_some()
                            }
                        })
                    {
                        return Err(bad(
                            "a quadratic handle is shared; specify both endpoint handle aliases",
                        ));
                    }
                }
            }
        }
        let mut elements = path.elements().to_vec();
        for (slot, value) in writes {
            let index = match slot {
                Slot::Anchor(i) | Slot::First(i) | Slot::Second(i) | Slot::Quadratic(i) => i,
            };
            let el = elements
                .get_mut(index)
                .ok_or_else(|| bad("coordinate index out of range"))?;
            match (slot, el) {
                (
                    Slot::Anchor(_),
                    PathEl::MoveTo(p)
                    | PathEl::LineTo(p)
                    | PathEl::QuadTo(_, p)
                    | PathEl::CurveTo(_, _, p),
                )
                | (Slot::First(_), PathEl::CurveTo(p, _, _))
                | (Slot::Second(_), PathEl::CurveTo(_, p, _))
                | (Slot::Quadratic(_), PathEl::QuadTo(p, _)) => *p = value,
                _ => return Err(bad("invalid coordinate slot")),
            }
        }
        let next = BezPath::from_vec(elements);
        layout(&next)?;
        if !coordinate_slots(&next).keys().eq(slots.keys()) {
            return Err(bad("edit would merge or separate a closing node"));
        }
        if patch.preserve_bounds
            && !bounds_equal(
                (t * path.clone()).bounding_box(),
                (t * next.clone()).bounding_box(),
            )
        {
            return Err(bad("edit changes the object's page bounds; preserving size and all other nodes is impossible for this edit"));
        }
        if !touched.is_empty() {
            changed_nodes += touched.len();
            paths.push((patch.shape_id, next));
        }
    }
    Ok(PreparedEdits {
        changed_nodes,
        paths,
    })
}

pub fn apply(doc: &mut Document, patches: &[PathNodePatch]) -> Result<()> {
    let prepared = prepare(doc, patches)?;
    for (id, next) in prepared.paths {
        let shape = doc.find_shape_mut(id).ok_or(Error::ShapeNotFound(id))?;
        if let ShapeKind::Path { path, .. } = &mut shape.kind {
            *path = next;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Engine, Fill, Shape, Stroke};

    fn fixture() -> (Engine, ShapeId, ShapeId) {
        let mut doc = Document::default();
        let layer = doc.pages[0].layers[0].id;
        let id = doc.ids_mut().shape();
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((100.0, 0.0));
        path.line_to((100.0, 100.0));
        path.line_to((70.0, 100.0));
        path.curve_to((70.0, 80.0), (65.0, 52.0), (50.0, 52.0));
        path.curve_to((35.0, 52.0), (30.0, 80.0), (30.0, 100.0));
        path.line_to((0.0, 100.0));
        path.line_to((0.0, 0.0));
        path.close_path();
        // A separate open quadratic subpath must keep its degree and bits.
        path.move_to((10.0, 20.0));
        path.quad_to((20.0, 40.0), (30.0, 20.0));
        let mut shape = Shape::new(id, ShapeKind::Path { path, closed: true });
        shape.name = Some("irregular outline".into());
        shape.fill = Fill::Solid(crate::Color::rgb8(51, 102, 204));
        shape.stroke = Some(Stroke::default());
        shape.opacity = 0.75;
        shape.node_types.insert(4, crate::nodes::NodeType::Smooth);
        let other_id = doc.ids_mut().shape();
        let other = Shape::new(
            other_id,
            ShapeKind::Rect {
                rect: Rect::new(120.0, 0.0, 140.0, 20.0),
                radius: 0.0,
                corners: None,
            },
        );
        let mut engine = Engine::new(doc);
        engine
            .run_batch(
                "Fixture",
                &[
                    Command::AddShape { layer, shape },
                    Command::AddShape {
                        layer,
                        shape: other,
                    },
                ],
            )
            .expect("fixture");
        engine.mark_saved();
        (engine, id, other_id)
    }

    fn patch(doc: &Document, id: ShapeId) -> PathNodePatch {
        PathNodePatch {
            shape_id: id,
            geometry_token: geometry(doc, id, true).expect("geometry").geometry_token,
            coordinate_space: CoordinateSpace::Local,
            preserve_bounds: true,
            edits: vec![NodeEdit {
                node_index: 4,
                position: Some(Coordinate { x: 50.0, y: 50.0 }),
                in_handle: None,
                out_handle: None,
            }],
        }
    }

    #[test]
    fn reads_exact_nodes_handles_segments_and_subpaths() {
        let (engine, id, _) = fixture();
        let info = geometry(engine.document(), id, true).expect("read");
        assert!(info.selected && info.closed);
        assert_eq!(info.units, "mm");
        assert_eq!(info.subpaths.len(), 2);
        assert!(info.subpaths[0].closed && !info.subpaths[1].closed);
        assert_eq!(info.subpaths[0].node_indices, [0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(info.subpaths[1].node_indices, [9, 10]);
        let node = &info.nodes[4];
        assert_eq!(node.position, Coordinate { x: 50.0, y: 52.0 });
        assert_eq!(node.in_handle, Some(Coordinate { x: 65.0, y: 52.0 }));
        assert_eq!(node.out_handle, Some(Coordinate { x: 35.0, y: 52.0 }));
        assert_eq!(info.subpaths[1].segments[0].kind, "quadratic");
    }

    #[test]
    fn command_changes_only_explicit_slots_and_is_one_undo_step() {
        let (mut engine, id, other) = fixture();
        let before = engine.document().clone();
        let mut edits = patch(&before, id);
        edits.edits[0].in_handle = Some(Coordinate { x: 65.0, y: 50.0 });
        let command = Command::EditPathNodes {
            patches: vec![edits],
        };
        let json = serde_json::to_string(&command).expect("serialise");
        let command: Command = serde_json::from_str(&json).expect("deserialise");
        engine.run(&command).expect("edit");
        let after = engine.document().clone();
        assert_ne!(after, before);
        assert_eq!(after.find_shape(other), before.find_shape(other));
        let old_shape = before.find_shape(id).expect("old");
        let mut expected = old_shape.clone();
        if let ShapeKind::Path { path, .. } = &mut expected.kind {
            let mut els = path.elements().to_vec();
            if let PathEl::CurveTo(_, incoming, position) = &mut els[4] {
                *incoming = Point::new(65.0, 50.0);
                *position = Point::new(50.0, 50.0);
            }
            *path = BezPath::from_vec(els);
        }
        assert_eq!(after.find_shape(id), Some(&expected));
        let mut restored = after.clone();
        restored.find_shape_mut(id).expect("shape").kind = old_shape.kind.clone();
        assert_eq!(
            restored, before,
            "IDs, order, styles and every other property"
        );
        assert_eq!(engine.undo_label(), Some("Edit Path Nodes"));
        engine.undo().expect("undo");
        assert_eq!(engine.document(), &before);
        engine.redo().expect("redo");
        assert_eq!(engine.document(), &after);
    }

    #[test]
    fn rejects_invalid_indices_coordinates_types_tokens_bounds_and_locked_objects_atomically() {
        let (mut engine, id, other) = fixture();
        let before = engine.document().clone();
        let original = patch(&before, id);
        let mut invalids = Vec::new();
        let mut p = original.clone();
        p.edits[0].node_index = 9999;
        invalids.push(p);
        let mut p = original.clone();
        p.edits[0].position = Some(Coordinate {
            x: f64::NAN,
            y: 0.0,
        });
        invalids.push(p);
        let mut p = original.clone();
        p.edits[0].position = Some(Coordinate {
            x: f64::INFINITY,
            y: 0.0,
        });
        invalids.push(p);
        let mut p = original.clone();
        p.edits[0].position = Some(Coordinate {
            x: MAX_COORDINATE_MM + 1.0,
            y: 0.0,
        });
        invalids.push(p);
        let mut p = original.clone();
        p.shape_id = other;
        invalids.push(p);
        let mut p = original.clone();
        p.shape_id = ShapeId(u64::MAX);
        invalids.push(p);
        let mut p = original.clone();
        p.geometry_token = "old".into();
        invalids.push(p);
        let mut p = original.clone();
        p.edits[0].node_index = 1;
        p.edits[0].position = Some(Coordinate { x: 110.0, y: 0.0 });
        invalids.push(p);
        let mut p = original.clone();
        p.edits[0].node_index = 1;
        p.edits[0].in_handle = Some(Coordinate { x: 1.0, y: 1.0 });
        invalids.push(p);
        for invalid in invalids {
            assert!(engine
                .run(&Command::EditPathNodes {
                    patches: vec![invalid]
                })
                .is_err());
            assert_eq!(engine.document(), &before);
            assert!(!engine.is_dirty());
        }
        // A valid first patch followed by an invalid object never commits.
        let mut invalid = original.clone();
        invalid.shape_id = other;
        let mut batch_doc = before.clone();
        assert!(apply(&mut batch_doc, &[original.clone(), invalid]).is_err());
        assert_eq!(batch_doc, before);
        let mut locked_doc = before.clone();
        locked_doc.find_shape_mut(id).expect("shape").locked = true;
        let locked_before = locked_doc.clone();
        assert!(apply(&mut locked_doc, &[original]).is_err());
        assert_eq!(locked_doc, locked_before);
    }

    #[test]
    fn quadratic_controls_are_shared_and_never_elevated() {
        let (engine, id, _) = fixture();
        let mut doc = engine.document().clone();
        let mut p = patch(&doc, id);
        p.edits = vec![NodeEdit {
            node_index: 9,
            position: None,
            in_handle: None,
            out_handle: Some(Coordinate { x: 20.0, y: 41.0 }),
        }];
        assert!(apply(&mut doc, &[p.clone()]).is_err());
        p.edits.push(NodeEdit {
            node_index: 10,
            position: None,
            in_handle: Some(Coordinate { x: 20.0, y: 41.0 }),
            out_handle: None,
        });
        apply(&mut doc, &[p]).expect("both shared aliases");
        let ShapeKind::Path { path, .. } = &doc.find_shape(id).expect("shape").kind else {
            panic!("path")
        };
        assert_eq!(
            path.elements()[10],
            PathEl::QuadTo(Point::new(20.0, 41.0), Point::new(30.0, 20.0))
        );
    }

    #[test]
    fn closing_aliases_stay_connected_and_malformed_or_singular_paths_are_rejected() {
        let (engine, id, _) = fixture();
        let mut doc = engine.document().clone();
        let before = doc.clone();
        let mut p = patch(&doc, id);
        p.edits[0].node_index = 0;
        p.edits[0].position = Some(Coordinate { x: 0.5, y: 0.0 });
        apply(&mut doc, &[p]).expect("move shared closing anchor");
        let ShapeKind::Path { path, .. } = &doc.find_shape(id).expect("shape").kind else {
            panic!("path")
        };
        assert_eq!(path.elements()[0], PathEl::MoveTo(Point::new(0.5, 0.0)));
        assert_eq!(path.elements()[7], PathEl::LineTo(Point::new(0.5, 0.0)));
        let info = geometry(&doc, id, true).expect("same canonical topology");
        assert!(!info.nodes.iter().any(|n| n.node_index == 7));
        for elements in [
            vec![],
            vec![PathEl::LineTo(Point::ZERO)],
            vec![PathEl::MoveTo(Point::ZERO)],
            vec![
                PathEl::MoveTo(Point::ZERO),
                PathEl::ClosePath,
                PathEl::LineTo(Point::new(1.0, 1.0)),
            ],
        ] {
            assert!(layout_elements(&elements).is_err());
            // BezPath itself rejects a first element other than MoveTo.
            // Validate such raw data before constructing the library type.
            if matches!(elements.first(), Some(PathEl::LineTo(_))) {
                continue;
            }
            let mut invalid = before.clone();
            invalid.find_shape_mut(id).expect("shape").kind = ShapeKind::Path {
                path: BezPath::from_vec(elements),
                closed: false,
            };
            assert!(geometry(&invalid, id, true).is_err());
        }
        let mut invalid = before.clone();
        invalid.find_shape_mut(id).expect("shape").transform = Affine::scale_non_uniform(1.0, 0.0);
        assert!(geometry(&invalid, id, true).is_err());
        let mut locked = before.clone();
        locked.pages[0].layers[0].locked = true;
        assert!(apply(&mut locked, &[patch(&before, id)]).is_err());
    }

    #[test]
    fn page_coordinates_include_nested_parent_transforms_and_round_trip_exactly() {
        let (engine, id, _) = fixture();
        let mut doc = engine.document().clone();
        let child = doc.pages[0].layers[0].shapes.remove(0);
        let group_id = doc.ids_mut().shape();
        let mut group = Shape::new(
            group_id,
            ShapeKind::Group {
                children: vec![child],
            },
        );
        group.transform =
            Affine::translate((12.1234, 18.9876)) * Affine::scale_non_uniform(1.2, 0.75);
        doc.pages[0].layers[0].shapes.push(group);
        doc.find_shape_mut(id).expect("child").transform = Affine::translate((0.0123, -0.0456));
        let before = doc.clone();
        let g = geometry(&doc, id, true).expect("read");
        let n = &g.nodes[4];
        assert_ne!(n.position, n.page_position);
        let mut p = patch(&doc, id);
        p.coordinate_space = CoordinateSpace::Page;
        p.edits[0].position = Some(n.page_position);
        assert_eq!(prepare(&doc, &[p.clone()]).expect("noop").changed_nodes, 0);
        apply(&mut doc, &[p.clone()]).expect("noop");
        assert_eq!(doc, before);
        let t = Affine::new(g.local_to_page);
        p.edits[0].position = Some((t * Point::new(50.0, 50.0)).into());
        apply(&mut doc, &[p.clone()]).expect("page edit");
        let n = &geometry(&doc, id, true).expect("new geometry").nodes[4];
        assert!((n.position.y - 50.0).abs() < 1e-12);
        assert!(apply(&mut doc, &[p]).is_err(), "stale token");
    }
}
