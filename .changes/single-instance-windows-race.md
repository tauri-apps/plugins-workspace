---
single-instance: patch
---

Fix a race on Windows that could let a second app instance run unguarded: the first instance creates the single-instance mutex before its event target window, and a second launch inside that gap saw the mutex but no window and silently continued. The second instance now waits (up to 5 seconds) for either the primary's window to appear (then forwards its args and exits as usual) or for the mutex to be released or abandoned (then it takes over as the primary, which also makes updater-style respawns deterministic instead of timing-dependent).
