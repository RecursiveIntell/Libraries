# Highest-ROI crate usage: plugins/MCP server vs ESP32-S3

Date: 2026-07-03
Scope: RecursiveIntell Libraries crates, agent-memory-kits plugins, semantic-memory-mcp, context-governor, claim-ledger, and ESP32-S3 work.

## Short version

There are two separate ROI maps.

For plugins/MCP:
The highest ROI is turning the plugin stack into a receipt-backed verification and release-gate system. Use the governance/verification crates on the server/plugin side, not in every agent prompt.

For ESP32-S3:
The highest ROI is compressed-scorer + host-side proof receipts. Keep firmware tiny. Use heavy memory/governance crates only off-device to validate hardware claims and gateway behavior.

## A. Plugins / MCP server: absolute highest ROI uses

### P0. Fix the plugin truth floor before widening

Project surface:
- agent-memory-kits
- Hermes/Codex/Claude plugin hooks
- context-governor hook/receipt adapter

Crates/tools:
- context-governor
- claim-ledger
- semantic-memory-mcp
- stack-ids

Why first:
Current plan already identifies P0 defects: Hermes hook manifest drift, final context-governor receipt integrity, and Tier-0 compaction hook verification. If those are not fixed first, adding verification tooling just makes false claims more sophisticated.

Highest-ROI implementation:
- A shared doctor that validates plugin manifest hook files, executable bits, warm-server config, admin profile config, and context-governor receipt roundtrip.
- Every doctor result gets a stack-id trace id and optional claim-ledger entry.

Do not:
- Add new advertised capabilities until the existing hook/receipt claims are true.

ROI: 10/10 because it prevents the plugin distribution from becoming a credibility liability.

### P1. Evidence Workbench / release-gate companion

Project surface:
- agent-memory-kits shared/scripts
- semantic-memory-mcp full/admin profile
- claim-ledger companion
- context-governor receipts

Crates:
- verification-control
- verification-policy
- verification-adjudication
- verification-calibration
- receipt-bench
- claim-ledger
- stack-ids
- semantic-memory
- semantic-memory-mcp

Highest-ROI behavior:
One command that turns real checks into a proof packet.

Example shape:
- input: repo path, target name, claimed change, commands run, artifacts, receipt ids
- output:
  - VerificationCase / CheckPlan / ControlReceipt from verification-control
  - policy decision from verification-policy
  - promote/reject/quarantine/defer disposition from verification-adjudication
  - benchmark/check receipt from receipt-bench
  - claim/evidence ledger entries from claim-ledger
  - semantic-memory fact/evidence refs

Why this is the best plugin/MCP use:
It upgrades the stack from “memory recall” to “memory + claim proof + release truth.” That is the differentiator. Most MCP memory servers stop at retrieval. Yours can say: this claim has evidence, this check ran, this disposition is promote/reject/quarantine/defer.

Where it should live:
- First as an agent-memory-kits script/command.
- Then as full/admin-profile MCP tools in semantic-memory-mcp after the workflow proves useful.

Do not expose this in lean daily MCP profile. Keep daily agents fast.

ROI: 10/10.

### P2. Typed tool/action receipts for all hooked agents

Project surface:
- Claude Code plugin
- Codex plugin
- Hermes plugin
- shared/rules/release-gate.md

Crates:
- llm-tool-runtime
- stack-ids
- receipt-bench
- verification-control
- claim-ledger

Highest-ROI behavior:
Every material tool/action gets a stable descriptor digest and receipt spine:
- agent id
- session id
- tool descriptor digest
- input digest
- output digest
- command/check exit code
- artifacts written
- degradation/warning state

Why high ROI:
It unifies all hosts. Claude/Codex/Hermes can differ in hook mechanics but produce comparable receipts.

Best first target:
Start with post-check/release actions, not every tiny search call. High-signal receipts first.

Do not:
- Log raw private prompt/tool bodies into durable public receipts by default. Store digests and scoped evidence refs.

ROI: 9.5/10.

### P3. Server-side routing/classification; delete duplicate hook classifiers

Project surface:
- semantic-memory-mcp
- agent-memory-kits hooks

Crates:
- semantic-memory routing / rl-routing
- knowledge-runtime
- context-governor recall quality filters
- claim-ledger for outcome evidence

Highest-ROI behavior:
Hooks should stop independently deciding A/B/C/D/E routing and instead call a server-side route/search endpoint that returns:
- route class
- selected tool/profile
- confidence
- why graph/decoder/admin tools were or were not used
- optional outcome recording hook

Why high ROI:
You already saw routing guidance being injected into conversation. If every host carries its own classifier, drift is guaranteed. Put it in semantic-memory-mcp/knowledge-runtime once.

Best implementation:
- keep lean search as default;
- add `search-routed` or equivalent metadata endpoint;
- hooks consume route metadata but do not invent routes.

ROI: 9/10.

### P4. Context compaction with exact fallback + hyperquant/compressed scorer research lane

