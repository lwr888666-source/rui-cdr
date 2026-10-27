# Bevels

A bevel makes the edges of an object look raised or carved, by lighting one
side of the edge and shading the other. It is a live effect, set in the
[[docker.bevel]] panel; open it with [[menu.effects > menu.effects.bevel]].

![The Bevel panel](../../img/effects-bevel.png)

## To bevel an object {#create}

1. Select a filled object.
2. In the [[docker.bevel]] panel, choose the style:
   - [[docker.soft_edge]]: the edge slopes into the object, lit on one side
     and shaded on the other;
   - [[docker.emboss]]: the object seems pressed out of the page.
3. Set the [[docker.distance]] (how far the bevel reaches into the object),
   the [[docker.light_angle]], the [[docker.intensity]] of the light, the
   [[docker.shadow_color]] and the [[docker.light_color]].
4. Click [[docker.apply]].

To copy a bevel from another object, use
[[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]].
[[menu.object > menu.object.clear_effect]] removes it. The extrusion panel
also has a bevel for the front face; see [Extrusions](extrude.md#settings).
