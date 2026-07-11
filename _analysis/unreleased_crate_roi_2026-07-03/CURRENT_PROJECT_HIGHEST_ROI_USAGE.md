# Highest-ROI current-project uses for the Libraries crate stack

Date: 2026-07-03
Scope: current projects under /home/sikmindz/Coding plus the public ESP32-S3 sentinel repo cloned from GitHub for inspection.

## Evidence checked

Local/project evidence:

- /home/sikmindz/Coding/Libraries/_analysis/unreleased_crate_roi_2026-07-03/ROI_REPORT.md
- /home/sikmindz/Coding/Libraries/_analysis/unreleased_crate_roi_2026-07-03/unreleased_inventory.json
- /home/sikmindz/Coding/agent-memory-kits/docs/plans/2026-07-03-plugin-stack-roi-implementation-plan.md
- /home/sikmindz/Coding/Libraries/semantic-memory-mcp/Cargo.toml
- /home/sikmindz/Coding/Libraries/compressed-scorer/README.md
- /home/sikmindz/Coding/Libraries/context-governor/README.md
- /home/sikmindz/Coding/Recall-Coding/Cargo.toml
- /home/sikmindz/Coding/Recall-Coding/recall-session/Cargo.toml
- /home/sikmindz/Coding/Rivot/Cargo.toml and AGENTS/README
- /home/sikmindz/Coding/forge-workbench/Cargo.toml and AGENTS
- /home/sikmindz/Coding/chat-rs/Cargo.toml and AGENTS
- /home/sikmindz/Coding/MiniRecall/Cargo.toml and AGENTS
- /home/sikmindz/Coding/recursiveintell-web/content/projects/esp32-sentinel.mdx
- /tmp/esp32-sentinel-audit cloned from https://github.com/RecursiveIntell/esp32-sentinel

Semantic-memory facts checked:

- agent-memory-kits currently has 9 host plugins and 3 companions: semantic-memory-mcp, context-governor, claim-ledger.
- semantic-memory-mcp is the primary memory backend; hooks exist for Claude Code, Codex, Hermes.
- compressed-scorer use catalog already identified ESP32-S3 attention/KV and semantic-memory acceleration use cases.
- Libraries is the central monorepo; Gloss/Recall/Forge/Rivot/projmind/etc. depend on it by sibling path or vendored snapshots.

## Executive answer

The absolute highest ROI is not publishing more crates first.

The absolute highest ROI is to use the crates as control planes in the projects that already need them:

1. Agent-memory-kits + semantic-memory-mcp: build the Evidence Workbench / release-gate using verification-control, verification-policy, verification-adjudication, receipt-bench, claim-ledger, and stack-ids.
2. ESP32-S3 sentinel: use compressed-scorer as the embedded no_std attention/retrieval compression seam; use receipt-bench/verification crates on the host side for hardware proof packets.
3. Forge Workbench / Rivot / Recall-Coding: turn Forge crates into the verify->apply integrity spine: typed-patch, sandbox-workspace, check-runner, forge-engine, cea-core, verification-control/policy.
4. Recall-Coding: it already depends on the right crates; highest ROI is closing execution truth and scheduler/control receipts, not adding more crates.
5. MiniRecall: do not pull in the heavy stack; add only tiny/mobile-safe receipts and maybe compressed-scorer-style primitives later. Keep semantic-memory brute-force policy.
6. RI-Chat/chat-rs: use llm-tool-runtime + semantic-memory + stack-ids as the provider/tool/memory receipt spine.
7. Public site: use the generated receipts to update project pages; do not hand-write inflated claims.

## Ranked ROI list

### 1. Agent-memory-kits Evidence Workbench / release gate

Project:
- /home/sikmindz/Coding/agent-memory-kits
- /home/sikmindz/Coding/Libraries/semantic-memory-mcp
- /home/sikmindz/Coding/Libraries/context-governor
- /home/sikmindz/Coding/Libraries/claim-ledger

Crates to use:
- verification-control
- verification-policy
- verification-adjudication
- verification-calibration
- receipt-bench
- stack-ids
- claim-ledger
- semantic-memory-mcp
- context-governor
- llm-tool-runtime later for typed tool receipts

