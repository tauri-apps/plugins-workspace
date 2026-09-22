---
"upload": major
"upload-js": major
---

**Breaking:** `upload` and `download` now take an `options` object as their last argument instead of positional `headers`, `method` and `body` arguments:

```ts
await upload(url, filePath, onProgress, { headers, method: HttpMethod.Put })
await download(url, filePath, onProgress, { headers, body })
```

`headers` accepts both a `Map` and a plain object. Previously a `Map` was silently serialized as an empty object.
