# Smussi

Uno smusso fa sembrare i bordi di un oggetto in rilievo o scolpiti,
illuminando un lato del bordo e ombreggiando l'altro. È un effetto dinamico,
impostato nel pannello [[docker.bevel]]; aprilo con
[[menu.effects > menu.effects.bevel]].

![Il pannello Smusso](../../img/effects-bevel.png)

## Per smussare un oggetto {#create}

1. Seleziona un oggetto con riempimento.
2. Nel pannello [[docker.bevel]], scegli lo stile:
   - [[docker.soft_edge]]: il bordo scende in pendenza verso l'interno
     dell'oggetto, illuminato da un lato e ombreggiato dall'altro;
   - [[docker.emboss]]: l'oggetto sembra sbalzato fuori dalla pagina.
3. Imposta la [[docker.distance]] (fin dove lo smusso arriva dentro
   l'oggetto), l'[[docker.light_angle]], l'[[docker.intensity]] della luce, il
   [[docker.shadow_color]] e il [[docker.light_color]].
4. Fai clic su [[docker.apply]].

Per copiare uno smusso da un altro oggetto, usa
[[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]].
[[menu.object > menu.object.clear_effect]] lo rimuove. Anche il pannello
dell'estrusione ha uno smusso per la faccia anteriore; vedi
[Estrusioni](extrude.md#settings).
