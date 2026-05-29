# P28 Master Issue Matrix

## P0 release blockers

| ID | Issue | Owner surface | Required resolution | Gate |
|---|---|---|---|---|
| P28-001 | Close Claude P0 bug set C05/C07/C11/C24/C25/C32/C53/C54/C55/C59/C66/C72 | contracts/tool-kit/z.py/status | fix + regression or release-blocking quarantine | Gate B |
| P28-002 | Add artifact envelope/lifecycle/transition law | aidens-contracts | implement/admit v11A artifact kernel | Gate C |
| P28-003 | Add execution context envelope and per-call receipts | runner/tool-kit/receipts | no material done without receipts | Gate E |
| P28-004 | Add material operation registry and effect contracts | contracts/runner/tool-kit/cli | register declared production path | Gate D |
| P28-005 | Add boundary compiler profiles and repair/treatment receipts | boundary-kit/contracts | strict JSON/schema/canonicalization/repair law | Gate F |
| P28-006 | Add proof profile/debt/waiver semantics | contracts/governance/runner | waiver != proof; debt restricts use | Gate G |
| P28-007 | Make run bundles/event logs immutable/replay-aware | receipts/runner | no overwrite; hash chain | Gate I |
| P28-008 | Fix aggregate exact/degraded status honesty | contracts/status/package | top-level status downgrades on degraded subcheck | Gate J |

## P1 implementation issues

| ID | Issue | Resolution |
|---|---|---|
| P28-101 | Megafile containment | split contract/runner/tool modules without ownership drift |
| P28-102 | Schema catalog runtime recomputation | memoize or generate static schema docs where appropriate |
| P28-103 | Tool/patch sandbox hardening | symlink/dirty-write/allowlist/timeouts tests |
| P28-104 | View disclosure and semantic state | exact/degraded/support/proof carriers |
| P28-105 | Bitemporal reference fixtures | minimal differential harness for declared query path |
| P28-106 | v11C reserved stubs | activation levels, quarantine default, agency high-risk classification |

## P2 follow-up issues

| ID | Issue | Resolution |
|---|---|---|
| P28-201 | v11B region/subtraction DTO validation | keep draft/advisory only, add reserved tests |
| P28-202 | Provider native tool loop | keep deferred until provider/admission receipt law exists |
| P28-203 | Production daemon authority | keep deferred until daemon control-plane evidence exists |
| P28-204 | Full federation/mechanism/self-hosting | keep v11C reserved only |
