## Commit: Add support for UUID in `to_json` function in Postgres decoder

### Changes:
- The `to_json` function in `postgres.rs` has been modified to include `UUID` as a supported type for decoding.
- In the match statement for `v.type_info().name()`, the type `UUID` has been added to the list of supported types that decode into `JsonValue::String`.