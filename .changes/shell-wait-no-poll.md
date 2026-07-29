---
"shell": patch
---

Replace the polling loop in the child wait thread with a blocking wait on the process itself, removing a 10ms poll per spawned child and the corresponding latency on `Terminated` events.
