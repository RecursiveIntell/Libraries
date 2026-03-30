# 13_IMPLEMENTATION_PLAYBOOK

## Operating posture

- patch truth surfaces first
- then patch semantic seams
- then patch convergence / docs / maintainability
- prove every change as you go

## Mechanical sequence

### Pass 1 — front door
- land the numbered pack
- update `scripts/check_pack_truth.sh` only if the pack convention is changing deliberately
- fix archive manifest
- regenerate status / receipt surfaces

### Pass 2 — support lane
- pick one lane
- make `Makefile`, `SUPPORT_PROFILE.md`, dashboard, and receipt agree
- document adjacent but non-certified crates separately

### Pass 3 — kernel/runtime seam
- add degraded-reason fields
- thread them through runtime outputs and query provenance
- regenerate schemas
- land fixtures

### Pass 4 — artifact convergence
- unify execution evidence across tool runtime / pilot / verification
- centralize `SurfaceStatus`
- rename or deepen thin runtime crates

### Pass 5 — polish
- add rustdoc where the repo is currently silent
- split hotspot files
- clean package outputs
- archive duplicate root/meta surfaces

## Change discipline

Never close a row because the code looks right.
Close it only when the named proof artifact exists and the named command passes.
