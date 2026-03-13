---
"notification": minor
"notification-js": minor
---

Improve notification actions across Rust and Android by adding Rust support for defining/registering action types and actions, and by fixing Android action-group storage consistency.

Extend reliability and API consistency by adding listener-ready queue/replay handling for cold-start action events and making `onAction` payloads consistent for both immediate delivery and queued replay on Android.
