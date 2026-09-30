---
"single-instance": patch
---

Return plugin initialization errors when the Linux session bus cannot be reached or a secondary instance cannot deliver its arguments to the primary instance, instead of silently starting without ownership or exiting successfully after failed delivery.
