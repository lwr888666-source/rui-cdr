# Abschrägungen

Eine Abschrägung lässt die Kanten eines Objekts erhaben oder eingraviert
wirken, indem sie eine Seite der Kante beleuchtet und die andere schattiert.
Sie ist ein Live-Effekt, der im Andockfenster [[docker.bevel]] eingestellt
wird; öffnen Sie es mit [[menu.effects > menu.effects.bevel]].

![Das Andockfenster Abschrägung](../../img/effects-bevel.png)

## Ein Objekt abschrägen {#create}

1. Wählen Sie ein gefülltes Objekt aus.
2. Wählen Sie im Andockfenster [[docker.bevel]] den Stil:
   - [[docker.soft_edge]]: Die Kante fällt zum Objekt hin ab, auf einer Seite
     beleuchtet und auf der anderen schattiert;
   - [[docker.emboss]]: Das Objekt scheint aus der Seite herausgeprägt.
3. Stellen Sie [[docker.distance]] (wie weit die Abschrägung ins Objekt
   reicht), [[docker.light_angle]], [[docker.intensity]] des Lichts,
   [[docker.shadow_color]] und [[docker.light_color]] ein.
4. Klicken Sie auf [[docker.apply]].

Um eine Abschrägung von einem anderen Objekt zu kopieren, verwenden Sie
[[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]].
[[menu.object > menu.object.clear_effect]] entfernt sie. Das Andockfenster für
Extrusionen hat ebenfalls eine Abschrägung für die Vorderseite; siehe
[Extrusionen](extrude.md#settings).