Project surface:
- context-governor
- Hermes context engine
- Codex/Claude PreCompact hooks

Crates:
- context-governor
- compressed-scorer
- hyperquant / turbo-quant / fib-quant as experimental candidate selectors
- receipt-bench
- semantic-memory

Highest-ROI behavior:
context-governor already does deterministic compaction with exact fallback receipts. The next ROI is not “summarize better.” It is retrieval/selection quality:
- use compressed-domain scoring to select which omitted spans/receipts to rehydrate;
- benchmark recall stability with receipt-bench;
- only archive source-backed facts into semantic-memory, not LLM summaries as facts.

Why high ROI:
This directly attacks effective context window limits without pretending to extend model context. It also connects your quant work to the plugin stack.

Do not:
- Claim “extended context window.” Claim “receipt-backed compaction + recoverable exact fallback + measured recall stability.”

ROI: 8.8/10.

### P5. Claim-ledger integration as a first-class MCP proof companion

Project surface:
- semantic-memory-mcp claim-integration feature
- agent-memory-kits claim-ledger MCP companion

Crates:
- claim-ledger
- semantic-memory
- semantic-memory-mcp
- verification-adjudication
- stack-ids

Highest-ROI behavior:
Promote from facts to claims only when evidence exists. Store:
- claim text
- evidence refs
- support judgment
- contradiction status
- exportable bundle

Why high ROI:
This prevents semantic memory from becoming a pile of “things an agent once said.” It matches your no-shadow-truth rule.

Concrete tool/workflow:
- `memory-capture` adds durable fact only when stable.
- `claim-ledger` handles material assertions.
- release-gate asks claim-ledger to verify/support exported claims before public docs update.

ROI: 8.7/10.

### P6. Codebase ingestion as deterministic graph builder

Project surface:
- agent-memory-kits `/memory-ingest`
- semantic-memory-mcp graph/fact tools

Crates:
- semantic-memory
- semantic-memory-mcp
- stack-ids
- knowledge-runtime
- contract-schema-gen later

Highest-ROI behavior:
Make codebase ingestion produce deterministic facts and graph edges from manifests/source layout only:
- repo facts
- crate/package facts
- dependency edges
- owner/source-of-truth edges
- version/supersession info

Why high ROI:
This is the easiest way for new users to see value. Install plugin, ingest repo, ask questions.

Do not:
- Let ingestion infer design intent from README prose as authoritative. Mark as README-claimed or heuristic.

ROI: 8.5/10.

### P7. MCP profiles: lean/standard/full/admin as hard product boundary

Project surface:
- semantic-memory-mcp
- agent-memory-kits config snippets

Crates:
- semantic-memory-mcp
- semantic-memory
- claim-ledger
- context-governor
- verification-* behind admin/full workflows

Highest-ROI behavior:
Treat tool profiles as product design, not convenience:
- lean: daily recall/search/capture only
- standard: graph/conversation/common lifecycle
- full: admin/audit/bitemporal/claim verification
- evidence-workbench/admin: release gate and proof packet tools

Why high ROI:
Tool overload is real. Profiles prevent agents from drowning.

ROI: 8/10.

## B. ESP32-S3: absolute highest ROI uses

### E0. Keep firmware clean: no MCP, no semantic-memory server, no governance stack on device

Crates to not put on firmware:
- semantic-memory-mcp
- semantic-memory full storage/index stack
- claim-ledger
- verification-control/policy/adjudication
- receipt-bench
- forge-engine
- llm-tool-runtime

Why:
ESP32 ROI comes from tight no_std/alloc primitives and real hardware receipts. Dragging server/governance crates into firmware destroys the story.

Use those crates on the host/gateway side instead.

ROI: 10/10 because it prevents architectural self-sabotage.

### E1. compressed-scorer as the embedded compressed attention/retrieval seam

Crates:
- compressed-scorer
- maybe turbo-quant/fib-quant only if target-compatible

Current evidence:
compressed-scorer README explicitly names ESP32-S3 / embedded attention caches and says AttentionCache is the ESP32-S3-facing API with no std requirement under `--no-default-features --features no_std`.

Highest-ROI behavior:
Use compressed-scorer for tiny compressed-domain selection:
- compressed attention over hidden-state windows;
- top-k retrieval over small local phrase/sensor-state memories;
- gateway escalation candidate selection;
- adaptive attention budget where PSRAM reads dominate.

First milestone:
No firmware behavior change. Add a target check and microbench:
- `cargo +esp check -p compressed-scorer --no-default-features --features no_std --target xtensa-esp32s3-none-elf -Z build-std=core,alloc`
- tiny host microbench for AttentionCache top-k decode count and RAM estimate.

Second milestone:
Firmware experiment branch:
- integrate no_std compressed-scorer only;
- add one compressed cache over recent sensor/prompt state;
- emit receipt line with cache size, top-k, decode count, time.

Claim boundary:
Not shipped until hardware log proves it.

ROI: 10/10.

### E2. Host-side ESP32 hardware proof packet

