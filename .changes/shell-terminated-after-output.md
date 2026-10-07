---
shell: patch
shell-js: patch
---

Fix `Command::spawn` (and the JavaScript `Command` API) sometimes emitting the `Terminated` (`close`) event before the child's `stdout`/`stderr` output when the process exits quickly.
