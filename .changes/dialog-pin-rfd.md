---
"dialog": "patch"
---

Pin the version of `rfd` to 0.12.0.

This is a workaround for [PolyMeilex/rfd#152](https://github.com/PolyMeilex/rfd/pull/152), in which rfd took over responsibility for `gtk_init` by running its own event loop thread. As described in that pull request:

> Yeah, this is a decent solution, not much else we can do about this global state bs on C side.
>
> This will obviously blow up as soon as someone has other code that also uses GTK, but let's ignore that for now, as I want to get rid of GTK backend one day anyway (#66).

Yes, Tauri is other code that uses GTK and it does, indeed, blow up. Tauri already worked around this issue by using the synchronous dialog API on GTK and carefully running it in the main event loop thread.

The best way around this is to stop using rfd entirely (they're planning to drop the GTK backend, which will probably break since the XDG Portal API doesn't cover message dialogs).

