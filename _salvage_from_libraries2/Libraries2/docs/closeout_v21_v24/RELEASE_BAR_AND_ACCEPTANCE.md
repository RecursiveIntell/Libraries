# Release bar and acceptance — v21 through v24

## What counts as landed

A v21–v24 surface counts as landed only if:
- it has a canonical owner crate,
- it has a schema file,
- it has an example file,
- it appears in the per-wave manifest,
- it has at least one fixture or typed receipt path,
- and its advisory/admitted/emergency status is explicit.

## What does not count
- a noun in a spec with no code owner,
- a struct with no schema owner,
- a schema with no example,
- an emergency path with no expiry or review artifact,
- a release decision with no assurance case,
- a delegated action with no authority chain,
- a live effect with no observation or compensation semantics.

## Pass-level acceptance

The final pass is acceptable if it lands:
- the four new owner crates,
- all v21–v24 schema/example/manifests in this pack,
- one bounded vertical slice per wave,
- additive integration points into the existing canonical lane,
- and a terminal design position that refuses v25 inflation.

## Explicit non-goals
- domain-specific privacy or regulatory overlays as new core spec versions,
- deep optimization of every runtime,
- full operational automation,
- compile-certified production readiness from this pack alone.
