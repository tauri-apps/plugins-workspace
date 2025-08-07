---
nfc: patch
nfc-js: patch
---

On iOS, the reader session will now get closed properly on errors, preventing dangling invalid sessions that could prevent subsequent write attempts.
