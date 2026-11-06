---
single-instance: patch
---

On Linux, updated `zbus` to version 6. The `zvariant`, `zvariant_derive` and `zbus_names` crates, which RustSec flags as unmaintained (RUSTSEC-2026-0370, RUSTSEC-2026-0371 and RUSTSEC-2026-0372), are no longer in the plugin's dependency tree. The D-Bus connection of the first instance now runs on a thread of its own named `single-instance-dbus`, which is also the thread your callback runs on. Calling `destroy` from inside the callback no longer hangs.
