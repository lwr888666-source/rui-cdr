# Rui name, icon and languages

The application name is Rui. Desktop shortcuts, window titles, dialogs,
downloads, the browser loader and platform package metadata use Rui.
The executable names are `rui` and `rui-cli`. Existing `.tdraw` documents
and internal library crate identifiers remain compatible.

The native viewport explicitly uses `assets/icon/rui-128.png`, the same
project-provided artwork shown in the application. Windows executable
resources and the installer use `assets/icon/rui.ico`. Setting only the
executable's resource icon does not set the native viewport's icon.
The viewport icon is decoded once at startup; changing a document title
still sends a window-title command only when the title changes.

Windows installation offers English and Simplified Chinese. The Chinese
installer messages and their license are bundled in
`packaging/windows/languages/` so a runner does not need an extra language
download. An update removes the old application shortcut names and creates
Rui shortcuts. Existing preference files and drawings are preserved.

The application offers Simplified Chinese and seventeen other interface
languages. A saved language choice wins. With no saved choice, the system
locale is used; Windows desktop launches read the Windows user locale if
`LC_ALL` and `LANG` are absent. Chinese Windows installations therefore use
the Chinese interface without requiring a locale environment variable.
Language can be changed on the Welcome Screen or in Options > Global >
General, Language, without restarting.

Checks cover the native title and exact icon pixels, Chinese welcome text,
the cached title, the installer desktop shortcut's name and target, and
the installed executable's product name. CI also captures a Chinese
welcome screenshot under Xvfb and runs the editing and random-action
smoke scripts. Package installation and SVG-to-PDF conversion checks
continue to gate publication.
