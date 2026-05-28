# P23 Rollback and Quarantine Plan

## z.py failure

If `z.py` changes make packaging worse, revert to the last strict-passing P22 `z.py`, then reapply only:

1. verifier self-replay fix,
2. legacy `zip.py` deprecation,
3. generic current-run derivation.

Do not keep half-broken package modes.

## Capability failure

If the agent-run capability cannot be finished without semantic invention, quarantine the capability behind `partial` and keep only tested fixture/library API behavior.

## Archive classification failure

If classification is ambiguous, move the artifact to `docs/codex-runs/archive/unclassified/<stamp>/` with a manifest and require human review. Do not leave it active and unclassified.

## Cargo failure

Cargo failures block promotion. If a cargo gate is not runnable because of environment/toolchain limits, capture exact toolchain status and run all available non-cargo gates. Do not claim cargo success without logs.
