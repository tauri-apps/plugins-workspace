---
"updater": minor
---

Add a `requireSignedVersion` configuration option that binds an update to the version it was signed for.

The update endpoint response is fetched over TLS but is not itself signed, and the signature only covers the downloaded artifact. Anyone able to serve a crafted response could therefore pair an inflated `version` field with the `url` and `signature` of an older release and force a downgrade to a genuine but outdated build, because that older artifact carries a valid signature.

The Tauri CLI records the version in the signature's trusted comment, which the signature covers. With this option enabled the updater compares that signed version against the one announced by the endpoint and rejects the update when they differ:

```json
{
  "plugins": {
    "updater": {
      "requireSignedVersion": true
    }
  }
}
```

It defaults to `false` because releases signed before the CLI started recording the version carry none and would be rejected. Re-sign and re-publish every release your users can still update from before enabling it, otherwise an older signature can still be served to bypass the check. When the signature does carry a version, a mismatch is rejected whether or not the option is enabled.

This is checked independently of `allowDowngrades`: it constrains which artifact a given version number may resolve to, not whether that version is newer than the running one.
