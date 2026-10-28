//! The selected-vector bridge shared by AI context and the script dispatcher.

use super::{HostInner, Trust};
use serde_json::{json, Value};
use tracedraw_core::{path_nodes, Command, Document, ShapeId, ShapeKind};

pub(super) fn selected_geometry(doc: &Document, selection: &[ShapeId], mut budget: usize) -> Value {
    let mut objects = Vec::new();
    let mut unsupported = Vec::new();
    for id in selection {
        match path_nodes::geometry(doc, *id, true) {
            Ok(geometry) if geometry.nodes.len() <= budget => {
                budget -= geometry.nodes.len();
                objects.push(json!(geometry));
            }
            Ok(_) => unsupported.push(json!({"shape_id": id, "reason": "geometry exceeds the request node budget; inspect it with GetSelectedVectorGeometry in the script"})),
            Err(error) => unsupported.push(json!({"shape_id": id, "reason": error.to_string()})),
        }
    }
    json!({"coordinate_system": "millimetres; page origin bottom-left, x right, y up; local coordinates use local_to_page [a,b,c,d,e,f]", "indexing": "zero-based original path element indices; closing twins use the start node index", "objects": objects, "unsupported": unsupported})
}

impl HostInner {
    pub(super) fn node_api(&mut self, method: &str, args: &Value) -> Result<Value, String> {
        if self.trust == Trust::Untrusted {
            self.node_only = true;
            if self
                .commands
                .iter()
                .any(|c| !matches!(c, Command::EditPathNodes { .. }))
            {
                return Err("node editing cannot be mixed with object creation, replacement, styling or transforms".into());
            }
        }
        if method == "get_selected_vector_geometry" {
            return Ok(selected_geometry(
                &self.doc,
                &self.node_scope,
                path_nodes::MAX_ELEMENTS,
            ));
        }
        let values = args
            .get("patches")
            .and_then(Value::as_array)
            .ok_or("patches must be an array")?;
        if values.len() > 256 {
            return Err("too many objects in one edit".into());
        }
        let patches: Vec<path_nodes::PathNodePatch> =
            serde_json::from_value(Value::Array(values.clone())).map_err(|e| e.to_string())?;
        for patch in &patches {
            if !self.node_scope.contains(&patch.shape_id) {
                return Err(format!(
                    "object {} is outside the original selection",
                    patch.shape_id.raw()
                ));
            }
            if self.trust == Trust::Untrusted && !patch.preserve_bounds {
                return Err("AI node edits must preserve the object's page bounds".into());
            }
        }
        let prepared = path_nodes::prepare(&self.doc, &patches).map_err(|e| e.to_string())?;
        let changed = prepared.changed_nodes;
        if changed > 0 {
            self.run(Command::EditPathNodes { patches })?;
        }
        Ok(
            json!({"changed_nodes": changed, "geometry": selected_geometry(&self.doc, &self.node_scope, path_nodes::MAX_ELEMENTS)}),
        )
    }
}

/// Count final changed nodes, rather than suggestions or repeated assignments.
pub(super) fn changed_nodes(
    before: &Document,
    after: &Document,
    scope: &[ShapeId],
) -> Result<usize, String> {
    let mut count = 0;
    for id in scope {
        let (Ok(old), Ok(new)) = (
            path_nodes::geometry(before, *id, true),
            path_nodes::geometry(after, *id, true),
        ) else {
            continue;
        };
        if old.nodes.len() != new.nodes.len() {
            return Err("node topology changed".into());
        }
        for (old, new) in old.nodes.iter().zip(&new.nodes) {
            if old.node_index != new.node_index {
                return Err("node identity changed".into());
            }
            if old.position != new.position
                || old.in_handle != new.in_handle
                || old.out_handle != new.out_handle
            {
                count += 1;
            }
        }
    }
    Ok(count)
}