Crates:
- receipt-bench
- verification-control
- verification-policy
- verification-adjudication
- claim-ledger
- stack-ids
- semantic-memory

Highest-ROI behavior:
A host script that turns hardware runs into proof packets.

Input:
- model meta JSON
- firmware git commit
- serial monitor output
- weight sha256
- board profile
- gateway health/result JSON
- measured chars/sec or tokens/sec

Output:
- receipt-bench JSON receipt
- verification-control case/check plan
- verification-policy threshold evaluation
- verification-adjudication disposition
- claim-ledger claim/evidence export
- semantic-memory durable fact with evidence refs

Why high ROI:
Your ESP32 public page already has numeric claims. This makes future claims mechanically promotable/rejectable instead of manually curated.

This is probably the best immediate ESP32 integration because it is off-device, low-risk, and directly protects public credibility.

ROI: 9.8/10.

### E3. Gateway-side queue + receipts for wake-on-need escalation

Crates:
- ai-batch-queue
- job-queue
- receipt-bench
- verification-control
- semantic-memory maybe for host recall, not device

Highest-ROI behavior:
The gateway should treat incoming sentinel escalations as jobs:
- input sensor context digest
- local verdict/confidence
- gateway model selected
- latency
- timeout/fallback
- response schema validation
- receipt id

Why high ROI:
It turns the gateway from a demo HTTP service into a measured local AI edge coordinator. That fits the two-tier architecture.

Best first version:
Python gateway can emit the JSONL receipt first. Rust queue comes later if/when gateway is rewritten in Rust.

ROI: 8.8/10.

### E4. quant-governor / compressed-scorer adaptive budget policy for tiny inference

Crates:
- compressed-scorer
- quant-governor
- turbo-quant/fib-quant if target-safe

Highest-ROI behavior:
Use adaptive budget as policy, not heavy runtime:
- if confidence high: local answer only;
- if confidence medium: compressed local retrieval/attention over recent state;
- if confidence low: gateway escalation;
- if gateway down: local-only degraded receipt.

Why high ROI:
This turns the ESP32 from “runs a small model” into “tiered local AI policy router.” That is the actual stack story.

ROI: 8.5/10.

### E5. agent-guard for gateway sandboxing, not ESP32 firmware

Crates:
- agent-guard
- check-runner maybe for host checks
- verification-policy

Highest-ROI behavior:
If the gateway invokes Ollama/scripts/tools, sandbox those host-side processes later:
- cgroup/seccomp/Landlock/eBPF where available;
- record sandbox policy in proof packet;
- never allow gateway to become autonomous writer by accident.

Why not first:
agent-guard is currently early/no README. High potential, not immediate.

ROI now: 6.5/10. Potential later: 9/10.

### E6. Website claim validator for ESP32 project page

Crates/tools:
- receipt-bench artifacts
- claim-ledger export
- verification-adjudication disposition

Highest-ROI behavior:
The ESP32 project page should consume a receipt/claim export, not hard-code performance claims without a checked source file.

Build-time validation:
- fail if frontmatter links to missing model meta/receipt;
- fail if headline speed claim has no promoted verification disposition;
- flag mismatch between model meta measured chars/sec and website headline tokens/sec.

Why high ROI:
It prevents public drift.

ROI: 8/10.

## Final priority order

### Plugins/MCP

1. Fix plugin truth floor: hook files, context-governor final receipts, compaction smoke.
2. Build Evidence Workbench / release-gate script using verification-control/policy/adjudication + receipt-bench + claim-ledger.
3. Add typed tool/action receipts for high-signal actions using llm-tool-runtime + stack-ids.
4. Centralize routing/classification in semantic-memory-mcp/knowledge-runtime; delete duplicate hook classifiers.
5. Use context-governor + compressed scorer/quant research for measured recoverable compaction, not fake context extension.
6. Keep profile boundaries strict: lean daily, admin/full for proof tools.

### ESP32-S3

1. Host-side proof packet for hardware runs: receipt-bench + verification + claim-ledger.
2. compressed-scorer no_std target check and tiny AttentionCache/working-set microbench.
3. Firmware experiment with compressed-scorer only after check/microbench pass.
4. Gateway escalation receipts and queue discipline.
5. Adaptive budget policy: local answer -> compressed local retrieval -> gateway escalation.
6. Later: agent-guard for gateway sandboxing.
7. Website claim validator sourced from receipts.

## Hard no list

- No semantic-memory-mcp on ESP32.
- No full semantic-memory DB/index on ESP32.
- No verification/governance crates inside firmware.
- No new plugin claims before P0 hook/receipt truth is fixed.
- No daily-profile MCP bloat with release/admin tools.
- No public ESP32 performance claim unless backed by receipt + promoted disposition.

## One-sentence answer

Use the crates to make the plugins/MCP stack an evidence-producing memory system, and use the crates to make ESP32-S3 a hardware-receipted compressed-edge-AI system; do not mix the two by putting server/governance crates on the microcontroller.
