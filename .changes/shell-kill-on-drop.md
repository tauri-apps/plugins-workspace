---
"shell": minor:feat
---

Add `Command::kill_on_drop` to kill the spawned process (or its process group / job object when `Command::set_process_group` is enabled) when its `CommandChild` is dropped.
