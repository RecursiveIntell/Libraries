# 14 — AiDENs Design Laws

1. Make the correct architecture easier than the cursed shortcut.
2. Public API simple, internal boundaries strict.
3. Profiles expand to plans; plans validate before runtime.
4. Receipts are not logging. Receipts are execution evidence.
5. Capability truth is distinct from configuration intent.
6. Disabled means absent.
7. Parser fallback is degraded and receipt-bearing.
8. UI displays approval state; it does not own approval truth.
9. Host wake mechanisms wake; they do not define schedule truth.
10. Runner coordinates; it does not own memory truth.
11. Model output crossing a boundary must be validated, canonicalized, and repair-receipted if changed.
12. Every dangerous capability has an explicit permit story.
13. Tests encode footguns.