Why this is highest ROI:
Agent-memory-kits is already a public-facing distribution surface for the memory stack. Its README claims persistent memory, receipt-backed compaction, and claim/evidence provenance. The existing 2026-07-03 plan already identifies P2.1: “Evidence Workbench/release-gate” and P2.2: “Typed tool/action receipts.” That is exactly where the verification crates belong.

Current gap:
The plan says P0 still comes first:
- Hermes hook manifest/files drift.
- context-governor final emitted receipt integrity.
- Tier-0 transcript compaction hook verification.

After P0, the Evidence Workbench becomes the cleanest high-ROI integration.

Best first implementation:
Create a release-gate command/script in agent-memory-kits that emits one proof packet per release candidate:

Input:
- git repo path
- crate/package name
- commands actually run
- context-governor receipt IDs
- semantic-memory fact/evidence refs
- claim-ledger claim IDs

Output:
- Verification case from verification-control
- Policy evaluation from verification-policy
- Adjudication result from verification-adjudication
- Benchmark/check receipt from receipt-bench
- Claim/evidence bundle in claim-ledger
- Durable source-backed summary in semantic-memory

Immediate win:
This turns “receipts or it didn’t happen” into a reusable product feature for every agent host, not just a slogan.

Do not do first:
Do not add more local hook classifiers or another truth plane. The plan already warns against duplicate hook classifiers and prefers server-side routing/classification.

ROI score: 10/10.

### 2. ESP32-S3 sentinel compressed attention / retrieval primitive

Project:
- public repo cloned to /tmp/esp32-sentinel-audit
- project page: /home/sikmindz/Coding/recursiveintell-web/content/projects/esp32-sentinel.mdx

Current evidence:
- ESP32-S3 repo ships Rust no_std firmware under esp32-s3/.
- Model metadata says 6.34M params, H512, 3 layers, 4.8 MB int8+int4 weight file.
- Hardware verification fields show ESP32-S3 Freenove WROOM N8R8, 8 MB PSRAM, measured 10.57 chars/sec in metadata; website says 11.6 tok/s.
- README says receipt-backed, worked example writes JSONL receipt.
- compressed-scorer README explicitly says the ESP32-S3-facing API is AttentionCache with no std requirement when built with --no-default-features --features no_std.

Crates to use:
- compressed-scorer
- turbo-quant or fib-quant only if the selected feature actually builds for Xtensa/no_std
- receipt-bench on host side, not firmware side
- verification-control/policy/adjudication on host side for hardware proof promotion
- agent-guard later for gateway sandboxing, not firmware

Highest-ROI use:
Do not put semantic-memory-mcp on the ESP32. Do not put full receipt-bench on the ESP32. Do not drag governance crates into firmware.

Use compressed-scorer as a tiny embedded scoring/attention seam:

1. Firmware side:
   - add an experimental feature branch that links compressed-scorer with no_std only;
   - build a tiny compressed working set or AttentionCache over sensor/state embeddings or small hidden-state windows;
   - score compressed keys directly and decode only top-k values;
   - emit only compact local decision metadata.

2. Gateway/host side:
   - convert serial/WiFi hardware logs into receipt-bench receipts;
   - use verification-control/policy/adjudication to decide whether a run may update public claims;
   - store run facts in semantic-memory and claim-ledger.

Why this is huge:
The ESP32 project is already public and hardware-verified. compressed-scorer gives it a second differentiator: not just “small LSTM on ESP32,” but “compressed-domain attention/retrieval policy for tiny local agents.” That ties the embedded work back to the memory/quant stack without bloating firmware.

First concrete milestone:
A host-side receipt generator:
- parse ESP32 serial output and model meta;
- verify weight sha256;
- record board, model, prompt, chars/sec, gateway status, threshold, build commit;
- emit receipt-bench JSON;
- create a verification-control case with disposition “promote” only if thresholds pass.

Second milestone:
A no_std check gate:
- cargo +esp check compressed-scorer no_std for xtensa target in CI or local script;
- add it as an ESP32 optional integration receipt.

Claim boundary:
Until the compressed-scorer path runs on hardware, say “planned/experimental compressed attention seam,” not shipped.

