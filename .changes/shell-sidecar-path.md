---
shell: minor
shell-js: minor
---

Add API to get path to sidecar binary (`Shell::sidecar_path` in Rust and `Command.sidecarPath` in JS). The JS API requires the new `shell:allow-sidecar-path` permission, scoped with `sidecar: true` entries like `shell:allow-execute`.
