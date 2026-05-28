# P20 Master Issue Matrix

| ID | Priority | Stream | Title | Why | Acceptance | Required proof |
|---|---|---|---|---|---|---|
| P20-001 | P0 | Proof | Run full cargo/script baseline and capture evidence | All later claims need proof | cargo/check/test/clippy/verify logs captured; failures fixed or quarantined | target/aidens-final-audit/*.log |
| P20-002 | P0 | Docs | Rewrite root README into real project README | Current README may be stale Codex-bundle prose | README has operator/developer quickstart, supported matrix, limitations | README.md + docs-code report |
| P20-003 | P0 | Docs | Reconcile STATUS and issue matrices with actual code | STATUS overclaim is the current main risk | No done/implemented claim without code+test proof | docs-code report |
| P20-004 | P0 | Contracts | Inventory every aidens-contracts public type | Oversized contracts crate can become shadow truth | Ownership inventory complete; ambiguous types resolved | CONTRACT_OWNERSHIP_INVENTORY |
| P20-005 | P0 | Scanner | Install deterministic P20 scanner into verify gate | Grep-only checks are too weak | scanner runs in p20_verify and verify.sh or documented equivalent | p20-scan.json/md |
| P20-006 | P0 | Providers | Lock provider capability matrix | Provider truth must match executable support | mock/Ollama supported only if tested; cloud unavailable unless implemented | provider matrix + tests |
| P20-007 | P0 | Runner | Prove vertical slice with durable receipts | No E2E path means no finish | tool turn test passes and records event log/receipts | runner test + audit log |
| P20-008 | P0 | Agency | Add agency/influence policy crate or module | New research requires first-class influence governance | agency artifacts + pre-generation gate + receipts exist | agency tests/report |
| P20-009 | P0 | Agency | Add influence/advice receipts | Consequential influence without receipts is non-auditable | high-impact/memory/repeated-nudge receipts emitted | agency eval report |
| P20-010 | P0 | Testkit | Close pending temporal reference behavior or demote docs | Phase 09 closed the temporal reference branch for supported claims | no deferred reference for supported features | `crates/aidens-integration-tests/tests/phase_09_reference_hostile_tests.rs` |
| P20-011 | P1 | Memory | Prove canonical memory adapter path | AiDENs must delegate to semantic-memory lane | forge export -> bridge -> import -> query test exists | integration test |
| P20-012 | P1 | Kernel | Prove canonical kernel adapter path | Kernel law belongs to canonical crates | compile/execute/oracle/witness path tested or docs demote | integration test |
| P20-013 | P1 | Governance | Prove verification/governance adapter path | Control decisions require receipts | adapter test + receipt proof | integration test |
| P20-014 | P1 | Repair | Demote or prove repair record path | Repair cannot stay folklore | repair tests or status demotion | repair report |
| P20-015 | P1 | Boundary | Harden JSON repair/treatment integrity | Parser repair is semantic choke point | duplicate-key and treatment-changing repair tests pass | boundary tests |
| P20-016 | P1 | Digests | Audit digest algorithm labels | Mislabeling hash algorithm breaks evidence integrity | no hardcoded wrong digest prefixes | scanner + unit tests |
| P20-017 | P1 | Scaffolds | Resolve scaffold crate fate | Scaffolds cannot be product-ready | implemented, removed, or marked deferred | scaffold-status.md |
| P20-018 | P1 | Queue | Truthfully bound queue/schedule/wake support | One-shot vs recurring semantics must be honest | queue/schedule docs and tests match implementation | tests + status |
| P20-019 | P1 | CLI | Update doctor/package audit outputs | Operator UX must reflect real support | CLI reports supported/partial/deferred correctly | CLI tests |
| P20-020 | P1 | Audit | Generate final audit bundle | Release handoff needs artifacts | bundle exists and summarizes pass/fail | target/aidens-final-audit |

## Completion rule

Every P0 issue must be either `done` with proof or `failed/quarantined` with an explicit release block. P20 cannot pass with open P0s.
