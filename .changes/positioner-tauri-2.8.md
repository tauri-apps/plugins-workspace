---
positioner: patch
positioner-js: patch
---

`readFile` now returns a more specific type `Promise<Uint8Array<ArrayBuffer>>` instead of the default `Promise<Uint8Array<ArrayBufferLike>`
