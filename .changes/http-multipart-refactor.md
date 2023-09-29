---
"http-js": minor
---

Multipart requests are now handled in JavaScript by the `Request` JavaScript class so you just need to use a `FormData` body and not set the content-type header to `multipart/form-data`. `application/x-www-form-urlencoded` requests must be done manually.
