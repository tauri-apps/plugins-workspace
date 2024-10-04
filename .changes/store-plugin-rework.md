---
"store": minor:breaking
---

**Breaking change**: Renamed `StoreCollection` to `StoreState`

Changes:

- Disallow calling `create_store` when there're still active stores with the same path alive
- Save and cancel pending auto save on drop
- Use absolute path as store's key, fix #984

New features:

- Add `getStore`/`get_store` share stores across js and rust side
- Add default (de)serialize functions settings
- Allow js to use pre-stored (de)serialize functions
- Add back lazy store (implemented in js)

Default changes:

- Share store to resource table by default
- Enable auto save with 100ms debounce time by default
- Use pretty json by default, close #1690