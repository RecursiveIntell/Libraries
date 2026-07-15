# Issue matrix

Source baseline: `c65972dbdf0ee5a7b472019b12c905a9de77c5c9` on `p32-schema-compat`.

| ID | Severity | Phase | Title | Primary surface |
|---|---:|---:|---|---|
| `CTRL-001` | P1 | -1 | Establish a truthful active-run control surface | `AGENTS.md` |
| `AG-001` | P0 | 0 | Preserve graph execution failures instead of returning Complete | `agent-graph/src/engine.rs` |
| `GOV-001` | P0 | 0 | Make missing, malformed, contradictory, or unavailable governance fail closed | `forge-pilot/src/governance_gate.rs` |
| `CMP-001` | P0 | 0 | Remove fake exact decoding and ignored codec dispatch | `scr-runtime-compression/src/codec_dispatch.rs` |
| `ID-001` | P1 | 1 | Make stack-ids the exclusive canonical ID type and construction authority | `stack-ids/src/ids.rs` |
| `ID-002` | P1 | 1 | Remove competing ID generators and raw canonical ID strings | `agent-graph` |
| `DIG-001` | P1 | 2 | Replace separator-based digest composition with typed length framing | `stack-ids/src/digest.rs` |
| `SCP-001` | P1 | 2 | Prevent lossy ScopeKey-to-legacy namespace collapse | `stack-ids/src/scope.rs` |
| `LED-001` | P1 | 2 | Make claim-ledger parsing, IDs, and completeness fail closed | `claim-ledger/src/ids.rs` |
| `INT-001` | P1 | 3 | Create one codec/profile/wire contract for interchangeable backends | `poly-kv/crates/quant-codec-core` |
| `QUE-001` | P1 | 4 | Make ai-batch-queue claim and completion atomic | `ai-batch-queue/src/queue.rs` |
| `QUE-002` | P2 | 4 | Make job-queue cancellation, heartbeat, lease, and DB concurrency explicit | `job-queue/src/lib.rs` |
| `SEM-001` | P1 | 4 | Align semantic-memory search corruption behavior with strict integrity claims | `semantic-memory/src/search.rs` |
| `CI-001` | P1 | 5 | Certify every required workspace and feature lane | `.github/workflows/` |
| `LINT-001` | P1 | 5 | Make lint policy mandatory or explicitly exempted | `Cargo.toml` |
| `EVD-001` | P1 | 5 | Separate evidence recording from verification and bind receipts to source/environment | `scripts/run_release_gates.py` |
| `DOC-001` | P1 | 5 | Bind benchmark and readiness claims to reproducible evidence | `fib-quant/README.md` |
| `PERF-001` | P2 | 6 | Optimize only after correctness with reproducible baselines | `stack-ids` |

## Interpretation

- **P0**: immediate false-success/release blocker.
- **P1**: correctness, authority, integrity, or release-proof blocker.
- **P2**: material operational/efficiency debt, after prerequisite correctness.
- A changed file or green test subset is not closure.
- Full issue contracts are in `05_ISSUE_MATRIX.json`.
