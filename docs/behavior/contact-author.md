# Contact author and feedback

The standard toolbar has a red outline heart immediately after AI Generate.
It is available on the Welcome Screen and while editing. Clicking it opens
Contact author / Feedback with the original contact card supplied by the
author. Scan its QR code with WeChat to contact the author or report a problem.
The card keeps its full aspect ratio and scales to the available window.
The title, hint, tooltip and Close button follow the interface language.

Close the dialog with Close, Enter or Escape. There is no default shortcut.
Opening it runs `Command::ContactAuthor` through the engine without changing
the document, its revision, undo history or saved state. It sends no request
and does not open an external service. The embedded image is decoded and
uploaded once, only when the dialog is first opened.

Existing customized standard toolbars gain the heart after AI Generate
once on upgrade. Users can then move or remove it using toolbar customization.
Resetting the toolbar restores the default position.

Tests cover the original image dimensions, texture reuse, unchanged drawing
and revision, toolbar migration, and the dialog's close behavior. The Linux
visual check opens it from the Chinese toolbar and captures the displayed card.
