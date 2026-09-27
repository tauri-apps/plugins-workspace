---
"single-instance": patch
---

On Windows, a second instance no longer waits indefinitely for a first instance that stopped processing messages: it waits for the first instance to take its arguments for up to 10 seconds, then exits.
