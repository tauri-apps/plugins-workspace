---
"single-instance": patch
---

On Windows, a second instance no longer waits indefinitely for a first instance that stopped processing messages: it stops waiting after 10 seconds and exits. The first instance may still receive the arguments once it resumes processing messages.
