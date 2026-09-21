---
"dialog-js": major
---

**Breaking:** Removed the deprecated `okLabel` option of `MessageDialogOptions`. Use `buttons: { ok: 'label' }` instead. `ConfirmDialogOptions` (used by `ask` and `confirm`) still accepts `okLabel` and `cancelLabel`.
