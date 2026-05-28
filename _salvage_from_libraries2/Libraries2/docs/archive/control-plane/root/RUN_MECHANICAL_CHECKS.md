# RUN_MECHANICAL_CHECKS.md

Run equivalent search/grep/static-audit checks and report findings.

## 1. Legacy naming / path checks
- find bare `ImportEnvelope` outside compat-labeled legacy contexts
- find `import_envelope(` call sites and classify each as:
  - compat-only still valid
  - stale normal-path usage that must be migrated

## 2. Shared primitive ownership checks
- inventory all shared ID / trace / scope primitives in `stack-ids`
- search for local equivalents still living in other crates
- flag any new business logic accidentally added to `stack-ids`

## 3. Namespace / scope checks
- find manual namespace -> scope conversion outside canonical `ScopeKey` helpers
- find code still treating raw namespace string as the canonical partition key in newly migrated paths

## 4. Retry / trace shape checks
- find local `TraceId`
- find `trace_id: String`
- find `attempt: u32`
- find `attempt_count`
- find stale comments/examples implying “new AttemptId per retry”
- find stale owner tables contradicting the canonical retry matrix

## 5. Coupling checks
- find direct Forge -> `semantic-memory` normal-path bypasses
- classify remaining seams as compat-only or unresolved entanglement

## 6. Compatibility labeling checks
- ensure all surviving legacy surfaces are phase-labeled
- ensure comments/examples/docs do not present compat-only surfaces as preferred normal guidance

## 7. Reporting checks
- ensure the new current-state snapshot visibly reflects:
  - `stack-ids` shared primitive inventory
  - `semantic-memory` canonical vs compat import surfaces
  - bridge contract visibility
  - supporting-crate retry/trace propagation status

## Output requirement
For each check:
- report what matched,
- whether it was acceptable,
- and what was changed.
