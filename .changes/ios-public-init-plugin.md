---
barcode-scanner: patch
biometric: patch
clipboard-manager: patch
dialog: patch
geolocation: patch
haptics: patch
log: patch
nfc: patch
notification: patch
opener: patch
shell: patch
---

On iOS, mark the plugin's `init_plugin_*` entry point `public` so release builds made with Xcode 27 keep it global and link.
