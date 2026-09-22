---
haptics: patch
geolocation: patch
---

Fixed a build failure when the `specta` feature is enabled: the `Error` type used the `#[serde(skip)]` helper attribute, which recent `specta` versions no longer register. It now uses `#[specta(skip)]`.
