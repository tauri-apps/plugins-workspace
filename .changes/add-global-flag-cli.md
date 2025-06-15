---
"cli": minor
"cli-js": minor
---

Added a new `global` boolean flag to the `CliArg` struct to support global CLI arguments. This flag allows arguments like `--verbose` to be marked as global and automatically propagated to all subcommands, enabling consistent availability throughout the CLI.

The new field is optional and defaults to false.
