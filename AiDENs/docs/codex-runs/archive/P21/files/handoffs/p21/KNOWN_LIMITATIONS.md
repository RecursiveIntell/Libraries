# P21 Known Limitations

## Supported

- `chat-only` profile: supported with provider required, no tools by default, agency enabled by config surfaces.
- `coding-agent` profile: supported with safe read/list/search/stat/propose defaults; write/admin tools remain permit-gated.
- `mock` provider: supported fixture provider; executable and tested, not cloud support.
- `run-test-agent`: supported operator command using the real runner path and emitting bundle artifacts.
- Generated `coding-agent` project: supported safe mock default; runnable with `aidens run`, `doctor`, `provider-check`, `tools inspect`, `plan validate`, and `plan compile`.
- Tool inspection: supported for declared, registered, executable, exposed, hidden, blocked, and permit-required truth.
- Agency v0.2 gate: supported at AiDENs output boundary when enabled; emits policy reports and receipts.
- Release archive replay: supported through `scripts/p21_verify_release_archive.sh`.

## Partial

- `memory-agent`: partial/proof-only. AiDENs delegates memory truth to canonical memory crates; profile/product wiring is not complete.
- `autonomous-daemon`: partial/safe-mode. Queue/schedule/wake, duplicate suppression, leases, safe mode, and drain are tested; no full daemon loop or desktop daemon UX is claimed.
- `ollama`: partial local chat boundary. It is not a native tool loop and requires a local service.
- Daemon smoke: partial operator proof over queue/schedule/wake, not a scheduler service.
- Recall/Recall-Coding extraction: partial pattern extraction only; no app behavior parity claim.
- Runtime memory views and optional memory: partial/degraded without explicit durable store configuration.

## Scaffold

- `aidens-profile-daemon`
- `aidens-profile-desktop`
- `aidens-profile-memory`
- `aidens-profile-research`

These crates remain scaffold/deferred surfaces and are reported as such by `doctor` and package readiness surfaces.

## Deferred

- OpenAI, OpenRouter, Anthropic, and generic OpenAI-compatible cloud execution.
- Provider-native tool loops.
- Streaming provider output.
- Full cloud provider suite.
- Full desktop daemon UX, IPC/socket server, recurring timer loop, and host wake wrapper.
- Multi-agent fanout.
- Federation, remote oracle admission, attested exchange, settlement, and mechanism search product flows.
- Full research workbench product surface.

## Quarantined

- Recall-specific DB schema, socket paths, Tauri/UI bridge, host wake wrappers, session state, and memory model.
- Recall-Coding `.recall-coding` data roots, hooks, checkpoint store, tool IDs, and app-specific agent manifest contracts.
- Local replacement semantics for canonical memory, evidence, kernel, repair, verification, federation, or mechanism truth.

## Failed

- No mandatory P21 gate is currently failed in the final audit run.
- One Phase 10 hostile-audit finding was repaired before final certification: `package examples` had classified Ollama examples as supported despite the local-service requirement. It now reports them as partial with explicit reason codes.

## Residual Risks

- The parent Git repository sees `AiDENs/` as an untracked directory, so file-change accounting in this run is based on handoffs and workspace inventories rather than Git diff metadata.
- `docs/p21/P21_RELEASE_AND_AUDIT_REQUIREMENTS.md` lists older descriptive handoff filenames for some phases, while the actual phase protocol and release verifier use `PHASE_NN_REPORT.md` plus `PHASE_01_BUILD_CERTIFICATION.md`. This is disclosed; canonical phase handoffs are present through Phase 10.
- Target logs are generated artifacts under `target/p21/`; they must be preserved with the release/audit bundle even though they are not normal source files.
