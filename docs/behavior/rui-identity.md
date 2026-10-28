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
languages. This distribution starts in Simplified Chinese, including an
empty or missing preference file. Preference upgrade 5 migrates the old
English default to Simplified Chinese once. A language selected after that
upgrade is preserved, including English. Existing choices of other
languages are preserved.
Language can be changed in Options > Global > General, Language, without
restarting. The installer language does not control the interface language.
The visual check starts with an actual version-4 English preference file
and checks that the launched application displays Chinese.

Checks cover the native title and exact icon pixels, Chinese welcome text,
the cached title, the installer desktop shortcut's name and target, and
the installed executable's product name. CI also captures a Chinese
welcome screenshot under Xvfb and runs the editing and random-action
smoke scripts. Package installation and SVG-to-PDF conversion checks
continue to gate publication.
