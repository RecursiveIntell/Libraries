# Schema Authority Source Of Truth

SOURCE BASIS: 2026-04-28

Canonical stack artifact family schema generation is owned by canonical owner crates and `~/Coding/Libraries/contract-schema-gen`.

AiDENs may generate schemas only for AiDENs-local product, display, report, CLI, operator, queue, schedule, wake, permit, provider, release, and schema-governance DTOs. These schemas are non-authoritative and do not define canonical stack artifact law.

AiDENs must not emit canonical family schemas for:

- attestation, admission, trust, or federation settlement artifacts;
- mechanism theory, hypothesis, simulator, fit, or invariance artifacts;
- memory, evidence, episode, claim, projection, runtime-view, or widening artifacts;
- kernel, region, syndrome, residual, convergence, subtraction, or support-core artifacts;
- verification, repair, adjudication, reference-conformance, or schema-validation truth artifacts;
- digest/content-addressing law.

Canonical family schema work is routed to `contract-schema-gen`, which already registers the canonical owner crate types for attestation, settlement, mechanism, memory/evidence, verification/repair, and related stack surfaces.

The AiDENs schema manifest remains a local compatibility report for AiDENs DTOs only. Its registry and manifest digests are non-authoritative display digests and must not be used as stack artifact identity.
