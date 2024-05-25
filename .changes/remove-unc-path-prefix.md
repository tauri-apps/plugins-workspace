---
dialog: patch
fs: patch
store: patch
---

**Breaking Change:** All apis that return paths to the frontend will now remove the `\\?\` UNC prefix on Windows.
