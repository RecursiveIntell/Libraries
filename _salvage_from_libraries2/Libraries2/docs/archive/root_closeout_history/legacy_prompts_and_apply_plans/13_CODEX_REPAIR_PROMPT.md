# Codex repair prompt

Use this if the first pass partially landed but still left obvious debt.

```text
Continue the closeout pass, but focus only on unfinished or fake-complete items.

Re-read:
- 04_MASTER_ISSUE_MATRIX.md
- 08_EXACT_FILE_TOUCH_MAP.md
- 10_ACCEPTANCE_AND_COMMANDS.md

Then inspect the repo and fix whatever is still incomplete.

Typical failure modes to look for:
- `forge-pilot/src/main_support/mod.rs` is still large
- `knowledge-runtime/src/runtime/core.rs` still contains the real implementation
- broad `allow(deprecated)` remained in supported core
- production `unwrap` / `expect` still exist in the audited crates
- `stack-ids` constructors are still too permissive
- root config files/scripts/CI were not fully wired
- mirror-drift check is still ceremonial instead of exact
- metadata/docs were skipped

Do not re-plan the whole repo. Finish the remaining implementation work and rerun the acceptance lane.
```
