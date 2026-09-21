---
"fs": major
"fs-js": major
---

**Breaking:** The `notify` crate's `serialization-compat-6` feature is no longer enabled, so `WatchEvent` now follows notify v8's format: the event kind is flattened into the event, with `type` holding the top-level kind (`any`, `access`, `create`, `modify`, `remove` or `other`) and `kind`/`mode` refining it. For instance `{ type: { modify: { kind: 'data', mode: 'content' } } }` is now `{ type: 'modify', kind: 'data', mode: 'content' }`, and `attrs.flag` is `rescan` instead of `Rescan`. `WatchEvent.attrs` is now typed as `WatchEventAttributes`.
