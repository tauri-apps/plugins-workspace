---
'log': 'minor:feat'
'log-js': 'minor:feat'
---
Add a `is_skip_logger` flag to the Log Plugin `Builder` struct, a `skip_logger()` method to the Builder, and logic to avoid acquiring (creating) a logger and attaching it to the global logger. Since acquire_logger is pub, a `LoggerNotInitialized` is added and returned if it's called when the `is_skip_looger` flag is set. Overall, this feature permits a user to avoid calling `attach_logger` which can only be called once in a program's lifetime and allows the user to control the logger returned from `logger()`. Additionally, it also will allow users to generate multiple Tauri Mock apps in test suites that run and parallel and have the `log` plugin attached (assuming they use `skip_logger()`).
