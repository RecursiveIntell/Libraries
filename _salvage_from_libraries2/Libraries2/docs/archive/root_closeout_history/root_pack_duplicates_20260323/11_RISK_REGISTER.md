
# 11_RISK_REGISTER

## Top risks

### R1 — false green status persists after packaging fixes
If the dashboard/receipt/evidence manifest are not rewritten after the front-door fixes, the repo will remain materially misleading.

### R2 — panic-audit cleanup gets misread as “ignore panics”
The goal is to separate production code from inline test modules, not to lower the safety bar.

### R3 — broken v25 scripts get silently abandoned
If they are no longer part of the supported story, retire them explicitly. Do not leave dead commands in place.

### R4 — CI arrives before gate convergence
A CI workflow that runs a different gate set than the local front door will create two incompatible truths.

### R5 — thin governance shells continue to dominate first impressions
Even after the front door is repaired, external reviewers will still hit the naming/doc credibility problem unless it is handled deliberately.

### R6 — giant modules keep slowing every future review
If `forge-pilot`, `semantic-memory`, `profile-runtime`, and `knowledge-runtime` stay concentrated in giant files, the repo remains needlessly hard to maintain.

## Forbidden shortcuts

- deleting a failing file instead of fixing or retiring its surface
- narrowing the claimed support lane after the fact without updating the receipt
- keeping curated green checks while presenting them as full-workspace signals
- leaving `.github/workflows/ci.yml` absent while talking about a finish pass
