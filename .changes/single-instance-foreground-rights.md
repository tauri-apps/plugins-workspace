---
"single-instance": patch
---

On Windows, the second instance now allows the first instance to bring its window to the front before exiting, so focusing a window from the callback no longer gets refused by Windows.
