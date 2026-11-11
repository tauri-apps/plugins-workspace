---
"sql": patch
"sql-js": patch
---

Fix MySQL `BINARY` and `VARBINARY` columns failing to decode with `unsupported datatype: VARBINARY`. This notably affects `VARCHAR` columns using a binary collation such as `utf8mb4_bin`, which MySQL reports as `VARBINARY`. They are now decoded as byte arrays like the other binary types. Also fix a typo (`TINIYBLOB`) that prevented `TINYBLOB` columns from being decoded.
