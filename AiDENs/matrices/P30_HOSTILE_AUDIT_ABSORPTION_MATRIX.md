# P30 Hostile Audit Absorption Matrix

Source hostile audit issues absorbed: **554**.

## Counts by priority

- P0: 15
- P1: 292
- P2: 247

## Counts by severity

- Critical: 10
- High: 138
- Low: 73
- Medium: 333

## Counts by phase

- **P30-00 — Preflight, source-basis lock, workspace portability, build-certification split:** 5 issues
- **P30-01 — Executable tool-call parser boundary and strict structured-output law:** 5 issues
- **P30-02 — Patch safety, rollback truth, command sandbox, and permit fail-closed behavior:** 5 issues
- **P30-03 — Replay identity, deterministic material IDs, and exposure/attempt identity law:** 183 issues
- **P30-04 — Execution evidence defaults, durable failure receipts, retry/provider/tool evidence:** 37 issues
- **P30-05 — Verification semantics, proof debt, degradation honesty, and no advisory promotion:** 82 issues
- **P30-06 — Gate resurrection, schema/root-doc hygiene, active-doc manifest, and package evidence:** 11 issues
- **P30-07 — Hostile sweep: panic, dynamic JSON, silent degradation, lint suppression, code shape:** 220 issues
- **P30-08 — v11A/B seed: artifact runtime, operator receipts, right-graph/region/convergence hooks:** 6 issues
- **P30-09 — Full conformance, replay, final packaging, hostile auditor handoff, and unresolved risks:** 0 issues

## Top categories

- REPLAY-IDENTITY: 83
- SILENT-DEGRADATION: 80
- PANIC-SURFACE: 80
- DYNAMIC-JSON: 80
- NONDETERMINISM: 60
- LINT-SUPPRESSION: 50
- DETERMINISM: 40
- OBSERVABILITY: 30
- GATE-DRIFT: 7
- CODE-SHAPE: 7
- SCHEDULING: 6
- PARSER-BOUNDARY: 5
- PATCH-SAFETY: 2
- WORKSPACE-PORTABILITY: 2
- DEFAULTS: 2
- PACKAGE-INTEGRITY: 2
- COMMAND-SANDBOX: 2
- PACKAGE-HYGIENE: 2
- RUNTIME-PANIC: 2
- BUILD-CERTIFICATION: 1

## Hard P0/P1 targets

