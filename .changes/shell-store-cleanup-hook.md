---
"shell": patch
"shell-js": patch
"store": patch
"store-js": patch
---

The shell plugin now kills the child processes spawned from JavaScript, and the store plugin now saves its stores, in `on_cleanup_before_exit` instead of on `RunEvent::Exit`. So this also happens when the app restarts on the main thread with `AppHandle::restart`, which skips `RunEvent::Exit`. Before, those children kept running and unsaved store values were lost on such a restart.
