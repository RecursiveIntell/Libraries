# AiDENs

**Current run:** `P31A-RECOVERY`  
**Status:** `certified` — all P31A phases complete; release truth, artifact classification, verification gates, and package replay passed.  
**Last certified run:** `P31A`  
**Target run:** `P31A`  

## What AiDENs is

AiDENs is an orchestration, display, packaging, inspection, fixture, operator, and supported-local runtime layer for the RecursiveIntell stack. It wires, scopes, exposes, validates, and coordinates. It does not own canonical truth for memory, governance, kernel, IDs, or tool contracts.

## Current scope

This repository is actively repairing:

- Release/run truth ledger (`CURRENT_RUN.json`)
- Root Markdown artifact classification and archival
- Verification gate semantics and script alignment
- Static safety hardening (`p30_guard` findings)
- Build/test/package replay evidence

## What is not claimed

- v11B, v11C, production, broad autonomy, or cloud readiness
- Boundary compiler runtime integration (deferred to post-P31A)
- Canonical ownership of sibling-crate semantics
- Full test certification until command bar passes

## Quick start

1. Read `docs/codex-runs/CURRENT_RUN.json` for the active run identity.
2. Read `AGENTS.md` for execution doctrine.
3. Run `scripts/verify_current.sh` after any material change.
4. Check `docs/codex-runs/BUILD_SCOPE.md` for current build posture.

**Support label:** `p31a-certified-release-truth-repair`  
**Production status:** not production-cloud-ready  
**Local candidate status:** supported-local-candidate

## Directory guide

- `crates/` — Rust workspace crates
- `scripts/` — Verification, packaging, and assertion scripts
- `docs/codex-runs/` — Active run docs and archive
- `matrices/` — Issue and audit matrices
- `scaffold/` — Experimental / deferred scaffold material (not production-wired)

## Support

See `SUPPORT_PROFILE.md` for current support posture and known limitations.
