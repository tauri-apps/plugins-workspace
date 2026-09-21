---
"deep-link": major
---

**Breaking:** On Linux, failing to run `xdg-mime` or `update-desktop-database` in `register`, `unregister` and `is_registered` now returns the new `Error::Execute(command, io_error)` variant instead of logging the failure and returning the raw `Error::Io`.
