---
"http": patch
"http-js": patch
---
Added HTTP3 to the allowed feature flags for reqwest. This enables users to leverage HTTP3 functionality in Rust-side code without causing panics due to multiple imports with different features. Since HTTP3 is still considered experimental in reqwest, this feature is limited to Rust-side usage only.