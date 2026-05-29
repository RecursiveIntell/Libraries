# P24 hard audit — AiDENs before the completion super-pass

## Verdict

AiDENs is no longer a fake-ready scaffold. P23 delivered useful local operator capability, package hygiene, and a fixture-backed run path. The remaining risk is not absence of code. The remaining risk is **ownership drift**: AiDENs must become a product/compiler/orchestration surface over canonical libraries, not a shadow truth system.

The P24 run should therefore finish **runnable local product slices** while freezing canonical seams:

- execution evidence must bind to `semantic_memory_forge::ExecutionContextV1`;
- identity must route through `stack-ids` and canonical library owners;
- memory truth must route through `semantic-memory-forge`, `forge-memory-bridge`, `semantic-memory`, and `knowledge-runtime`;
- verification/repair/control semantics must route to `verification-*`, `semantic-memory-forge`, and related canonical owners;
- AiDENs display/report artifacts must carry support-tier honesty and canonical backpointers.

## Source-package health

The latest AiDENs context package passed strict configured validation with zero sidecar findings. It included 1189 files and 482 Rust files across AiDENs plus adjacent canonical library roots. It had 33 AiDENs workspace members and 62 local schema files.

Important caveat: the AiDENs package intentionally includes many sibling library roots via external Cargo path dependencies (77 path dependency entries in the static scan). That is good for Codex context, but it means Codex must respect owner boundaries.

## Code-shape observations

### Large surfaces

| File | LOC | Nonblank LOC |
|---|---:|---:|
| `AiDENs/crates/aidens-contracts/src/lib.rs` | 10190 | 9524 |
| `AiDENs/crates/aidens-cli/src/lib.rs` | 5773 | 5413 |
| `knowledge-runtime/tests/cross_crate_proof.rs` | 3609 | 3283 |
| `AiDENs/crates/aidens-tool-kit/src/lib.rs` | 2453 | 2301 |
| `verification-control/src/lib.rs` | 1914 | 1810 |
| `AiDENs/crates/aidens-runner/src/lib.rs` | 1909 | 1784 |
| `AiDENs/crates/aidens-agency-kit/src/lib.rs` | 1814 | 1707 |
| `profile-runtime/src/adapters.rs` | 1791 | 1753 |
| `semantic-memory/tests/import_ugly_cases.rs` | 1782 | 1652 |
| `semantic-memory/tests/import_boundary_tests.rs` | 1640 | 1506 |
| `living-memory/living-memory/src/lab/evidence.rs` | 1626 | 1528 |
| `semantic-memory/src/lib.rs` | 1618 | 1476 |
| `semantic-memory/src/db.rs` | 1613 | 1463 |
| `semantic-memory/src/search.rs` | 1595 | 1479 |
| `AiDENs/crates/aidens-provider-kit/src/lib.rs` | 1471 | 1372 |

Interpretation:

- `aidens-contracts/src/lib.rs` is a large display/report registry. P24 should stabilize and inventory it, not casually split it unless tests make that safe.
- `aidens-cli/src/lib.rs` and `aidens-runner/src/lib.rs` are now real product surfaces. P24 should convert their local run evidence into canonical-seam-bearing evidence, not redesign them from scratch.
- `aidens-tool-kit` and `aidens-agency-kit` are high leverage for a supported local coding-agent lane.

### Risk scan counts

- `todo_macro`: 0
- `unimplemented_macro`: 0
- `panic_macro`: 11
- `unwrap_call`: 586
- `expect_call`: 148
- `unsafe`: 0
- `placeholder`: 7
- `stub`: 0
- `fake`: 21
- `mock`: 276
- `deferred`: 86
- `unsupported`: 27
- `scaffold`: 138

Interpretation:

- `todo!` and `unimplemented!` are zero; that is good.
- `mock`, `deferred`, `scaffold`, and `unsupported` remain common because the project intentionally distinguishes supported, partial, scaffold, and deferred surfaces. P24 must keep those labels honest.
- `unwrap`/`expect` counts are high enough that P24 should focus on operator/runtime paths first, not tests. The acceptance bar is not "zero unwrap"; it is "no panic/unwrap on supported operator-facing paths."

### Profile readiness

| Profile crate | LOC | Status signal |
|---|---:|---|
| `aidens-profile-coding` | 84 | scaffold=False | deferred=False |
| `aidens-profile-daemon` | 18 | scaffold=True | deferred=False |
| `aidens-profile-desktop` | 18 | scaffold=True | deferred=False |
| `aidens-profile-memory` | 18 | scaffold=True | deferred=False |
| `aidens-profile-research` | 18 | scaffold=True | deferred=False |

Interpretation:

- `aidens-profile-coding` is the best candidate for P24 supported-local promotion.
- daemon/desktop/memory/research profile crates are scaffold-sized. Promote only daemon-safe if the queue/schedule/wake local slice lands with tests; leave the others explicitly scaffold/partial.

## Primary blockers

1. **Run bundle semantics are local.** P23 did not claim canonical memory/identity/verification ownership. P24 must bind run-bundle evidence to canonical execution context and receipt semantics.
2. **Active docs drift.** Some active docs still carry P22/P23 framing. P24 must leave one current state.
3. **Verifier hardening remains crucial.** A local script check showed signs of non-terminating behavior under container constraints. P24 must make every verifier bounded, pruned, and receipt-emitting.
4. **Memory/runtime vertical slice is the missing proof.** Adapters exist; P24 must prove a real canonical export/import/query path.
5. **Product utility needs one supported lane.** The coding-agent local lane is the best ROI. Daemon-safe queue is second.
6. **Parser/repair boundary must hard-fail ambiguity.** Structured output is evidence; lenient repair without provenance is a silent poison channel.

## What P24 must not do

- Do not implement V10+ regional decoder geometry as the finish line.
- Do not promote desktop/research/memory profiles without runnable evidence.
- Do not let AiDENs mint canonical `EpisodeBundle`, `ExecutionContext`, `EvidenceBundle`, `RepairRecord`, `VerificationPlan`, `ExportEnvelope`, or `ProjectionImportBatch` semantics.
- Do not convert scaffold/partial/deferred into "supported" with prose.
- Do not refactor the large contracts/CLI files merely for aesthetics before locking evidence gates.

## Completion definition

AiDENs is "as complete as possible" for P24 when it can:

1. run a local test agent and local coding-agent lane;
2. emit typed run bundles with canonical execution context and replay-normalized evidence;
3. import/query a canonical memory fixture through the canonical bridge/runtime path;
4. run a daemon-safe local queue lifecycle if time permits;
5. fail closed on parser/repair ambiguity;
6. produce a final package with zero findings, active P24 docs, command evidence, and exact support claims.