/// A node edit may change only selected paths. Check after the same text-flow
/// refresh used by Engine so dependency side effects cannot alter other objects.
pub(super) fn validate_result(before: &Document, host: &HostInner) -> Result<(), String> {
    let changed: std::collections::BTreeSet<_> = host
        .node_scope
        .iter()
        .copied()
        .filter(|id| {
            before.find_shape(*id).map(|s| &s.kind) != host.doc.find_shape(*id).map(|s| &s.kind)
        })
        .collect();
    validate_dependencies(before, &changed)?;
    let mut checked = host.doc.clone();
    checked.refresh_text_flow();
    for id in &host.node_scope {
        let original = before
            .find_shape(*id)
            .ok_or("selected object no longer exists")?;
        let current = checked
            .find_shape_mut(*id)
            .ok_or("selected object was removed")?;
        if let (ShapeKind::Path { path: old, .. }, ShapeKind::Path { path: new, .. }) =
            (&original.kind, &mut current.kind)
        {
            *new = old.clone();
        }
    }
    if checked != *before {
        return Err("node edit would alter object properties, topology or a dependent object; nothing was changed".into());
    }
    Ok(())
}

/// Some native objects regenerate after a frame. Reject these edits before
/// commit rather than letting a later refresh silently alter other geometry.
fn validate_dependencies(
    doc: &Document,
    changed: &std::collections::BTreeSet<ShapeId>,
) -> Result<(), String> {
    if changed.is_empty() {
        return Ok(());
    }
    let mut affected = changed.clone();
    for id in changed {
        let mut containers = doc.ancestors(*id);
        affected.extend(containers.iter().copied());
        containers.push(*id);
        for ancestor in containers {
            let shape = doc
                .find_shape(ancestor)
                .ok_or("selected object no longer exists")?;
            if shape.raster_fx.is_some()
                || crate::connectors::Connection::of(shape).is_some()
                || crate::callouts::Callout::of(shape).is_some()
                || crate::dimensions::Dimension::of(shape).is_some()
            {
                return Err("this path belongs to a generated connector, callout, dimension or raster effect; coordinate-only repair cannot preserve its dependencies".into());
            }
            if let ShapeKind::ClipFrame { frame, .. } = &shape.kind {
                if affected.contains(&frame.id) {
                    return Err("editing a clipping boundary would change other objects; nothing was changed".into());
                }
            }
        }
    }
    fn walk(shape: &tracedraw_core::Shape, affected: &std::collections::BTreeSet<ShapeId>) -> bool {
        if crate::connectors::Connection::of(shape)
            .is_some_and(|c| affected.contains(&c.from) || affected.contains(&c.to))
        {
            return true;
        }
        if shape.effects.iter().any(|e| matches!(e, tracedraw_core::live::Effect::Blend { end_id: Some(id), .. } if affected.contains(id))) { return true; }
        match &shape.kind {
            ShapeKind::Group { children } => children.iter().any(|s| walk(s, affected)),
            ShapeKind::ClipFrame { frame, contents } => {
                walk(frame, affected) || contents.iter().any(|s| walk(s, affected))
            }
            _ => false,
        }
    }
    if doc
        .all_layers()
        .flat_map(|l| &l.shapes)
        .any(|s| walk(s, &affected))
    {
        return Err("this path drives a linked blend or connector; repairing it could alter another object, so nothing was changed".into());
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        app::App,
        scripting::{run, run_node_repair, run_untrusted},
    };

    pub(crate) fn node_fixture() -> (App, ShapeId, ShapeId) {
        let mut app = App::headless();
        run(&mut app, "const p = ActiveLayer.CreatePath('M0 0 L100 0 L100 100 L70 100 L70 50 L50 52 L30 50 L30 100 L0 100 Z'); p.Name = 'selected outline'; p.Fill.ApplyUniformFill(Color.RGB(100,160,230)); p.Outline.SetProperties(0.4, Color.RGB(0,0,0)); p.CreateSelection();").expect("path");
        let selected = app.selection[0];
        run(&mut app, "const other = ActiveLayer.CreateEllipse(120, 20, 20, 20); other.Name = 'PRIVATE OTHER OBJECT'; other.CreateSelection();").expect("other");
        let other = app.selection[0];
        app.selection = vec![selected];
        app.engine.mark_saved();
        (app, selected, other)
    }

    // The test service returns a program that reads actual nodes at runtime,
    // detects a small protrusion between level neighbours and edits only it.
    // It deliberately does not hardcode an object ID, node index or new y.
    pub(crate) const REPAIR_SCRIPT: &str = r#"
const geometry = ActiveDocument.GetSelectedVectorGeometry();
let changed = 0;
for (const object of geometry.objects) {
  const edits = [];
  for (const sub of object.subpaths) {
    const nodes = sub.node_indices.map(i => object.nodes.find(n => n.node_index === i));
    for (let i = 1; i + 1 < nodes.length; i++) {
      const a = nodes[i-1], b = nodes[i], c = nodes[i+1];
      if (!b.in_handle && !b.out_handle && a.position.y === c.position.y &&
          b.position.x > Math.min(a.position.x,c.position.x) && b.position.x < Math.max(a.position.x,c.position.x) &&
          b.position.y > a.position.y && b.position.y - a.position.y <= 2) {
        edits.push({node_index:b.node_index,position:{x:b.position.x,y:a.position.y}});
      }
    }
  }
  if (edits.length) {
    const result = ActiveDocument.ApplyPathNodeEdits([{shape_id:object.shape_id,
      geometry_token:object.geometry_token,coordinate_space:'local',edits}]);
    changed += result.changed_nodes;
  }
}
print('Applied nodes: ' + changed);
"#;

    #[test]
    fn registered_bridge_edits_real_geometry_pixels_and_undo_redo() {
        let (mut app, selected, other) = node_fixture();
        let before = app.doc().clone();
        let image_before =
            tracedraw_render::render_page_image(&before, app.page, 72.0).expect("render before");
        let (output, count) =
            run_node_repair(&mut app, REPAIR_SCRIPT, "AI Drawing").expect("script");
        assert_eq!(count, 1);
        assert_eq!(output, ["Applied nodes: 1"]);
        assert_eq!(app.selection, [selected]);
        assert_eq!(app.doc().find_shape(other), before.find_shape(other));
        let after = app.doc().clone();
        let mut expected = before.clone();
        if let ShapeKind::Path { path, .. } =
            &mut expected.find_shape_mut(selected).expect("path").kind
        {
            let mut elements = path.elements().to_vec();
            elements[5] =
                tracedraw_core::geometry::PathEl::LineTo(tracedraw_core::Point::new(50.0, 50.0));
            *path = tracedraw_core::BezPath::from_vec(elements);
        }
        assert_eq!(
            after, expected,
            "all unsupplied geometry, properties, IDs and order"
        );
        let image_after =
            tracedraw_render::render_page_image(&after, app.page, 72.0).expect("render after");
        assert_ne!(
            image_before.data(),
            image_after.data(),
            "the real rendered canvas changes"
        );
        if let Ok(folder) = std::env::var("RUI_NODE_SHOTS") {
            std::fs::create_dir_all(&folder).expect("shots folder");
            image_before
                .save_png(std::path::Path::new(&folder).join("nodes-before.png"))
                .expect("before PNG");
            image_after
                .save_png(std::path::Path::new(&folder).join("nodes-after.png"))
                .expect("after PNG");
        }
        app.undo();
        assert_eq!(app.doc(), &before);
        app.redo();
        assert_eq!(app.doc(), &after);
    }

    #[test]
    fn even_caught_errors_roll_back_all_edits_and_do_not_dirty_the_drawing() {
        let (mut app, _, _) = node_fixture();
        let before = app.doc().clone();
        let revision = app.engine.revision();
        let bad = format!("{REPAIR_SCRIPT}\ntry {{ ActiveDocument.ApplyPathNodeEdits([{{shape_id:999999,geometry_token:'invalid',coordinate_space:'local',edits:[]}}]); }} catch(e) {{ print(e); }}");
        assert!(run_node_repair(&mut app, &bad, "AI Drawing").is_err());
        assert_eq!(app.doc(), &before);
        assert_eq!(app.engine.revision(), revision);
        assert!(!app.engine.is_dirty());
    }

    #[test]
    fn linked_blends_are_rejected_before_a_frame_can_change_other_objects() {
        let (mut app, selected, other) = node_fixture();
        app.selection = vec![other, selected];
        app.blend_selection(3, 0.0, 0.0, 0.0);
        app.refresh_blends();
        app.selection = vec![selected];
        app.engine.mark_saved();
        let before = app.doc().clone();
        let revision = app.engine.revision();
        let error =
            run_node_repair(&mut app, REPAIR_SCRIPT, "AI Drawing").expect_err("linked dependent");
        assert!(error.contains("linked blend"), "{error}");
        app.refresh_blends();
        assert_eq!(app.doc(), &before);
        assert_eq!(app.engine.revision(), revision);
        assert!(!app.engine.is_dirty());
    }

    #[test]
    fn scope_cannot_expand_and_repair_cannot_replace_restyle_or_transform_objects() {
        let (mut app, selected, other) = node_fixture();
        let before = app.doc().clone();
        let src = format!("new Shape({}).CreateSelection(); const g = ActiveDocument.GetSelectedVectorGeometry(); print(g.objects.map(o=>o.shape_id).join(',')); ActiveDocument.ApplyPathNodeEdits([{{shape_id:{},geometry_token:'bad',coordinate_space:'local',edits:[]}}]);", other.raw(), other.raw());
        assert!(run_node_repair(&mut app, &src, "AI Drawing")
            .expect_err("scope")
            .contains("original selection"));
        for operation in [
            format!("new Shape({}).Move(1,0);", selected.raw()),
            format!("new Shape({}).Delete();", selected.raw()),
            format!("new Shape({}).Fill.ApplyNoFill();", selected.raw()),
            "ActiveLayer.CreateEllipse(0,0,1,1);".into(),
        ] {
            let src = format!("{REPAIR_SCRIPT}\ntry {{ {operation} }} catch(e) {{ print(e); }}");
            assert!(run_node_repair(&mut app, &src, "AI Drawing").is_err());
            assert_eq!(app.doc(), &before);
        }
        assert!(run_node_repair(&mut app, "ActiveSelection.Delete();", "AI Drawing").is_err());
        assert_eq!(app.doc(), &before);
    }

    #[test]
    fn repair_keeps_the_original_canvas_page_and_selection() {
        let (mut app, selected, _) = node_fixture();
        let original_page = app.page;
        let size = app.doc().page(original_page).expect("page").size;
        app.engine
            .run(&Command::AddPage { name: None, size })
            .expect("another page");
        let source = format!("ActiveDocument.Pages.Item(2).Activate(); ActiveDocument.ClearSelection();\n{REPAIR_SCRIPT}");
        assert_eq!(
            run_node_repair(&mut app, &source, "AI Drawing")
                .expect("repair")
                .1,
            1
        );
        assert_eq!(app.page, original_page);
        assert_eq!(app.selection, [selected]);
    }

    #[test]
    fn zero_changes_keep_history_and_actual_counts_ignore_repeated_assignments() {
        let (mut app, _, _) = node_fixture();
        let before = app.doc().clone();
        let revision = app.engine.revision();
        assert_eq!(
            run_node_repair(&mut app, "print('nothing to repair');", "AI Drawing")
                .expect("noop")
                .1,
            0
        );
        assert_eq!(app.engine.revision(), revision);
        assert!(!app.engine.is_dirty());
        let twice = format!("{REPAIR_SCRIPT}\n{{ const g=ActiveDocument.GetSelectedVectorGeometry().objects[0]; const n=g.nodes.find(n=>n.node_index===5); ActiveDocument.ApplyPathNodeEdits([{{shape_id:g.shape_id,geometry_token:g.geometry_token,coordinate_space:'local',edits:[{{node_index:n.node_index,position:{{x:50,y:51}}}}]}}]); }}");
        assert_eq!(
            run_untrusted(&mut app, &twice, "AI Drawing")
                .expect("two valid writes")
                .1,
            1
        );
        app.undo();
        assert_eq!(app.doc(), &before);
        let restore = format!("{REPAIR_SCRIPT}\n{{ const g=ActiveDocument.GetSelectedVectorGeometry().objects[0]; ActiveDocument.ApplyPathNodeEdits([{{shape_id:g.shape_id,geometry_token:g.geometry_token,coordinate_space:'local',edits:[{{node_index:5,position:{{x:50,y:52}}}}]}}]); }}");
        let revision = app.engine.revision();
        assert_eq!(
            run_node_repair(&mut app, &restore, "AI Drawing")
                .expect("net noop")
                .1,
            0
        );
        assert_eq!(app.engine.revision(), revision);
        assert_eq!(app.doc(), &before);
    }
}
