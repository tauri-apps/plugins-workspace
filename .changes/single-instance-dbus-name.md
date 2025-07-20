---
"single-instance": patch
---

Fix D-Bus name replacement logic on Linux to prevent multiple instances from acquiring the same well-known name.  
This patch updates the `zbus` dependency to the latest compatible version (`^5.9`) and explicitly sets `RequestNameFlags` to ensure a second instance fails to acquire the D-Bus name when another one is already running.