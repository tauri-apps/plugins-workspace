---
autostart: minor:enhance
autostart-js: minor:enhance
---

Use the generic `IntoIterator<Item = impl Into<String>>` instead of `Vec<&'static str>` as the parameter type for `init(args)` to remove the `&'static` lifetime constraint.
