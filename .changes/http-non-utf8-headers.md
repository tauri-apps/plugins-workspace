---
"http": patch
"http-js": patch
---

Fix `fetch` throwing a `TypeError` when a response header value contains non-ASCII bytes (e.g. a UTF-8 encoded `Content-Disposition` filename). Header values are now treated as byte strings like in a browser: response header values are decoded as ISO-8859-1 so the raw bytes are preserved as a valid `ByteString` for `new Headers()`, and request header values are sent as their raw bytes (one byte per character) instead of being UTF-8 encoded. A request header value with a character such as `é` is now sent as the single byte `0xE9`, as browsers do.
