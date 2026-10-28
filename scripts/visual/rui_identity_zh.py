# The installed name, native title/icon and Chinese menus share the same identity.
from PIL import Image

assert alive(), "Rui must stay running"
windows = [w for w in d.screen().root.query_tree().children
           if w.get_attributes().map_state == X.IsViewable and w.get_geometry().width > 500]
assert windows, "No Rui window"
window = windows[-1]
title = window.get_full_property(d.intern_atom("_NET_WM_NAME"), X.AnyPropertyType)
assert title is not None
title = bytes(title.value).decode("utf-8")
assert title.startswith("Rui - "), title
assert "欢迎" in title, "The Chinese welcome title must be localized: " + title
icon = window.get_full_property(d.intern_atom("_NET_WM_ICON"), X.AnyPropertyType)
assert icon is not None, "The native window icon must be set explicitly"
values = list(icon.value)
expected = Image.open("assets/icon/rui-128.png").convert("RGBA")
assert values[:2] == list(expected.size), "Wrong window icon dimensions"
pixels = [(a << 24) | (r << 16) | (g << 8) | b for r, g, b, a in expected.getdata()]
assert values[2:] == pixels, "The window must use the Rui artwork"
shot("rui_identity_zh")
print("Rui title, native icon and Chinese welcome screen verified")
