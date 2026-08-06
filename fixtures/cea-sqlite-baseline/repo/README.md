# cea-sqlite

## Role
SQLite implementation of the cea-store contract

## Snapshot position
- Path: `Primitives/cea-sqlite`
- Position: **Excluded**
- Plane: **Supporting crate**
- Snapshot status: **Critical dependency**

## What this crate is for
This README exists to satisfy manifest truth and to give contributors a truthful front door for the current library stack. The active finish-line control plane lives at the repository root in `PACK_README.md`, `MASTER_ISSUE_MATRIX.md`, `MASTER_ISSUE_CHANGE_MATRIX.md`, `AGENTS.md`, and the docs index at `docs/README.md`.

This crate currently serves low-frequency evaluation persistence. It is not part of the supported-core default-members bar, and it is not claimed as a hot-path pooled store on the shipped root surface.

## Non-negotiables
- Keep this crate aligned with the canonical authority map.
- Do not let local convenience APIs become de facto truth surfaces.
- Add or update tests when crossing a semantic seam.
- Prefer explicit versioned contracts over drift-by-example.

## Current proof posture
This snapshot review was static. Treat this crate as **not build-certified from this pack alone** until the relevant Rust gates are re-run in a toolchain-backed environment.
