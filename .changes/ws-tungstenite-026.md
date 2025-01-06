---
websocket: patch
websocket-js: patch
---

**Breaking change:** Updated tokio_tungstenite to `0.26`. This may be a breaking change if you use the `tls_connector` Builder method in Rust.
**Breaking change:** Removed the accidental `ConnectionConfig` struct re-export (in rust). This should not affect anyone since the Config was not used in any public API.
