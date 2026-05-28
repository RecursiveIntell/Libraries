# P20.1 Final Auditor Checklist

A hostile auditor should be able to verify:

```text
[ ] all include_str/include_bytes targets exist
[ ] MANIFEST.txt has zero missing entries
[ ] eval agency cases validate and run
[ ] aidens-testkit is pure or integration tests are split
[ ] ownership scanner sees non-empty canonical baseline in the real workspace
[ ] cargo fmt/check/test/clippy pass
[ ] provider matrix does not overclaim native tool loops
[ ] runner vertical slice produces receipts
[ ] agency receipts are emitted for high-impact/memory/tool/repeated surfaces
[ ] final audit bundle is included or linked in release artifact
```
