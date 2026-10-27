# Biseles

Un bisel hace que los bordes de un objeto parezcan en relieve o tallados,
iluminando un lado del borde y sombreando el otro. Es un efecto dinámico, que
se ajusta en el panel [[docker.bevel]]; ábralo con
[[menu.effects > menu.effects.bevel]].

![El panel Bisel](../../img/effects-bevel.png)

## Para biselar un objeto {#create}

1. Seleccione un objeto con relleno.
2. En el panel [[docker.bevel]], elija el estilo:
   - [[docker.soft_edge]]: el borde desciende hacia el interior del objeto,
     iluminado por un lado y sombreado por el otro;
   - [[docker.emboss]]: el objeto parece estampado en relieve sobre la
     página.
3. Ajuste la [[docker.distance]] (hasta dónde llega el bisel dentro del
   objeto), el [[docker.light_angle]], la [[docker.intensity]] de la luz, el
   [[docker.shadow_color]] y el [[docker.light_color]].
4. Haga clic en [[docker.apply]].

Para copiar un bisel de otro objeto, use
[[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]].
[[menu.object > menu.object.clear_effect]] lo quita. El panel de extrusión
también tiene un bisel para la cara frontal; vea
[Extrusiones](extrude.md#settings).
