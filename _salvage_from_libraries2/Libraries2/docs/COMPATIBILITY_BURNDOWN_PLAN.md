# Compatibility Burndown Plan

## Objective

Prevent legacy compatibility lanes from becoming the default path by inertia.

## Current compatibility surfaces to track

- historical root control-plane docs (`README_CONTROL_PLANE_V7.md`, `README_CONTROL_PLANE_V8_FINISHLINE.md`)
- envelope version lanes (V1 / V2 / V3)
- importer wire compatibility in `forge-memory-bridge`
- repo-local spec copies vs canonical target-state law
- excluded ecosystem crates that are real but outside the supported release lane

## Retirement rules

1. No legacy file may remain in the front-door read order once a replacement exists.
2. Compatibility-only lanes must be explicitly labeled as such in code comments and docs.
3. Any version retirement must name:
   - last supported release,
   - migration path,
   - tests that protect downgrade/upgrade behavior,
   - and the removal gate.

## Immediate follow-on

- move any remaining historical front-door references behind the `ARCHIVE/` pointer only;
- once schemas land, document which envelope versions remain writable vs readable-only.
