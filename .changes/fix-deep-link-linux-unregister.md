---
deep-link: patch
---

Fixed `unregister` on Linux leaving the app as the scheme's handler: it now also removes the scheme from the `MimeType` of the handler's `.desktop` file and refreshes the desktop database, which `xdg-mime` falls back to. It also no longer fails when `mimeapps.list` does not exist.
