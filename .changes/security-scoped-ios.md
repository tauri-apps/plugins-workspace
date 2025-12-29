---
"fs": patch
"fs-js": patch
---

Enable access for security-scoped resources on iOS by automatically calling `NSURL::startAccessingSecurityScopedResource` on resource access and adding the `stopAccessingSecurityScopedResource` API.
