# Completion Pass Map

| Pass | Title | Phase | Primary exit gate |
|---|---|---|---|
| P00 | Source-basis lock, fake-ready freeze, and repo hygiene gate | Foundation / honesty | bash scripts/verify.sh exists and is referenced by README, AGENTS.md, and CI. |
| P01 | Public API honesty, no-op removal, and plan/runtime parity | Foundation / honesty | No public method accepts meaningful input and silently discards it. |
| P02 | Provider runtime truth and executable backend matrix | Foundation / runtime | provider-check never reports native_tool_loop=true for an unavailable or unsupported backend. |
| P03 | TurnExecutorV1, provider/tool loop, and budgeted dispatch | Foundation / runtime | A mock/provider fixture can request repo-read and receive the tool result before final answer. |
| P04 | Capability gate, permits, approvals, and side-effect denial law | Foundation / safety | list-tools and inspect-tools distinguish declared, registered, executable, exposed, hidden, blocked. |
| P05 | Durable execution evidence ledger, receipt store, and transactional outbox | Foundation / evidence | A run can be inspected after process restart via durable store. |
| P06 | Boundary compiler, schema validation, repair provenance, and canonical digests | Contracted core | Duplicate-key fixture is rejected with DuplicateKeyFindingV1. |
| P07 | Schema generation, artifact registry, compatibility, and migration law | Contracted core | cargo run -p aidens-cli -- schemas generate creates deterministic schema files. |
| P08 | Reference interpreters and semantic conformance harness | Contracted core | Reference fixtures cover all provider kinds, risk classes, memory modes, receipt levels, and tool lifecycle states. |
| P09 | Episode-first memory and bitemporal evidence store | Memory/runtime core | Can insert a claim, supersede it retroactively, and answer both “what was true at valid time V” and “what did we believe at recorded time R”. |
| P10 | Coding-agent tool suite, sandbox discipline, and Codex packet generator | Product-grade coding spine | Coding profile can read repo, propose a patch, request approval, apply patch after permit, and run cargo checks with receipts. |
| P11 | Queue, schedule, wake, daemon, leases, and duplicate-storm immunity | Memory/runtime core | Repeated same schedule occurrence cannot create duplicate logical jobs. |
| P12 | Canonical verification, repair, contradiction, and governance adapters | Governed runtime | Risk-bearing claims route through canonical verification-control/policy/adjudication artifacts. |
| P13 | Multi-view runtime, retrieval disclosure, and query widening law | Runtime/query core | A time-scoped query cannot silently fall back to timeless retrieval. |
| P14 | Release-grade product surface, operator UX, and status truth | Product readiness | A new user can create an app, run provider-check, inspect tools, run mock turn, inspect receipts, and run verify.sh. |
| P15 | Regional decoder kernel, right-graph law, and local repair geometry | Advanced runtime geometry | A synthetic contradiction emits SyndromeV1 and local repair candidate instead of global recompute. |
| P16 | Lawful subtraction, compaction, and invariant-preserving reduction | Advanced runtime geometry | Subtraction cannot delete support needed by accepted claim unless claim is superseded/quarantined first. |
| P17 | Attested exchange, trust roots, federation, and external artifact admission | Federated horizon | External artifact can be imported only through AdmissionDecisionV1. |
| P18 | Mechanism/theory search, experiment runtime, and falsifiable model library | Mechanism horizon | A candidate mechanism can be fit, refuted, versioned, superseded, and replayed from artifacts. |
| P19 | Final integration, release bar, and completion audit | Release proof | cargo fmt/check/test/clippy pass for workspace with all features. |
