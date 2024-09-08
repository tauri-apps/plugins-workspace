---
"notification-js": patch
---

**Breaking change**: The permission type when using the API is now `'granted' | 'denied' | 'prompt' | 'prompt-with-rationale'` instead of `'granted' | 'denied' | 'default'` for consistency with Rust types. When using the `window.Notification` API the type is unchanged to match the Web API type.
