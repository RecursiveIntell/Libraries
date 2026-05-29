# AiDENs

**Current run:** `P31B-VERIFICATION-REPAIR`  
**Status:** `candidate` — all 17 verification gates pass; P31B repair complete.  
**Last certified run:** `P30`  
**Target run:** `P31B`  

## What AiDENs is

AiDENs is an orchestration, display, packaging, inspection, fixture, operator, and supported-local runtime layer for the RecursiveIntell stack. It wires, scopes, exposes, validates, and coordinates. It does not own canonical truth for memory, governance, kernel, IDs, or tool contracts.

## Current scope

This repository completed P31B verification repair of P31A false-certification drift:

- Release/run truth ledger recertified: P31B candidate, all gates pass
- Artifact classification and verifier self-poisoning repaired
- Static safety hardening: p30_guard 0 hard findings (child.kill already replaced)
- Build/test/package replay evidence established (15 command receipts)
- z.py normalize bug fixed for letter-suffix run IDs (P31B)
- Supported-local vertical slice proven (boundary compiler + tool dispatch)

## What is not claimed

- v11B, v11C, production, broad autonomy, or cloud readiness
- Boundary compiler runtime integration (deferred to post-P31B)
- Canonical ownership of sibling-crate semantics
- Full test certification until command bar passes
- Certified status — current status is `blocked` / `verification-repair-candidate`

## Quick start

1. Read `docs/codex-runs/CURRENT_RUN.json` for the active run identity.
2. Read `AGENTS.md` for execution doctrine.
3. Run `scripts/verify_current.sh .` after any material change.
4. Check `docs/codex-runs/BUILD_SCOPE.md` for current build posture.

**Support label:** `p31b-verification-repair-candidate` (`supported-local-candidate`)
**Production status:** not production-cloud-ready  
**Local candidate status:** repair-in-progress (do not claim certified)

## Directory guide

- `crates/` — Rust workspace crates
- `scripts/` — Verification, packaging, and assertion scripts
- `docs/codex-runs/` — Active run docs and archive
- `matrices/` — Issue and audit matrices
- `scaffold/` — Deferrable stub material (see STATUS.md crate inventory for scaffold-only crates)

## Support

See `SUPPORT_PROFILE.md` for current support posture and known limitations.