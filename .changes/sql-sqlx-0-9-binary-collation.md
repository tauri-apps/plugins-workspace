---
"sql": minor
"sql-js": minor
---

Update `sqlx` to 0.9 and decode MySQL text columns that use a binary collation (such as `VARCHAR ... COLLATE utf8mb4_bin`) as strings. These columns were reported as `VARBINARY`/`BINARY`/`BLOB` and failed with `unsupported datatype: VARBINARY` or came back as byte arrays. Columns with real binary data (`BINARY`, `VARBINARY`, `BLOB` and friends) are now decoded as byte arrays. The plugin now requires Rust 1.94, the minimum supported by `sqlx` 0.9.
