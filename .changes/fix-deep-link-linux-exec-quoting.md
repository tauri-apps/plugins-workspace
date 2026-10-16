---
"deep-link": patch
"deep-link-js": patch
---

On Linux, `register` now quotes the executable path in the generated `.desktop` file's `Exec` key only when the path needs it (e.g. it contains a space). Deep links now open the app when `xdg-open` runs without a detected desktop environment, which could not launch a quoted path, while paths with spaces keep working.
