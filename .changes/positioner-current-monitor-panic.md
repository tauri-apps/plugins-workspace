---
positioner: patch
positioner-js: patch
---

Replaced a panic in `calculate_position` with an error return when the window has no associated monitor (e.g. during display sleep or monitor reconfiguration).
