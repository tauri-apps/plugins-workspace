---
"shell": patch
"shell-js": patch
---

A child spawned from JavaScript with `processGroup` now stays tracked after its leader exits while the rest of its process group (Unix) or job object (Windows) is still running, until the group is empty. Before, `kill()` resolved without doing anything in that case, and the remaining processes also survived the app exit.
