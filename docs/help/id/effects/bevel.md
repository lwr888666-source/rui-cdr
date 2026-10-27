# Bevel

Bevel membuat tepi objek terlihat timbul atau terukir, dengan menerangi satu
sisi tepi dan membayangi sisi lainnya. Ini adalah efek langsung, diatur di
panel [[docker.bevel]]; buka dengan [[menu.effects > menu.effects.bevel]].

![Panel Bevel](../../img/effects-bevel.png)

## Untuk memberi bevel pada objek {#create}

1. Pilih objek yang berisi isian.
2. Di panel [[docker.bevel]], pilih gayanya:
   - [[docker.soft_edge]]: tepi melandai ke dalam objek, terang di satu sisi
     dan berbayang di sisi lain;
   - [[docker.emboss]]: objek tampak menonjol keluar dari halaman.
3. Atur [[docker.distance]] (seberapa jauh bevel masuk ke dalam objek),
   [[docker.light_angle]], [[docker.intensity]] cahaya,
   [[docker.shadow_color]], dan [[docker.light_color]].
4. Klik [[docker.apply]].

Untuk menyalin bevel dari objek lain, gunakan
[[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]].
[[menu.object > menu.object.clear_effect]] menghapusnya. Panel ekstrusi juga
memiliki bevel untuk sisi depan; lihat [Ekstrusi](extrude.md#settings).
