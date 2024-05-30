---
"shell": patch
---

When the "raw" encoding option is specified for a shell process, all bytes from the child's output streams are passed to the data handlers. 
This makes it possible to read output from programs that write unencoded byte streams to stdout (like ffmpeg)