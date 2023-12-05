---
"dialog": "patch"
---

Pin the version of `rfd` to 0.12.0. This is a workaround for [PolyMeilex/rfd#152](https://github.com/PolyMeilex/rfd/pull/152), in which rfd took over responsibility for `gtk_init` by running its own event loop thread.
