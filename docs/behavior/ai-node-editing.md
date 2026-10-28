# Selected path node editing

AI Generate > Draw with words can inspect and patch the selected native paths.
A request mentioning nodes, anchors or control handles enters a restricted edit
mode before the service's program runs. The standard JavaScript object model
also exposes the same bridge to user scripts. No new network API is used.

## Read

`ActiveDocument.GetSelectedVectorGeometry()` returns `objects` and `unsupported`.
Only the selection captured at evaluation start is read. Changing script
selection does not expand this scope. Each supported object contains:

- `shape_id`, `object_type`, `selected`, `units`, `closed`, `geometry_token`.
- `nodes`: `node_index`, `subpath_index`, `position`, `in_handle`, `out_handle`,
  `page_position`, `page_in_handle`, `page_out_handle`, `node_type`.
- `subpaths`: index, closure flag, canonical node indices and exact segment
  records (`element_index`, `kind`, `from`, `to`, `controls`).
- `local_to_page` and `page_to_local`, including every parent transform.

Coordinates are absolute millimetres. Page origin is bottom-left, x right and
y up. An affine `[a,b,c,d,e,f]` maps `(x,y)` to
`(a*x+c*y+e,b*x+d*y+f)`. Handles are absolute points, not offsets.
Indices are zero-based original `BezPath` element indices, not dense ordinals.
The repeated closing endpoint is an alias of the start node. Subpath indices
are zero-based. Indices stay valid while the geometry token stays current.

Native paths include open/closed subpaths, lines, quadratics and cubics.
Rectangles, ellipses, groups, text, bitmaps, symbols and B-spline outlines report
their unsupported reason without conversion or replacement. Select a native
path inside a group to edit it. Locked objects or containers cannot be changed.

## Patch

```javascript
const read = ActiveDocument.GetSelectedVectorGeometry();
// Choose an actual node from read.objects, using the request and its geometry.
const object = read.objects[0];
const result = ActiveDocument.ApplyPathNodeEdits([{
  shape_id: object.shape_id,
  geometry_token: object.geometry_token,
  coordinate_space: 'local', // or 'page'
  preserve_bounds: true,
  edits: [{node_index: 5, position: {x: 50, y: 50}}]
}]);
print(result.changed_nodes);
```

The example index and position apply only to the regression fixture. Production
programs must choose them from the returned geometry. Omitting `position`,
`in_handle` or `out_handle` leaves that coordinate exactly unchanged. Moving an
anchor never moves its handles implicitly. Adjacent segments necessarily change
when their endpoint or control is explicitly edited; unrelated segments retain
their original coordinates and degree. A quadratic's single control has two
aliases: both endpoint handles must be explicitly supplied with the same value.
The start node's two physical closing endpoints move together.

The result is `{changed_nodes, geometry}` with fresh tokens. Core validation
prepares the whole batch before applying `Command::EditPathNodes`. No SVG text
round trip, path fitting, node insertion/deletion or replacement object occurs.
Styles, IDs, node type overrides, transforms, layers, object order, references
and closure flags remain unchanged. The script records native commands and
commits once through `Engine::run_batch`; Undo/Redo restores the whole edit.

## Validation and failure

Untrusted edits target only the original selection and must preserve page-space
bounds within `1e-9 * max(width,height,1)` mm. User scripts can explicitly set
`preserve_bounds:false`. If fixing an extremum would change the overall size,
the AI edit is rejected instead of moving other nodes or rescaling the path.
This interface does not attempt automatic global simplification or curve fitting.

Tokens cover the ID, exact path coordinates, closure and composite transform.
Missing objects, stale tokens, duplicate objects/nodes, absent handles,
non-finite coordinates, absolute coordinates beyond 1,000,000,000 mm,
noninvertible transforms, invalid paths, more than 100,000 path elements,
256 objects or 4,096 edits per object are rejected. One AI request embeds at
most 2,048 selected nodes; larger paths report the omission and can be inspected
through the script bridge. No unrelated document geometry is sent for a selection.

Node repair refuses object creation, replacement, styles and transforms, even
when the generated script catches the error. Any script or validation failure
rolls back the complete script. A final document comparison, after the engine's
text flow refresh, rejects unintended changes to dependent objects. Generated
connector, callout, dimension and raster-effect paths, clipping boundaries,
and paths driving linked blends/connectors are refused before a later frame
can regenerate them or alter related objects. Changes to
the engine, document revision, active page or selection while waiting invalidate
the reply. The existing single retry can ask the service for a corrected program.

The UI counts unique nodes whose final coordinates actually differ, not model
suggestions or repeated assignments. A print-only or net-zero program reports
no changes and creates no undo step. A partially invalid program reports failure
and applies zero changes. Model inference still depends on the configured AI
service: the bridge does not guess which node the user wants changed.

## Checks

Core tests in `path_nodes::tests` cover exact mixed-path reads, handles, closure,
quadratics, parent transforms, ranges, tokens, locks, atomic batches and history.
`scripting::path_nodes::tests` runs real Boa programs through the registered
bridge and compares the whole document and rendered pixels before/after.
`ai::tests::natural_language_node_repair_reads_decides_edits_and_reports_actual_changes`
uses the real prompt/request/parser/Boa/command chain with a deterministic test
transport. Its returned script detects the fixture's protrusion from actual
neighbour coordinates without hardcoded IDs or target indices. This test does
not claim to validate a third party model's visual judgement.
