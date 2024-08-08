---
"shell": patch
---

Change the `execute` scope argument validator regex to match on the entire string by default.
If this behavior is not desired check the `raw` boolean configuration option that is available along the `validator` string.
