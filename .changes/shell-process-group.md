---
"shell": minor:feat
"shell-js": minor
---

Add `processGroup` option to spawn commands in a new process group (POSIX) or job object (Windows), allowing the entire process tree to be killed when calling `kill()` on the child process.
