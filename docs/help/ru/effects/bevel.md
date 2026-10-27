# Скосы

Скос делает края объекта рельефными или вырезанными: одна сторона края
освещается, другая затеняется. Это живой эффект, который задаётся на панели
[[docker.bevel]]; откройте её через [[menu.effects > menu.effects.bevel]].

![Панель «Скос»](../../img/effects-bevel.png)

## Создание скоса {#create}

1. Выделите объект с заливкой.
2. На панели [[docker.bevel]] выберите стиль:
   - [[docker.soft_edge]]: край плавно уходит в объект, с одной стороны он
     освещён, с другой затенён;
   - [[docker.emboss]]: объект кажется выдавленным из страницы.
3. Задайте [[docker.distance]] (насколько глубоко скос заходит в объект),
   [[docker.light_angle]], [[docker.intensity]] света, [[docker.shadow_color]]
   и [[docker.light_color]].
4. Нажмите [[docker.apply]].

Чтобы скопировать скос с другого объекта, используйте
[[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]].
[[menu.object > menu.object.clear_effect]] удаляет его. У панели выдавливания
тоже есть скос для лицевой грани; см. [Выдавливание](extrude.md#settings).