- **AIDENS-HA-20260508-0001** [P30-00 / P0 / Critical / BUILD-CERTIFICATION] No cargo/rustc execution was possible in this audit environment — `<audit environment>`
- **AIDENS-HA-20260508-0002** [P30-01 / P0 / Critical / PARSER-BOUNDARY] Malformed tool-call entries are silently discarded — `AiDENs/crates/aidens-runner/src/provider_tool.rs`
- **AIDENS-HA-20260508-0003** [P30-01 / P0 / Critical / PARSER-BOUNDARY] Tool-result serialization failure silently becomes an empty message — `AiDENs/crates/aidens-runner/src/provider_tool.rs`
- **AIDENS-HA-20260508-0004** [P30-01 / P0 / Critical / PARSER-BOUNDARY] Successful parser repair drops degradation reason codes from returned result — `AiDENs/crates/aidens-runner/src/provider_tool.rs`
- **AIDENS-HA-20260508-0005** [P30-01 / P0 / Critical / PARSER-BOUNDARY] Parser fallback uses permissive degraded JSON repair in the tool-call path — `AiDENs/crates/aidens-runner/src/provider_tool.rs`
- **AIDENS-HA-20260508-0006** [P30-02 / P0 / Critical / PATCH-SAFETY] Patch apply treats unreadable/missing files as empty input — `AiDENs/crates/aidens-tool-kit/src/lib.rs`
- **AIDENS-HA-20260508-0007** [P30-02 / P0 / Critical / PATCH-SAFETY] Rollback write errors are ignored — `AiDENs/crates/aidens-tool-kit/src/lib.rs`
- **AIDENS-HA-20260508-0008** [P30-03 / P0 / Critical / REPLAY-IDENTITY] Global process-local counter is available for artifact IDs — `AiDENs/crates/aidens-contracts/src/lib.rs`
- **AIDENS-HA-20260508-0009** [P30-03 / P0 / Critical / REPLAY-IDENTITY] Non-replay `generated_artifact_id` is public and easy to misuse — `AiDENs/crates/aidens-contracts/src/lib.rs`
- **AIDENS-HA-20260508-0010** [P30-00 / P0 / Critical / WORKSPACE-PORTABILITY] Nested AiDENs workspace depends on sibling crates outside AiDENs directory — `AiDENs/Cargo.toml`
- **AIDENS-HA-20260508-0011** [P30-04 / P0 / High / DEFAULTS] PlanActVerifyLoop defaults canonical receipt logging to None — `AiDENs/crates/aidens-runner/src/lib.rs`
- **AIDENS-HA-20260508-0012** [P30-00 / P0 / High / PACKAGE-INTEGRITY] Zip certifier passed packaging validation but did not prove semantic/build correctness — `AiDENs-aidens-next-codex-context-20260508.report.md`
- **AIDENS-HA-20260508-0013** [P30-00 / P0 / High / PACKAGE-INTEGRITY] Archive hash is explicitly zip-byte hash, not canonical content identity — `AiDENs-aidens-next-codex-context-20260508.report.md`
- **AIDENS-HA-20260508-0014** [P30-03 / P0 / High / REPLAY-IDENTITY] Tool exposure set has a constant artifact ID — `AiDENs/crates/aidens-tool-kit/src/lib.rs`
- **AIDENS-HA-20260508-0015** [P30-05 / P0 / High / VERIFICATION-SEMANTICS] Advisory-only checks can be represented as succeeded verification attempts — `AiDENs/crates/aidens-runner/src/lib.rs`
- **AIDENS-HA-20260508-0016** [P30-02 / P1 / High / COMMAND-SANDBOX] Command runner clears env then re-injects ambient PATH/CARGO_HOME/RUSTUP_HOME — `AiDENs/crates/aidens-tool-kit/src/lib.rs`
- **AIDENS-HA-20260508-0017** [P30-02 / P1 / High / COMMAND-SANDBOX] Timeout kills only the immediate child process — `AiDENs/crates/aidens-tool-kit/src/lib.rs`
- **AIDENS-HA-20260508-0018** [P30-04 / P1 / High / DEFAULTS] Default receipt level is Minimal — `AiDENs/crates/aidens-runner/src/lib.rs`
- **AIDENS-HA-20260508-0019** [P30-06 / P1 / High / DOC-DRIFT] Completion audit report hardcodes stale source basis — `AiDENs/crates/aidens-cli/src/package.rs`
- **AIDENS-HA-20260508-0020** [P30-04 / P1 / High / FAILURE-EVIDENCE] Failure control records can vanish when canonical receipts are absent — `AiDENs/crates/aidens-runner/src/lib.rs`
- **AIDENS-HA-20260508-0021** [P30-06 / P1 / High / GATE-DRIFT] Referenced/expected gate artifact is absent: schemas/artifact_envelope.schema.json — `AiDENs/schemas/artifact_envelope.schema.json`
- **AIDENS-HA-20260508-0022** [P30-06 / P1 / High / GATE-DRIFT] Referenced/expected gate artifact is absent: scripts/p24_verify.sh — `AiDENs/scripts/p24_verify.sh`
- **AIDENS-HA-20260508-0023** [P30-06 / P1 / High / GATE-DRIFT] Referenced/expected gate artifact is absent: scripts/p25_verify.sh — `AiDENs/scripts/p25_verify.sh`
- **AIDENS-HA-20260508-0024** [P30-06 / P1 / High / GATE-DRIFT] Referenced/expected gate artifact is absent: scripts/p26_verify.sh — `AiDENs/scripts/p26_verify.sh`
- **AIDENS-HA-20260508-0025** [P30-06 / P1 / High / GATE-DRIFT] Referenced/expected gate artifact is absent: scripts/p27_verify.sh — `AiDENs/scripts/p27_verify.sh`
- **AIDENS-HA-20260508-0026** [P30-06 / P1 / High / GATE-DRIFT] Referenced/expected gate artifact is absent: scripts/p28_verify.sh — `AiDENs/scripts/p28_verify.sh`
- **AIDENS-HA-20260508-0027** [P30-06 / P1 / High / GATE-DRIFT] Current verifier exists but does not replace missing historical phase gates by itself — `AiDENs/scripts/verify.sh`
- **AIDENS-HA-20260508-0028** [P30-06 / P1 / High / PACKAGE-HYGIENE] Root markdown archive is disabled while 134 ambiguous root markdown files remain — `AiDENs-aidens-next-codex-context-20260508.manifest.json`
- **AIDENS-HA-20260508-0029** [P30-01 / P1 / High / PARSER-BOUNDARY] Tool-call detection is substring-based — `AiDENs/crates/aidens-runner/src/provider_tool.rs`
- **AIDENS-HA-20260508-0030** [P30-02 / P1 / High / PERMIT-SCOPE] Permit/sandbox scope can fall back to wildcard — `AiDENs/crates/aidens-tool-kit/src/lib.rs`
- **AIDENS-HA-20260508-0031** [P30-06 / P1 / High / PROVIDER-CAPABILITY] Provider matrix exposes many provider kinds that are unavailable at execution — `AiDENs/crates/aidens-provider-kit/src/lib.rs`
- **AIDENS-HA-20260508-0032** [P30-03 / P1 / High / REPLAY-IDENTITY] Random UUID appears in identity path — `AiDENs/crates/aidens-agency-kit/src/lib.rs`
- **AIDENS-HA-20260508-0033** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-cli/src/agent.rs`
- **AIDENS-HA-20260508-0034** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/agent_bundle.rs`
- **AIDENS-HA-20260508-0035** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/agent_bundle.rs`
- **AIDENS-HA-20260508-0036** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/app_status.rs`
- **AIDENS-HA-20260508-0037** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/app_status.rs`
- **AIDENS-HA-20260508-0038** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/app_status.rs`
- **AIDENS-HA-20260508-0039** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/app_status.rs`
- **AIDENS-HA-20260508-0040** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/app_status.rs`
- **AIDENS-HA-20260508-0041** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/app_status.rs`
- **AIDENS-HA-20260508-0042** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/app_status.rs`
- **AIDENS-HA-20260508-0043** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/app_status.rs`
- **AIDENS-HA-20260508-0044** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/app_status.rs`
- **AIDENS-HA-20260508-0045** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0046** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0047** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0048** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0049** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0050** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0051** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0052** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0053** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0054** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0055** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0056** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0057** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0058** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0059** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0060** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0061** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0062** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0063** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0064** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/capability_turn.rs`
- **AIDENS-HA-20260508-0065** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/daemon_queue.rs`
- **AIDENS-HA-20260508-0066** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/daemon_queue.rs`
- **AIDENS-HA-20260508-0067** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/daemon_queue.rs`
- **AIDENS-HA-20260508-0068** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/lib.rs`
- **AIDENS-HA-20260508-0069** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/lib.rs`
- **AIDENS-HA-20260508-0070** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/lib.rs`
- **AIDENS-HA-20260508-0071** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/lib.rs`
- **AIDENS-HA-20260508-0072** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/lib.rs`
- **AIDENS-HA-20260508-0073** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/mechanism_display.rs`
- **AIDENS-HA-20260508-0074** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/mechanism_display.rs`
- **AIDENS-HA-20260508-0075** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/mechanism_display.rs`
- **AIDENS-HA-20260508-0076** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/mechanism_display.rs`
- **AIDENS-HA-20260508-0077** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/provider.rs`
- **AIDENS-HA-20260508-0078** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/provider.rs`
- **AIDENS-HA-20260508-0079** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/provider.rs`
- **AIDENS-HA-20260508-0080** [P30-03 / P1 / High / REPLAY-IDENTITY] Process-local generated_artifact_id used outside its definition — `AiDENs/crates/aidens-contracts/src/provider.rs`