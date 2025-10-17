---
"single-instance": patch
---

On macOS, accept and read incoming unix socket streams from a dedicated background thread instead of an async runtime task, so the blocking I/O calls don't stall the async executor.
