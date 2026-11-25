---
single-instance: patch
---

On Linux, updated `zbus` to version 6. The `zvariant`, `zvariant_derive` and `zbus_names` crates, which RustSec flags as unmaintained (RUSTSEC-2026-0370, RUSTSEC-2026-0371 and RUSTSEC-2026-0372), are no longer in the plugin's dependency tree.

The callback now runs on a thread of the plugin's own, so it can block, and calling `destroy` from it no longer hangs.
