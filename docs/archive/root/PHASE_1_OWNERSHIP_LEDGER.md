
# PHASE 1 — CANONICAL OWNERSHIP LEDGER
# Generated 2026-05-28 from live Cargo.toml inventory and source inspection.

## Legend
- canonical: owns a concept no other crate should own; intended for downstream use.
- support: helper or primitive; used by canonical crates; not app-facing.
- duplicate: another crate owns this concept; quarantine or remove.
- stale: leftover from prior pass; not current.
- partial: incomplete; experimental; should not be presented as release-ready.
- quarantine: unclear ownership; do not build until resolved.
- delete candidate: safe to remove; no workspace member depends on it.

## Canonical Owners by Concept
| Concept | Canonical Owner Crate(s) | Forbidden Duplicate Owners |
|---|---|---|
| Stable IDs, digests, scope/trace primitives | stack-ids | Any local ID generation in app crates |
| Semantic memory / projection / query | semantic-memory | turbo-semantic (duplicate), local memory layers in apps |
| Forge bridge / deterministic import | forge-memory-bridge, semantic-memory-forge | Local substitute in app crates |
| Tool-call receipts / tool execution evidence | llm-tool-runtime | App-level hidden tool state |
| Verification / policy / control | verification-adjudication, -calibration, -control, -policy | App-level "done" flags |
| Kernel / oracle / conformance | recursive-kernel-core, kernel-execution, kernel-conformance, kernel-oracles | App-level oracle semantics |
| Graph compilation / constraints | constraint-compiler, agent-graph | Local graph eval in apps |
| Quantization / compression math | turbo-quant, fib-quant, quant-governor, quant-eval | poly-kv codec internals only, no reimplementation |
| Compressed KV pool | poly-kv (with quant-codec-core traits) | Hidden runtime cache maps |
| Contract / schema / versioned envelopes | contract-schema-gen | Local schema versions conflicting with canonical |
| Causal edit attribution / structured patches | forge-engine (living-memory) | Local diff engines |
| Boundary compilation | boundary-compiler | AiDENs scaffold boundary-compiler-core (duplicate name, not in workspace) |
| Batch queue / job queue | job-queue, ai-batch-queue, tauri-queue | App-level queue state |
| Remote oracle admission | remote-oracle-admission | App-level fallback oracle |
| Attestation / assurance | attestation-exchange, assurance-runtime | App-level "verified" flags |
| Authority / delegation | authority-delegation | App-level permission shims |
| Bitemporal runtime | bitemporal-runtime | Collapsing valid/recorded time |
| Continuity / effect / mechanism / knowledge | continuity-runtime, effect-runtime, mechanism-runtime, knowledge-runtime | App-level runtime truth stores |
| LLM pipeline / output parsing | llm-pipeline, llm-output-parser | App-level parser wrappers |
| Vision / comfyui | ollama-vision, comfyui-rs | App-level vision wrappers |
| Discovery portfolio | discovery-portfolio | App-level search index |
| Federated settlement | federated-settlement | App-level settlement |
| Profile runtime | profile-runtime | App-level actor profiles |
| Spec execution | spec-execution | App-level spec eval |
| Claim ledger | claim-ledger | App-level claim tracking |
| Receipt benchmarks | receipt-bench | None (benchmark-only) |
| Agent guard | agent-guard | App-level security decisions |
| Constitutional / sandbox / cea / check / mindstate / stabilizer / typed-patch | Primitives/* | App-level copies |
| SCR runtime | scr-runtime-compression | None (integration-only) |

## Root Workspace Member Status
| Crate | Path | Owner Status | Owned Concepts | Concepts It Must Not Own | Downstream | Tests | Action |
|---|---|---|---|---|---|---|---|
| agent-graph | agent-graph | canonical | Graph compilation, agent coordination | IDs, kernel truth | AiDENs | yes | harden |
| agent-guard | agent-guard | canonical | Security decisions, guardrails | App-level truth | AiDENs | yes | harden |
| ai-batch-queue | ai-batch-queue | canonical | Batch queue primitives | Memory semantics | AiDENs | yes | fix clippy |
| assurance-runtime | assurance-runtime | canonical | Assurance evidence | Verification truth | AiDENs | yes | harden |
| attestation-exchange | attestation-exchange | canonical | Attestation protocol | Identity truth | AiDENs | yes | harden |
| authority-delegation | authority-delegation | canonical | Delegation chains | Permission truth | AiDENs | yes | harden |
| bitemporal-runtime | bitemporal-runtime | canonical | Valid vs recorded time | App time collapse | AiDENs | yes | fix clippy |
| boundary-compiler | boundary-compiler | canonical | Boundary compilation | Schema truth | AiDENs | yes | fix clippy |
| claim-ledger | claim-ledger | canonical | Claim tracking | Verification truth | downstream | yes | fix clippy |
| comfyui-rs | comfyui-rs | canonical | ComfyUI workflow RS | Vision truth | downstream | yes | fix clippy |
| constitutional-memory | constitutional-memory | canonical | Constitutional memory storage | Semantic query truth | downstream | yes | harden |
| constraint-compiler | constraint-compiler | canonical | Constraint compilation | Kernel truth | AiDENs | yes | harden |
| continuity-runtime | continuity-runtime | canonical | Continuity evidence | Time truth | downstream | yes | harden |
| contract-schema-gen | contract-schema-gen | canonical | Schema generation | App schema inventions | AiDENs | yes | harden |
| discovery-portfolio | discovery-portfolio | canonical | Discovery indexing | Search truth | downstream | yes | harden |
| effect-runtime | effect-runtime | canonical | Effect execution evidence | Kernel truth | downstream | yes | harden |
| federated-settlement | federated-settlement | canonical | Settlement protocol | App settlement shims | downstream | yes | harden |
| fib-quant | fib-quant | canonical | FibQuant math | PolyKV hidden math | poly-kv, turbo-quant | yes | harden |
| forge-memory-bridge | forge-memory-bridge | canonical | Deterministic import, digest preservation | Memory query truth | semantic-memory, AiDENs | yes | harden |
| forge-pilot | forge-pilot | canonical | Forge orchestration | App-level truth | downstream | yes | harden |
| job-queue | job-queue | canonical | Job queue | App-level queue state | tauri-queue, AiDENs | yes | fix clippy |
| kernel-conformance | kernel-conformance | canonical | Conformance checking | App-level check | AiDENs | yes | harden |
| kernel-execution | kernel-execution | canonical | Kernel execution | App-level execution | AiDENs | yes | harden |
| kernel-oracles | kernel-oracles | canonical | Oracle checks | App-level oracle | AiDENs | yes | harden |
| knowledge-runtime | knowledge-runtime | canonical | Knowledge evidence | App-level knowledge | downstream | yes | harden |
| living-memory | living-memory/living-memory | canonical | Causal edit attribution (forge-engine) | App-level diff | Gloss, AiDENs | yes | harden |
| llm-output-parser | llm-output-parser | canonical | LLM output parsing | App-level parsing | llm-pipeline | yes | harden |
| llm-pipeline | llm-pipeline | canonical | Pipeline orchestration | App-level pipeline | AiDENs | yes | harden |
| llm-tool-runtime | llm-tool-runtime | canonical | Tool-call receipts | App-level tool state | AiDENs | yes | fix clippy |
| mechanism-runtime | mechanism-runtime | canonical | Mechanism evidence | App-level mechanism | downstream | yes | harden |
| ollama-vision | ollama-vision | canonical | Ollama vision | App-level vision | downstream | yes | harden |
| poly-kv | poly-kv/crates/poly-kv | canonical | Compressed KV pool | Codec math (owned by quant-codec-core) | downstream | yes | harden |
| quant-codec-core | poly-kv/crates/quant-codec-core | support | Codec traits, IDs, shapes | Pool semantics | poly-kv | yes | harden |
| profile-runtime | profile-runtime | canonical | Profile orchestration | App-level profiles | AiDENs | yes | harden |
| quant-eval | quant-eval | canonical | Quant evaluation | App-level eval | quant-governor | yes | harden |
| quant-governor | quant-governor | canonical | Quant governance | App-level policy | scr-runtime-compression | yes | fix clippy |
| receipt-bench | receipt-bench | support | Benchmark suite | App-level benchmarks | downstream | yes | fix clippy |
| recursive-kernel-core | recursive-kernel-core | canonical | Core kernel primitives | App-level kernel | AiDENs | yes | harden |
| remote-oracle-admission | remote-oracle-admission | canonical | Oracle admission | App-level oracle | downstream | yes | harden |
| scr-runtime-compression | scr-runtime-compression | canonical | SCR runtime compression | Codec math | downstream | yes | harden |
| semantic-memory | semantic-memory | canonical | Semantic search, projection, query | TurboQuant math (owned by turbo-quant) | AiDENs, Gloss | yes | fix clippy |
| semantic-memory-forge | semantic-memory-forge | canonical | Forge export from semantic memory | Query semantics | forge-memory-bridge | yes | harden |
| spec-execution | spec-execution | canonical | Spec execution | App-level spec | downstream | yes | harden |
| stack-ids | stack-ids | canonical | Stable IDs, digests | App-level IDs | ALL | yes | harden |
| tauri-queue | tauri-queue | canonical | Tauri integration queue | Queue core (owned by job-queue) | downstream | yes | harden |
| turbo-quant | turbo-quant | canonical | TurboQuant math | Pool semantics | semantic-memory | yes | harden |
| verification-adjudication | verification-adjudication | canonical | Adjudication | App-level verdict | AiDENs | yes | harden |
| verification-calibration | verification-calibration | canonical | Calibration | App-level calibration | AiDENs | yes | harden |
| verification-control | verification-control | canonical | Control | App-level control | AiDENs | yes | harden |
| verification-policy | verification-policy | canonical | Policy | App-level policy | AiDENs | yes | harden |

## Primitives Workspace Member Status
| Crate | Path | Owner Status | Action |
|---|---|---|---|
| cea-core | Primitives/cea-core | support | harden |
| cea-sqlite | Primitives/cea-sqlite | support | harden |
| cea-store | Primitives/cea-store | support | harden |
| check-runner | Primitives/check-runner | support | harden |
| effect-signature | Primitives/effect-signature | support | harden |
| forge-policy | Primitives/forge-policy | support | harden |
| mindstate-core | Primitives/mindstate-core | support | harden |
| sandbox-workspace | Primitives/sandbox-workspace | support | harden |
| stabilizer-core | Primitives/stabilizer-core | support | harden |
| typed-patch | Primitives/typed-patch | support | fix clippy |

## Duplicate / Shadow / Stale Crates
| Crate | Path | Issue | Action |
|---|---|---|---|
| turbo-semantic | turbo-semantic/ | Duplicate name "semantic-memory" v0.5.0; MIT vs Apache-2.0; NOT in workspace; shadow of semantic-memory | DELETE or QUARANTINE |
| boundary-compiler-core | AiDENs/scaffold/boundary-compiler-core | Same concept as boundary-compiler; not in root workspace; potential stale scaffold | Quarantine or archive |
| coding-agent-repo-fixture | AiDENs/docs/codex-runs/.../Cargo.toml | Fixture crate, not a library | Keep as fixture |

## Notable Ownership Boundaries
- semantic-memory owns projection/query/storage; it must NOT own TurboQuant math (turbo-quant owns that).
- poly-kv owns pool semantics; it must NOT own TurboQuant or FibQuant math.
- forge-memory-bridge owns deterministic import/digest preservation; it must NOT own query semantics.
- AiDENs is an APP WORKSPACE; it must not own canonical library semantics. Its path deps point to root workspace crates correctly.
