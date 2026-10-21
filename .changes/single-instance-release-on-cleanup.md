---
"single-instance": patch
---

Release the single instance lock in `AppHandle::cleanup_before_exit`, right before the process exits or restarts, instead of on `RunEvent::Exit`. The app now remains the single instance while your own `RunEvent::Exit` handler runs. `destroy` can still release the lock earlier.
