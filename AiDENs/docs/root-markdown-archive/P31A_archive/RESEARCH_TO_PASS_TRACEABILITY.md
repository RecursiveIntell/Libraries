# Research-to-Pass Traceability

This file maps the latest research corpus to concrete implementation passes. Use it to avoid treating research themes as vibes.

| Research theme / source family | Core claim | Implementation passes |
|---|---|---|
| `execution/evidence.md`, `control-plane.md`, `system/testify.md`, `recall-testify.md`, `final-recall-implementation.md` | Execution metadata is epistemic evidence: provider route, tool call, retry, queue hop, deadline, budget, and dispatch outcome must be durable and queryable. | P02, P03, P05, P11, P14 |
| `episode/identity.md`, `semantic-artifact-law.md`, v9 spec material | Documents are containers; episodes are identity-bearing evidentiary units. Execution context is artifact semantics. | P05, P09, P12, P13 |
| `contract/hardening.md`, `contract/discipline.md`, `contract/temporal.md`, `json/contract.md` | Type-owned contracts, generated schemas, compatibility checks, canonical digests, and temporal replay are non-negotiable. | P06, P07, P08, P09 |
| `parser/patch.md` | Structured-output parsing/repair is a compiler front end with treatment-integrity obligations. | P06, P10 |
| `bitemporal/*`, `temporal/truth.md`, `truth/temporal.md`, `lawful/bitemporal.md` | Valid time and recorded time must be separate, append-plus-supersession is required, and as-of queries must be deterministic. | P09, P13, P16 |
| `causal.md`, `verification/causal.md`, `synthesis/verification.md` | Causal/risk-bearing claims require treatment/outcome/confounder/evidence/refutation packages. | P12, P18 |
| `decoder/*`, `quantum/decoder.md`, `message/prmitive.md` | The runtime eventually behaves like a decoder: syndromes, residuals, hyperedges, oracle slices, and convergence governance. | P15 |
| `decoder/proticals.md`, `region.md`, v12 region material | Small communicating regions with typed boundary protocols beat one giant graph. | P15, P16 |
| `lawful/epistemic.md`, subtraction/end-state material | Subtraction is the dual of accumulation: compaction/removal must preserve declared invariants and emit receipts. | P16 |
| v16/v17/federation/mechanism material | External artifacts and discovered theories require admission, settlement, refutation, and local-authority-preserving publication. | P17, P18 |
| `final-recall-implementation.md` | SQLite/WAL/outbox, sandbox truth, scheduler model, provider certification, packet fail-closed, and migration law are immediate high-ROI implementation risks. | P05, P07, P10, P11, P14 |

## Research implication for pass order

Advanced inference is deliberately late. The research does not say “build a decoder first.” It says: first build the typed, durable, temporally truthful evidence substrate; then compile lawful regions and run decoder-style inference on artifacts that can be audited.