ROI score: 9.5/10.

### 3. Forge Workbench verify->apply integrity closure

Project:
- /home/sikmindz/Coding/forge-workbench

Current evidence:
- Cargo.toml already depends on forge-engine, typed-patch, ai-batch-queue, tauri-queue, semantic-memory, semantic-memory-forge, stack-ids.
- AGENTS says the remaining work is truth, integrity, packaging, transactional guarantees, verify->apply integrity, audit/export/import closure, queue resilience.

Crates to use more deeply:
- typed-patch
- sandbox-workspace
- check-runner
- forge-engine
- cea-core
- receipt-bench
- verification-control
- verification-policy
- verification-adjudication
- ai-batch-queue
- tauri-queue

Highest-ROI use:
Turn Forge Workbench into the GUI/workbench for the Forge spine.

The invariant:
A patch verified against repo state X must not be applied to repo state Y.

Best first implementation:
- use typed-patch as the only candidate patch object;
- use sandbox-workspace for staging;
- use check-runner for verification;
- use forge-engine to score/persist evidence;
- use cea-core to attribute failures/successes to patch regions;
- use verification-control to record the case/check plan/control receipt;
- use verification-policy to enforce approval/apply policy;
- use tauri-queue/ai-batch-queue to make the UI queue durable and visible.

Why this is high ROI:
This project is literally a repair workbench. The crates match its AGENTS file perfectly. The missing value is not invention; it is closing the integrity spine and making the app impossible to lie about.

ROI score: 9/10.

### 4. Rivot operator shell: replace temp adapters with stack-backed seams

Project:
- /home/sikmindz/Coding/Rivot

Current evidence:
- Cargo.toml already depends on knowledge-runtime, llm-tool-runtime, stack-ids, verification-control, verification-policy.
- AGENTS says Rivet/Rivot is the application layer over the canonical stack, not a second truth plane.
- Required order says replace remaining temp-local policy/router adapters, require real backpointers on real backend paths, define real codebase pilot bar.

Crates to use:
- verification-control
- verification-policy
- llm-tool-runtime
- knowledge-runtime
- stack-ids
- typed-patch/check-runner/sandbox-workspace later if Rivot owns mutation paths

Highest-ROI use:
Rivot should become the truthful CLI/operator shell over the stack, not another runtime.

Best first implementation:
- route all tool invocation through llm-tool-runtime descriptors/receipts;
- make verification-control the source for control cases/dispatch decisions;
- make verification-policy the only approval policy evaluator;
- use stack-ids for immutable request/apply identities;
- populate canonical_backpointers for every stack-backed artifact.

Why this is high ROI:
Rivot’s hard laws match your doctrine exactly: no shadow truth plane, no summary-bound approvals, no stale turn/plan reuse. The verification crates are built for this.

ROI score: 8.8/10.

### 5. Recall-Coding execution truth and scheduler/control receipts

Project:
- /home/sikmindz/Coding/Recall-Coding

Current evidence:
- Workspace already depends on cea-core, forge-engine, forge-pilot, profile-runtime, verification-control, verification-policy, llm-tool-runtime, check-runner, sandbox-workspace, typed-patch, semantic-memory, stack-ids.
- AGENTS says active finish line is: stop duplicate execution, stop false-ready state, preserve operator control, then harden GUI truth surfaces.
- recall-session already depends on verification-control, verification-policy, forge-pilot optional, profile-runtime, llm-tool-runtime.

Crates to use:
- verification-control
- verification-policy
- profile-runtime
- llm-tool-runtime
- typed-patch
- sandbox-workspace
- check-runner
- cea-core
- forge-engine

Highest-ROI use:
Do not add a new subsystem. The dependencies are already there. The ROI is in forcing every scheduler/tool/execution state transition to emit stable control receipts.

Best first implementation:
- bind recurring/future actions to verification-control cases;
- use verification-policy for scheduleability and approval rules;
- use llm-tool-runtime descriptor digests for tool identity;
- ensure duplicate execution/idempotency failures are represented as verification dispositions, not ad hoc logs;
- expose degraded/false-ready state in GUI from the same backend receipt state.

