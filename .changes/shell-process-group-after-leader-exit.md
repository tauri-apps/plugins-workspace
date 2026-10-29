---
"shell": patch
"shell-js": patch
---

On Unix, a child spawned from JavaScript with `processGroup` stays tracked after its leader exits while the rest of its process group is still running. Before, `kill()` answered successfully without doing anything in that case, and the remaining processes also survived the app exit.
