# Canonical Stack Profile Spec P6 — Vendor Certification Adapter and External Evidence Translation

**Status:** Proposed post-v24 profile supplement  
**Relationship to v6-v24:** All prior law remains in force. This document does not weaken authority asymmetry, bitemporality, execution evidence, assurance law, or continuity law.  
**Scope:** profile-layer typed overlays for vendor certification adapter and external evidence translation.

---

## 0. Purpose

This document defines a profile-layer target state after the general-purpose v24 closeout.

### 0.1 Design basis (non-normative)

This supplement is synthesized from v15 attested exchange and remote admission law, v23 certification bundles, and the explicit backlog item for vendor-specific certification adapters.

The design basis compresses to one sentence:

> The stack MUST treat vendor certification and external evidence translation as typed adapter law rather than one-off import glue.

### 0.2 Normative language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, and **MAY** are to be interpreted as normative requirements.

Where this document distinguishes between:
- **logical model** — required semantics, artifact classes, invariants, and transitions,
- **physical model** — concrete tables, crates, services, scripts, or SaaS configuration,

…the logical model is mandatory. The physical model is flexible only where this document explicitly allows flexibility.

---

## 1. Enduring doctrine carried forward

Nothing in this profile supplement weakens prior law.

- authority remains asymmetric,
- time remains part of meaning,
- execution remains evidence,
- base artifact families remain authoritative only within their existing planes,
- and profile artifacts may parameterize existing law but MUST NOT create a new truth plane.

---

## 2. Core thesis

The stack MUST treat vendor certification and external evidence translation as typed adapter law rather than one-off import glue.

This profile layer is lawful only if it remains:
- typed,
- replayable where materially important,
- reviewable,
- bounded by expiry or compatibility windows where exceptions exist,
- and explicitly subordinate to the existing base law.

---

## 3. Profile artifact law

### VendorCertificationAdapterV1
The stack MUST define one canonical logical artifact family for `VendorCertificationAdapterV1`.

It MUST include at minimum:
- `schema_version`
- `vendor_certification_adapter_id`
- `vendor_name`
- `product_surface`
- `covered_artifact_families`
- `translation_mode`
- `support_window`.

### VendorEvidenceTranslationV1
The stack MUST define one canonical logical artifact family for `VendorEvidenceTranslationV1`.

It MUST include at minimum:
- `schema_version`
- `vendor_evidence_translation_id`
- `vendor_certification_adapter_id`
- `source_shapes`
- `canonical_targets`
- `lossy_fields`
- `required_caveats`.

### VendorTrustRootBindingV1
The stack MUST define one canonical logical artifact family for `VendorTrustRootBindingV1`.

It MUST include at minimum:
- `schema_version`
- `vendor_trust_root_binding_id`
- `vendor_certification_adapter_id`
- `trust_root_refs`
- `signer_classes`
- `rotation_channel`
- `revocation_channel`.

### VendorRevocationHandlingV1
The stack MUST define one canonical logical artifact family for `VendorRevocationHandlingV1`.

It MUST include at minimum:
- `schema_version`
- `vendor_revocation_handling_id`
- `vendor_certification_adapter_id`
- `revocation_inputs`
- `local_invalidation_actions`
- `replay_impact`
- `admission_impact`.

---

## 4. Overlay rule

These profile families MUST be treated as overlays over already-admitted base artifacts.

They MAY:
- constrain admission,
- constrain disclosure,
- constrain delegation,
- constrain release readiness,
- constrain incident routing,
- or bind sector- and vendor-specific doctrine to existing surfaces.

They MUST NOT:
- create a new durable truth plane,
- replace the existing base artifact families,
- or silently rewrite previously admitted meaning.

---

## 5. Publication rule

Every wire-visible family in this profile supplement MUST have:
- one canonical owner,
- one canonical schema filename,
- one example,
- one fixture presence,
- and one release-bar statement naming what it is allowed to mean.

A prose-only profile is non-conforming.

---

## 6. Compatibility and exception rule

If a deployment needs to override a default profile rule, it MUST do so through:
- a typed exception artifact where this supplement provides one,
- an explicit compatibility window,
- or a higher-level already-lawful constitutional change.

“Operator knew it was fine” is non-conforming.

---

## 7. Crate and plane implications

### 7.1 `attestation-exchange`
Should own the primary vendor adapter, translation, trust-binding, and revocation-handling families.

### 7.2 `assurance-runtime`
Should consume these profiles when vendor evidence contributes to local certification or release readiness.

### 7.3 `remote-oracle-admission`
May consume these profiles when remote providers are admitted through vendor-specific envelopes or execution evidence.

---

## 8. Build order

### P6-0 — vocabulary freeze
- freeze the four profile families,
- keep them explicitly adapter-level.

### P6-1 — publication
- publish schemas and examples,
- require declared lossiness and caveats.

### P6-2 — revocation pilot
- prove one happy translation path,
- prove one revocation-driven downgrade / reopen path.

---

## 9. Explicit non-goals

P6 does **not**:
- weaken local authority,
- create a general vendor abstraction for everything,
- legalize unsigned or unaudited translation,
- or permit vendor artifacts to self-promote into local truth.

---

## 10. Conformance headline

A system conforms to P6 only if, for a materially important path affected by this profile layer, it can answer all of the following without archaeology:

- which profile artifact governed the path,
- which base artifact families it constrained,
- whether an exception or downgrade was used,
- whether the effect remained replayable or auditable,
- and what review or expiry conditions still applied.

If the answer is “that lived in config somewhere,” the system is not yet P6-conforming.

For this document, that means the system can answer at minimum: for a materially important external vendor artifact, which adapter translated it, which trust roots bound it, what lossiness or caveats remained, and how revocation changed local admissibility.
