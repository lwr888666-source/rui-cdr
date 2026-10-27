# Panel Barva

Panel [[docker.color]] (<kbd>Shift+F11</kbd>) namíchá barvu a použije ji na
výplň nebo obrys výběru. Otevře jej také nástroj [[tool.outline_color]]
(<kbd>Shift+F12</kbd>). Otevřete jej z nabídky
[[menu.window > menu.window.dockers]].

![Panel Barva](../../img/color-color-docker.png)

## Výběr barvy {#choose}

1. Vyberte objekty. Panel začne barvou výplně prvního z nich.
2. Novou barvu namíchejte v jednom ze zobrazení:
   - [[docker.color_viewers]]: kliknutím nebo tažením ve čtverci nastavíte
     sytost (zleva doprava) a jas (zdola nahoru) a v pruhu vedle něj odstín;
   - [[docker.color_sliders]]: jeden posuvník pro každou složku nastavení
     [[docker.model]]: RGB, CMYK (výchozí), HSB, HSL, Lab, stupně šedi nebo
     YIQ;
   - [[docker.color_palettes]]: vzorky barev palety.

   Tři tlačítka nahoře zobrazují nebo skrývají jednotlivá zobrazení; zobrazit
   lze libovolnou kombinaci. Můžete také zadat hodnotu `#rrggbb`.
3. Kliknutím na [[docker.fill]] nebo [[docker.outline]] dole v panelu novou
   barvu použijete. [[docker.add_to_palette]] ji přidá do palety dokumentu.

Stará a nová barva se zobrazují vedle sebe. Pokud novou barvu nelze přesně
vytisknout barvami CMYK, panel zobrazí upozornění.

## Nabrání barvy {#sample}

Klikněte na [[docker.eyedropper]] a pak kamkoli do kresby a naberete barvu z
tohoto místa. Poté kliknutím na [[docker.use_sampled]] z ní uděláte novou
barvu. Viz také [Kapátka](eyedroppers.md).

## Přímé barvy {#spot-inks}

Pod tlačítky [[docker.spot_inks]] uvádí přímé barvy kresby; viz
[Přímé barvy](spot-colors.md).
