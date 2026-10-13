---
"geolocation": "patch"
"geolocation-js": "patch"
---

Fixed `watchPosition` on Android stopping updates for every other active watch, and `clearWatch` leaving location updates running until the last watch was cleared.
