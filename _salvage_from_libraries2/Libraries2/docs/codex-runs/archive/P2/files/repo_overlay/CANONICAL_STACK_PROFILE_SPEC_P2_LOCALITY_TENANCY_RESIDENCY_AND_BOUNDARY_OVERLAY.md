# Canonical Stack Profile Spec P2 — Locality, Tenancy, Residency, and Boundary Overlay

**Status:** Proposed post-v24 profile supplement  
**Relationship to v6-v24:** All prior law remains in force. This document does not weaken authority asymmetry, bitemporality, execution evidence, assurance law, or continuity law.  
**Scope:** profile-layer typed overlays for locality, tenancy, residency, and boundary overlay.

---

## 0. Purpose

This document defines a profile-layer target state after the general-purpose v24 closeout.

### 0.1 Design basis (non-normative)

This supplement is synthesized from v15 attested exchange and disclosure law, v16 treaty-scoped replay and shared-view law, and the repo’s explicit backlog for locality / tenancy overlays.

The design basis compresses to one sentence:

> The stack MUST treat locality, tenancy, residency, and cross-boundary transfer as typed overlay law rather than hidden infrastructure placement assumptions.

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

The stack MUST treat locality, tenancy, residency, and cross-boundary transfer as typed overlay law rather than hidden infrastructure placement assumptions.

This profile layer is lawful only if it remains:
- typed,
- replayable where materially important,
- reviewable,
- bounded by expiry or compatibility windows where exceptions exist,
- and explicitly subordinate to the existing base law.

---

## 3. Profile artifact law

### ResidencyPolicyProfileV1
The stack MUST define one canonical logical artifact family for `ResidencyPolicyProfileV1`.

It MUST include at minimum:
- `schema_version`
- `residency_policy_profile_id`
- `allowed_storage_regions`
- `allowed_execution_regions`
- `allowed_replay_regions`
- `forbidden_transfer_classes`
- `default_exception_path`.

### TenantBoundaryProfileV1
The stack MUST define one canonical logical artifact family for `TenantBoundaryProfileV1`.

It MUST include at minimum:
- `schema_version`
- `tenant_boundary_profile_id`
- `tenant_key_kind`
- `isolation_class`
- `shared_service_allowances`
- `cross_tenant_query_default`
- `key_management_segregation`.

### CrossBoundaryTransferClassV1
The stack MUST define one canonical logical artifact family for `CrossBoundaryTransferClassV1`.

It MUST include at minimum:
- `schema_version`
- `cross_boundary_transfer_class_id`
- `source_class`
- `destination_class`
- `allowed_artifact_families`
- `required_attestation`
- `required_disclosure_policy_class`
- `downgrade_behavior`.

### LocalityExceptionV1
The stack MUST define one canonical logical artifact family for `LocalityExceptionV1`.

It MUST include at minimum:
- `schema_version`
- `locality_exception_id`
- `residency_policy_profile_id`
- `reason`
- `scope`
- `expires_at`
- `approved_by`
- `post_hoc_review_required`.

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

### 7.1 `verification-policy`
Should own the primary policy surfaces for residency, tenancy, and cross-boundary transfer classes.

### 7.2 `remote-oracle-admission`
Should consume these profiles when remote replay or remote execution crosses declared locality boundaries.

### 7.3 `federated-settlement`
Should consume these profiles when treaty-scoped publication or replay spans multiple runtimes.

---

## 8. Build order

### P2-0 — vocabulary freeze
- freeze the four profile families,
- keep them explicitly profile-level.

### P2-1 — residency and tenancy publication
- publish schemas and examples,
- bind default-deny transfer and query rules explicitly.

### P2-2 — bounded exception pilot
- prove one blocked cross-boundary path,
- prove one expiring exception with post-hoc review.

---

## 9. Explicit non-goals

P2 does **not**:
- create a new federation layer,
- replace trust-root or treaty law,
- legalize permanent boundary exceptions,
- or make “cloud region placement” an implicit semantics source.

---

## 10. Conformance headline

A system conforms to P2 only if, for a materially important path affected by this profile layer, it can answer all of the following without archaeology:

- which profile artifact governed the path,
- which base artifact families it constrained,
- whether an exception or downgrade was used,
- whether the effect remained replayable or auditable,
- and what review or expiry conditions still applied.

If the answer is “that lived in config somewhere,” the system is not yet P2-conforming.

For this document, that means the system can answer at minimum: for a materially important transfer or replay, which residency and tenancy profiles governed it, which transfer class applied, whether an exception was used, and when that exception expires.
