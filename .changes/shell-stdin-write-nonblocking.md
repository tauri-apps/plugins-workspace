---
"shell": minor
"shell-js": minor
---

`Child.write` no longer blocks the main thread while the child isn't reading its stdin. The data is written by a background thread per child, in call order, and the promise resolves once it was written (or rejects with the write error, for example when the child exited).