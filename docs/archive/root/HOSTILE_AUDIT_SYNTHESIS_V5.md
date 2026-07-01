# Libraries Hostile Audit Synthesis — V5

## Date: 2026-03-30
## Auditors: Claude Opus 4.6 (code-grounded), GPT-5.4 (code-grounded + receipt-aware)

---

## Combined score: 8.1 / 10

| Auditor | Score | Basis |
|---|---|---|
| Claude Opus 4.6 | 8.0 | Direct code inspection of vendored deps + import boundaries in Recall workspace |
| GPT-5.4 | 8.2 | Direct static inspection of full 30-crate workspace archive |

---

## Convergent findings (both auditors agree)

### Theme 1: Governance is observational, not commanding
- **LIB-001** (GPT-5.4 P0): governance_gate.rs is fail-open, returns default on errors
- **CLIB-005** (Claude P1): Governance receipt is append-only metadata, never gates execution
- **Assessment:** This is the single most important architectural gap. The system knows when governance says "blocked" and records it honestly, but execution proceeds regardless.

### Theme 2: Reproducibility and external verifiability
- **LIB-007** (GPT-5.4 P1): No independent reproducibility harness
- **CLIB-012** (Claude P1): 15 sibling path dependencies — fresh clone requires full directory tree
- **Assessment:** Both auditors independently flagged that external reviewers cannot reproduce build gates. The vendored `deps/` pattern is the right model but only covers 2 of 17 dependencies.

### Theme 3: Hotspot concentration
- **LIB-004** (GPT-5.4 P1): Giant crates carry disproportionate risk
- **CLIB-009** (Claude P2): orient() has no hard cap, shares budget with largest crates
- **Assessment:** 4 crates account for >60% of source LOC. Future bugs will concentrate here.

### Theme 4: Operational truth sprawl
- **LIB-003** (GPT-5.4 P1): Root-level packet sprawl creates narrative drift
- **CLIB-014** (Claude P2): Capability spec claims are prose, not machine-verifiable
- **Assessment:** The documentation is thorough but creates a maintenance surface that can drift from code truth.

## Divergent findings

### GPT-5.4 found, Claude didn't flag:
- **LIB-002** (closeout claim scope): Claude didn't see the closeout receipt as only the Recall subset was in scope
- **LIB-006** (dev-profile benchmarks): Claude didn't audit performance evidence

### Claude found, GPT-5.4 didn't flag:
- **CLIB-001** (stringly-typed GovernanceReceipt): Code-level finding invisible to receipt-level audit
- **CLIB-002** (ToolCtx serde round-trip): Fragile construction pattern only visible in source
- **CLIB-003** (constraint compilation discarded): Pipeline-level finding requiring tracing the CompileOutput lifecycle
- **CLIB-006** (governance exceptions lack temporal validation): requires reading exception_covers() implementation
- **CLIB-007** (write tools shown without approval handler): requires tracing tool prompt → dispatch path

## Closure from prior audits

- **LIB-005** (GPT-5.4 P2: no workspace lints): **Closed in Recall workspace.** `Cargo.toml` lines 68-72 declare `unsafe_code = "deny"`, `clippy::todo = "deny"`, `clippy::dbg_macro = "deny"` at workspace level. Whether the parent library workspace also has these is unknown from this upload.

## Issue priority distribution (combined)

| Priority | Count | Sources |
|---|---|---|
| P0 | 1 | LIB-001 (GPT-5.4) |
| P1 | 7 | LIB-002, LIB-003, LIB-004, LIB-007, CLIB-003, CLIB-005, CLIB-007, CLIB-012 |
| P2 | 8 | CLIB-001, CLIB-002, CLIB-004, CLIB-006, CLIB-008, CLIB-009, CLIB-010, CLIB-014, LIB-006 |
| P3 | 2 | CLIB-011, CLIB-013 |
| Closed | 1 | LIB-005 |

## Recommended fix order for CLARA deadline (April 10)

1. **CLIB-007** — Filter write tools from prompt when no approval handler. Immediate UX improvement, < 20 lines.
2. **CLIB-005 / LIB-001** — Add strict mode that returns error when governance says "blocked". Core CLARA differentiator.
3. **CLIB-003** — Route constraint CompileOutput into session state on every ingest. Makes constraint compilation visible in the OODA loop.
4. **CLIB-001** — Enum-ify GovernanceReceipt disposition. Type safety improvement, moderate refactor.
5. **CLIB-012** — Vendor remaining deps or document the build topology for reviewers.
