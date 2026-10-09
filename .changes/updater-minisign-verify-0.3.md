---
"updater": patch
"updater-js": patch
---

Update `minisign-verify` to 0.3, which rejects signatures with a non-canonical Ed25519 `S` value. The `Error::Minisign` variant now wraps `minisign_verify` 0.3's `Error`, and its message reads like "The signature verification failed" instead of the variant name (`InvalidSignature`).