Why high ROI:
Recall-Coding already has the crate wiring. If you finish the receipt/control path, it becomes a major proof that the stack can govern real local apps.

ROI score: 8.7/10.

### 6. semantic-memory-mcp full tool surface: typed receipts and claim verification

Project:
- /home/sikmindz/Coding/Libraries/semantic-memory-mcp

Current evidence:
- Cargo.toml v0.4.0 exposes features for claim-integration, llm-parser, orchestration.
- Full feature includes semantic-memory provenance/temporal/multiscale/discord/decoder/subtraction/compression-governor/routing/integration/admin/late-interaction/turbo-quant-codec/rl-routing plus claim integration, parser, orchestration.
- semantic-memory-mcp currently depends on stack-ids, claim-ledger optional, llm-output-parser optional, knowledge-runtime optional, boundary-compiler.

Crates to use:
- verification-control
- verification-policy
- verification-adjudication
- receipt-bench
- llm-tool-runtime
- profile-runtime maybe later

Highest-ROI use:
Add typed “verification packet” endpoints/tools rather than more bare memory tools.

Potential new MCP tools:
- sm_open_verification_case
- sm_attach_evidence_to_case
- sm_evaluate_verification_policy
- sm_adjudicate_claim_or_release
- sm_emit_release_gate_packet
- sm_replay_receipt_bench

But keep lean/standard/full profiles strict. Daily agents should not see every admin/release tool.

Why high ROI:
semantic-memory-mcp is the distribution point. If it can emit verification packets, every host plugin gets proof discipline without bespoke glue.

Risk:
Tool overload. The MCP already has many tools; adding more to the daily profile would hurt agent performance. Put these in full/admin profile or Evidence Workbench only.

ROI score: 8.5/10.

### 7. RI-Chat/chat-rs provider/tool/memory receipts

Project:
- /home/sikmindz/Coding/chat-rs

Current evidence:
- Cargo.toml depends on semantic-memory, semantic-memory-forge, llm-tool-runtime, stack-ids, optional turbo-quant, optional knowledge-runtime.
- AGENTS says do not reimplement canonical library semantics locally; app-local modules may be adapters only.

Crates to use:
- llm-tool-runtime
- stack-ids
- semantic-memory
- semantic-memory-forge
- knowledge-runtime
- verification-control later for release gates

Highest-ROI use:
Make RI-Chat the clean minimal demo of RecursiveIntell-native chat:
- provider invocation receipt;
- tool descriptor digest;
- semantic-memory-backed persistence;
- stack-id trace spine;
- optional turbo-quant memory acceleration.

Why high ROI:
It is a contained app where the stack can be shown without the complexity of Recall/Forge.

ROI score: 7.8/10.

### 8. MiniRecall mobile: stay lightweight, add only mobile-safe proof seams

Project:
- /home/sikmindz/Coding/MiniRecall

Current evidence:
- AGENTS explicitly says do not vendor full Libraries, do not copy from Gloss/Recall/ClaimLedger, and semantic-memory must use default-features=false with brute-force only.

Crates to use:
- semantic-memory with brute-force only: already specified
- stack-ids tiny ID/digest surfaces if mobile-safe
- compressed-scorer later only if it passes Android target/no_std/alloc constraints and has a clear mobile search benefit

Crates not to pull into mobile core:
- forge-engine
- forge-pilot
- semantic-memory-mcp
- full verification/governance stack
- usearch/hnsw/server storage dependencies

Highest-ROI use:
Keep MiniRecall as the constraint proof: the stack can be reduced to a small mobile-safe core without dragging server/agent baggage.

Best first implementation:
Host-side release gate for MiniRecall builds, not in-app governance crates. Use verification/receipt crates outside the app to certify APK/core behavior.

ROI score: 7.5/10.

### 9. Public RecursiveIntell website: claim display from receipts

Project:
- /home/sikmindz/Coding/recursiveintell-web

Current evidence:
- ESP32 project page already makes specific performance claims.
- Site AGENTS says content is source of truth and frontmatter must fail fast; public positioning law forbids unsupported superiority/production claims.

Crates/tools to use:
- receipt-bench generated artifacts
- claim-ledger exported claim/evidence bundles
- verification-adjudication disposition
- semantic-memory summaries only as private drafting aid, not public authority

