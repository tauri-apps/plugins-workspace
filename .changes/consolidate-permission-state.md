---
"barcode-scanner": patch
"barcode-scanner-js": patch
"geolocation": patch
"geolocation-js": patch
"notification": patch
"notification-js": patch
---

Use `PermissionState` from the `tauri` crate, which now also includes a "prompt with rationale" variant for Android (returned when your app must explain to the user why it needs the permission).
