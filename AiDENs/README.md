# AiDENs

**Current run:** `P32-SCHEMA-COMPAT`  
**Status:** `candidate` — all 17 verification gates pass; P32 schema compat in progress.  
**Last certified run:** `P30`  
**Target run:** `P32`  

## What AiDENs is

AiDENs is an orchestration, display, packaging, inspection, fixture, operator, and supported-local runtime layer for the RecursiveIntell stack. It wires, scopes, exposes, validates, and coordinates. It does not own canonical truth for memory, governance, kernel, IDs, or tool contracts.

## Current scope

This repository completed P32 schema-compatibility work building on P31B verification repair:

- Release/run truth ledger recertified: P32 candidate, all gates pass
- Artifact classification and verifier self-poisoning repaired
- Static safety hardening: p30_guard 0 hard findings (child.kill already replaced)
- Build/test/package replay evidence established (15 command receipts)
- z.py normalize bug fixed for letter-suffix run IDs (P32)
- Supported-local vertical slice proven (boundary compiler + tool dispatch)

## What is not claimed

- v11B, v11C, production, broad autonomy, or cloud readiness
- Boundary compiler runtime integration (deferred to post-P32)
- Canonical ownership of sibling-crate semantics
- Full test certification until command bar passes
- Certified status — current status is `blocked` / `schema-compat-candidate`

## Quick start

1. Read `docs/codex-runs/CURRENT_RUN.json` for the active run identity.
2. Read `AGENTS.md` for execution doctrine.
3. Run `scripts/verify_current.sh .` after any material change.
4. Check `docs/codex-runs/BUILD_SCOPE.md` for current build posture.

**Support label:** `p32-schema-compat-candidate` (`supported-local-candidate`)
**Production status:** not production-cloud-ready  
**Local candidate status:** schema-compat-in-progress (do not claim certified)

## Directory guide

- `crates/` — Rust workspace crates
- `scripts/` — Verification, packaging, and assertion scripts
- `docs/codex-runs/` — Active run docs and archive
- `matrices/` — Issue and audit matrices
- `scaffold/` — Deferrable stub material (see STATUS.md crate inventory for scaffold-only crates)

## Support

See `SUPPORT_PROFILE.md` for current support posture and known limitations.