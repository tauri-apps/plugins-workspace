---
"single-instance": patch
---

On Linux, a secondary instance that cannot deliver its arguments to the primary instance now returns a plugin initialization error instead of exiting successfully, and an invalid `dbus_id` returns an error instead of panicking. When the session bus is unavailable the app keeps launching normally, without single-instance support, and logs a warning.
