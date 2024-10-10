---
"store": minor:breaking
---

**Breaking changes**:

- Renamed `StoreCollection` to `StoreState`
- `StoreBuilder::build` now returns a `Result`
- `StoreExt::store` now returns `Result<Arc<Store>>`

Other Changes:

- Save and cancel pending auto save on drop
- Use absolute path as store's key, fix #984
- Share store to resource table by default
- Enable auto save with 100ms debounce time by default
- Use pretty json by default, close #1690

New features:

- Add `getStore`/`get_store` share stores across js and rust side
- Add default (de)serialize functions settings
- Allow js to use pre-stored (de)serialize functions
- Add back lazy store (implemented in js)
