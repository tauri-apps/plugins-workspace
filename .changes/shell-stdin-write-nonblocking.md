---
"shell": patch
"shell-js": patch
---

`Child.write` no longer blocks the main thread while the child isn't reading its stdin. The data is queued and written by a background thread per child, in call order. If a write fails (for example because the child exited), the error is returned by the next `write` call.
