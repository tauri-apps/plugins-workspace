---
"cli": minor
"cli-js": minor
---

Added `Cli.matches_from(args)`. This can be combined with the `args` passed to the callback of `tauri_plugin_single_instance::init` to parse the command line arguments passed to subsequent instances of the application.
