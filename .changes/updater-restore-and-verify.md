---
updater: minor
---

Add `Updater::restore_update` to restore an update from cached release metadata, and `Update::verify` to verify package bytes using the configured signing key and signed-version policy. Restored updates always require the signature to bind the announced version.

`Update::install` now verifies the package bytes before installing them, and `Updater::check` resolves to `None` instead of a target error when the release is not an update and has no entry for the current target.
