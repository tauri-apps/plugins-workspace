---
"log": minor:feat
"log-js": minor
---

Added the `FileOpenStrategy` for log rotation. It defaults to append into existing file if any (previous behaviour), and brings a new feature to create a new file per session: `FileOpenStrategy::Rotate`. 
