# P21 Rollback / Repair Plan

## If cargo fails

Fix compile/test failures before any feature work. If the failure is due to missing sibling canonical crates, report environment failure; do not stub canonical crates locally.

## If package scanner fails

Restore missing files. Do not delete references unless the feature is intentionally removed and all docs/tests are updated truthfully.

## If ownership conflict appears

Stop. Identify canonical owner. Replace local duplicate with delegation or quarantine the feature.

## If provider capability is false

Mark provider unavailable or degraded. Remove support claims. Add tests proving false claims cannot reappear.

## If agency eval fails

Fix policy or receipt emission. Do not weaken eval expectations unless the expectation is clearly invalid and the reason is documented.

## If Recall extraction introduces assumptions

Revert extraction code. Keep only docs/templates that are clearly generic.

## If stretch work destabilizes base gates

Revert stretch changes and record the rollback in `handoffs/p21/STRETCH_ROLLBACK.md`.
