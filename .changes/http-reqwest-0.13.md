---
"http": major
"http-js": major
---

Updated `reqwest` to 0.13. `tauri_plugin_http::reqwest` re-exports it, so update your own `reqwest` dependency if you use that re-export. Cargo features that `reqwest` 0.13 removed are removed here too:

- `native-tls-alpn`: ALPN is now part of `native-tls`.
- `rustls-tls-manual-roots`, `rustls-tls-webpki-roots` and `rustls-tls-native-roots`: `rustls-tls` now verifies certificates with the platform verifier.
- `trust-dns`: use `hickory-dns`.
- `macos-system-configuration`: use `system-proxy`, which is on by default.

`rustls-tls` uses `ring` as the crypto provider and installs it as the process default only if none is installed yet.