Highest-ROI use:
Automate project-page claim receipts. The website should not manually drift from repo receipts.

Best first implementation:
For each project page with performance claims:
- reference a receipt file or claim-ledger export;
- add build-time validation that required receipt fields exist;
- fail if headline numeric claims lack a source basis.

Why high ROI:
It protects public credibility immediately.

ROI score: 7.3/10.

### 10. Gloss and desktop media apps: practical queue/vision crates

Projects:
- /home/sikmindz/Coding/Gloss
- /home/sikmindz/Coding/Kirsten
- /home/sikmindz/Coding/visionforge

Crates to use:
- ai-batch-queue
- tauri-queue
- ollama-vision
- comfyui-rs
- receipt-bench for job receipts

Highest-ROI use:
For desktop apps, the practical crates may beat the doctrine crates:
- ai-batch-queue for embedding/image/caption/batch jobs;
- tauri-queue for visible frontend job progress;
- ollama-vision for local tagging/captioning;
- comfyui-rs for local image-generation workflows.

Why not top 5:
These are useful and public-download-friendly, but they do not solve the highest-value trust/control problem across the stack.

ROI score: 7/10.

## Cross-project implementation order

### Phase 0: Do not widen before P0 plugin truth is fixed

Finish agent-memory-kits P0 first:
- Hermes hook file/manifest drift.
- context-governor final receipt integrity.
- Tier-0 transcript compaction smoke.

Reason:
If the plugin stack has false Tier-0 claims, adding Evidence Workbench on top makes the lie larger.

### Phase 1: Evidence Workbench / release gate

Implement in agent-memory-kits as the stack distribution point.

Crates:
- verification-control
- verification-policy
- verification-adjudication
- receipt-bench
- claim-ledger
- stack-ids

Deliverable:
A command/script that produces a release proof packet for a repo/crate.

### Phase 2: ESP32 host-side hardware proof packet

Use the release-gate machinery on esp32-sentinel:
- parse model meta;
- verify sha256;
- parse serial/hardware run logs;
- emit receipt-bench JSON;
- adjudicate whether public claims can be promoted.

Do not touch firmware first.

### Phase 3: Forge Workbench integrity spine

Close verify->apply invariants using typed-patch, sandbox-workspace, check-runner, forge-engine, cea-core, and verification-control/policy.

### Phase 4: Rivot/Recall-Coding control receipt cleanup

Rivot:
- replace temp adapters;
- require backpointers;
- llm-tool-runtime/verification-control as source of truth.

Recall-Coding:
- scheduler/tool execution receipt identity;
- false-ready and duplicate-execution closure.

### Phase 5: Practical app crates

Publish/use ai-batch-queue, tauri-queue, ollama-vision, comfyui-rs where they fit desktop/local media flows.

## Blunt keep/kill per proposed use

Keep / do now:
- agent-memory-kits Evidence Workbench after P0.
- ESP32 host-side proof packet.
- compressed-scorer no_std experiment for ESP32 compressed attention/retrieval.
- Forge Workbench verify->apply integrity spine.
- Rivot canonical stack-backed control seams.
- Recall-Coding execution/scheduler receipt closure.

Do later:
- semantic-memory-mcp admin/full verification tools after Evidence Workbench API shape is proven.
- MiniRecall stack expansion beyond semantic-memory brute-force.
- agent-guard sandboxing of gateway/agent processes.

Do not do:
- Put semantic-memory-mcp on ESP32.
- Put full verification/governance crates inside firmware/mobile core.
- Add more agent-memory hook classifiers.
- Create another local truth plane in Rivot/Recall/Forge.
- Publish new public claims from website copy without receipt files.

## Highest ROI single next action

Implement agent-memory-kits P0, then immediately build the Evidence Workbench/release-gate proof packet.

Why:
That one path benefits every current project:
- semantic-memory-mcp and agent plugins get truthful proof gates;
- ESP32 public claims can be promoted only from hardware receipts;
- Forge/Rivot/Recall can reuse the same verification/control/adjudication objects;
- website claims get a receipt source;
- future crate releases become evidence-backed instead of README-backed.

If only one integration gets done, do that.
