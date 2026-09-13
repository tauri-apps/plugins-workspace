---
"http": patch
"http-js": patch
---

Fix `fetch` throwing a `TypeError` when a response header value contains non-ASCII bytes (e.g. a UTF-8 encoded `Content-Disposition` filename). Header values are now decoded as ISO-8859-1 instead of UTF-8 so the raw bytes are preserved as a valid `ByteString` for `new Headers()`, matching what a browser exposes for the same response.
