# v25 effective constitution exec plan

## Goal

Make the repo itself the authoritative landing surface for v25, not just an external pack plus a partial overlay.

## Ordered steps

1. Freeze current repo truth with a supersession note.
2. Carry the canonical v25 spec into the repo root.
3. Carry the v26 horizon spec into the repo root, clearly marked advisory-only.
4. Add a repo-facing docs/v25 execution pack.
5. Expand the v25 fixture corpus and add a fixture manifest.
6. Add repo-truth and JSON-surface validation scripts that do not require Rust.
7. Replace mirror sync path enumeration with whole-tree sync.
8. Sync the `libraries-source/` mirror.
9. Run local validation scripts.
10. Package the full workspace zip, overlay zip, patch, and delivery docs.

## Non-goals for this pass

- pretending downstream effect/control/adjudication adoption is already fully proven,
- deleting the historical no-v25 materials,
- or claiming schema regeneration happened without the Rust toolchain.
