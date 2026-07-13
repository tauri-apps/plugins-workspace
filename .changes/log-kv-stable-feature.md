---
"log": patch
"log-js": patch
---

Log plugin is using the `kv_unstable` feature from the `log` crate. That feature was stabilized in [log v0.4.21](https://github.com/rust-lang/log/compare/0.4.20...0.4.21). And this workspace is using [v0.4.27](https://github.com/bajoca05/plugins-workspace/blob/v2/Cargo.lock#L3463), no need to keep using it.