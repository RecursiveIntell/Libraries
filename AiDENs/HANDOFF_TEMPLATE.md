# Pass Handoff Template

Use this exact shape at the end of every pass.

```markdown
# PXX Handoff — <pass title>

## Summary

- Status: complete | blocked | partial | deferred
- Commit/branch:
- Date:

## Files changed

- `path`: reason

## Artifacts introduced or changed

- `ArtifactNameV1`: owner crate, schema path, fixture path

## Tests added or updated

- `test name`: what it proves

## Commands run

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/verify.sh
```

## Results

- fmt:
- check:
- test:
- clippy:
- verify:

## Acceptance gates

- [ ] gate 1
- [ ] gate 2
- [ ] gate 3

## Blockers / risks

- blocker:
- exact evidence:
- proposed next action:

## Next pass readiness

- Ready for PXX+1: yes | no
- Reason:
```
