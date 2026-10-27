# Fazowanie

Fazowanie sprawia, że krawędzie obiektu wyglądają na wypukłe lub wyrzeźbione,
przez oświetlenie jednej strony krawędzi i zacienienie drugiej. Jest to efekt
dynamiczny, ustawiany w panelu [[docker.bevel]]; otwórz go poleceniem
[[menu.effects > menu.effects.bevel]].

![Panel Fazowanie](../../img/effects-bevel.png)

## Fazowanie obiektu {#create}

1. Zaznacz wypełniony obiekt.
2. W panelu [[docker.bevel]] wybierz styl:
   - [[docker.soft_edge]]: krawędź opada ku wnętrzu obiektu, oświetlona z jednej
     strony i zacieniona z drugiej;
   - [[docker.emboss]]: obiekt wydaje się wytłoczony ze strony.
3. Ustaw wartości [[docker.distance]] (jak daleko fazowanie sięga w głąb
   obiektu), [[docker.light_angle]], [[docker.intensity]] światła,
   [[docker.shadow_color]] i [[docker.light_color]].
4. Kliknij [[docker.apply]].

Aby skopiować fazowanie z innego obiektu, użyj
[[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]].
[[menu.object > menu.object.clear_effect]] usuwa je. Panel wytłaczania ma też
fazowanie przedniej ściany; zobacz [Wytłaczanie](extrude.md#settings).
