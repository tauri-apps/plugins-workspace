---
"geolocation": "patch"
"geolocation-js": "patch"
---

On Android, `getCurrentPosition` now honors the `timeout` option and rejects when no position is found in time. A `timeout` of `0` means no timeout.
