# RUN_OUTPUT_TEMPLATE.md

Use this structure for the final response.

# Final Conformance Report

## 1. Authority stack used
- confirm the exact document priority order used

## 2. Work classes completed
### A. Core-layer completion
- `semantic-memory`
- `stack-ids`
- `forge-memory-bridge`
- `knowledge-runtime` (if touched)

### B. Compatibility-surface containment
- list surviving compat-only surfaces
- state why they still survive
- state removal condition for each

### C. Supporting-crate propagation
For each crate touched:
- old visible shape
- new shape
- compat shims retained
- tests/events/checkpoints/docs updated

## 3. `stack-ids` inventory
Provide a table:
- primitive name
- implemented / added now / deferred / still external / hidden from old snapshot
- notes

## 4. Bridge parity resolution
State explicitly:
- code already matched patch record and reporting was improved
or
- code lagged patch record and was completed now

## 5. Forge -> memory seam classification
State explicitly:
- compat-only this phase
or
- unresolved entanglement reduced now

## 6. Mechanical check results
Summarize each required check and its result.

## 7. Test-obligation matrix
Map named obligations to:
- existing test(s)
- newly added test(s)
- still missing

Do not hide missing obligations.

## 8. New current-state snapshot
- provide the filename
- summarize what changed relative to `LATEST5.md`

## 9. Regression scan
Confirm whether any already-adopted correction regressed.
If none, say so plainly.
If any did, name them.

## 10. Remaining debt
List only the debt that still remains after this pass.
Be explicit. No fluff.
