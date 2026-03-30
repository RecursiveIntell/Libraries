# Source basis and reconciliation

## Inputs actually used

1. **Current repo snapshot** at `/mnt/data/auditwork/repo` from `libraries-source-clean-20260324.zip`.
2. **User hostile audit** pasted in chat on 2026-03-24.
3. **Repo-local historical audits** already present in the archive:
   - `03_HOSTILE_AUDIT.md`
   - `04_CLAUDE_RECONCILIATION.md`
4. **Constitutional spec line**:
   - `CANONICAL_STACK_SPEC_V6.md`
   - `CANONICAL_STACK_SPEC_V9_EPISODIC_AUTHORITY_AND_EXECUTION_EVIDENCE.md`
   - `canonical_stack_spec_v_11_executable_semantics_and_proof_governance.md`
   - `CANONICAL_STACK_SPEC_V12_REGIONAL_FIXPOINT_RUNTIME.md`
   - `CANONICAL_STACK_SPEC_V16_FEDERATED_CLAIM_SETTLEMENT_AND_TREATY_RUNTIME.md`
   - `CANONICAL_STACK_SPEC_V17_MECHANISM_LIBRARY_AND_THEORY_SEARCH_RUNTIME.md`
   - `CANONICAL_STACK_SPEC_ENDSTATE_RECURSIVE_SUBTRACTIVE_RUNTIME.md`

## Snapshot facts verified directly

- Workspace members: **30**
- Default members: **29**
- Scaffold duplicates shadowing production names: **17**
- `effect-runtime` public functions: **0**
- `llm-tool-runtime` approval carrier: **raw `Option<String>`**
- `kernel-oracles` test markers: **13**
- `kernel-execution` test markers: **10**
- `recursive-kernel-core` test markers: **3**
- CI workflow present: **yes** (`.github/workflows/ci.yml`)

## What the current snapshot already does well

- Episode bundles and execution context are real artifact families in `semantic-memory-forge`.
- The bridge already enforces at least one hard invariant: canonical bundle-bearing exports without `episode_id` fail.
- The core lane (`semantic-memory`, `living-memory/forge-engine`, `knowledge-runtime`, `forge-pilot`, `forge-memory-bridge`, `semantic-memory-forge`) is materially real and heavily documented/tested.
- `llm-tool-runtime` already uses enums for many runtime classifications. The governance crates should catch up to that standard.

## What the hostile audit got right and still matters

- The advisory-vs-enforcement distinction is the main problem.
- Controlled vocabularies are still too stringly.
- Several “runtime” crates are really typed surface crates.
- Scaffolds and naming mismatches pollute the review surface.
- Hotspot files remain too large.
- Panic/unwrap concentration remains non-trivial in operational lanes.

## What is stale or overstated

| Claim | Current status | Reason |
|---|---|---|
| `kernel-oracles` has zero tests | stale | 13 test markers are present in the current source tree. |
| `kernel-execution` has zero tests | stale | 10 test markers are present in the current source tree. |
| `recursive-kernel-core` has zero tests | stale | 3 test markers are present in the current source tree. |
| CI is absent | stale | `.github/workflows/ci.yml` exists and runs `make gate` plus v25 closure checks. |
| the repo is mostly shells and no real system | overstated | The core lane is deep and test-rich; the real problem is enforcement closure, not absence of substance. |

## Most important conclusion

Do **not** copy the hostile audit into the repo as-is.

Use it as an adversarial hypothesis generator, then close only the issues that are **confirmed live** in the current tree.
