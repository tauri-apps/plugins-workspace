---
nfc: minor
---

Fix `Nfc::scan` failing to deserialize every scanned tag in Rust. `NfcTag::id` is now `Vec<u8>` (empty when the OS reports no identifier) and `NfcTag::kind` is now `Vec<String>`, matching what Android and iOS send and the JavaScript `Tag` type. Android now reports tag and record bytes as unsigned values (0-255), as iOS already does.
