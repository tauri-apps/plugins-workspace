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
barcode-scanner-js: patch
biometric-js: patch
clipboard-manager-js: patch
dialog-js: patch
geolocation-js: patch
haptics-js: patch
log-js: patch
nfc-js: patch
notification-js: patch
opener-js: patch
shell-js: patch
---

On iOS, mark the plugin's `init_plugin_*` entry point `public` so release builds made with Xcode 27 keep it global and link.
