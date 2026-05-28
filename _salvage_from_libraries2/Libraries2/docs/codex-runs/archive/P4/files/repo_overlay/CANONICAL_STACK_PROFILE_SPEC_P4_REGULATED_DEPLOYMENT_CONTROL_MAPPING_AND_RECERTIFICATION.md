# Canonical Stack Profile Spec P4 — Regulated Deployment, Control Mapping, and Recertification

**Status:** Proposed post-v24 profile supplement  
**Relationship to v6-v24:** All prior law remains in force. This document does not weaken authority asymmetry, bitemporality, execution evidence, assurance law, or continuity law.  
**Scope:** profile-layer typed overlays for regulated deployment, control mapping, and recertification.

---

## 0. Purpose

This document defines a profile-layer target state after the general-purpose v24 closeout.

### 0.1 Design basis (non-normative)

This supplement is synthesized from v23 deployment and assurance law, v19 constitutional compatibility and retirement law, and the explicit backlog item for regulated-deployment control mappings.

The design basis compresses to one sentence:

> The stack MUST treat regime-specific control mappings and recertification schedules as typed overlays over existing assurance law rather than prose-only compliance binders.

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

The stack MUST treat regime-specific control mappings and recertification schedules as typed overlays over existing assurance law rather than prose-only compliance binders.

This profile layer is lawful only if it remains:
- typed,
- replayable where materially important,
- reviewable,
- bounded by expiry or compatibility windows where exceptions exist,
- and explicitly subordinate to the existing base law.

---

## 3. Profile artifact law

### RegulatoryRegimeProfileV1
The stack MUST define one canonical logical artifact family for `RegulatoryRegimeProfileV1`.

It MUST include at minimum:
- `schema_version`
- `regulatory_regime_profile_id`
- `regime_name`
- `regime_version`
- `covered_products`
- `mandatory_control_families`
- `audit_cycle_days`.

### RequirementControlMapV1
The stack MUST define one canonical logical artifact family for `RequirementControlMapV1`.

It MUST include at minimum:
- `schema_version`
- `requirement_control_map_id`
- `regulatory_regime_profile_id`
- `mappings`
- `gap_classification_default`
- `owner_ref`.

### EvidenceCollectionPlanV1
The stack MUST define one canonical logical artifact family for `EvidenceCollectionPlanV1`.

It MUST include at minimum:
- `schema_version`
- `evidence_collection_plan_id`
- `regulatory_regime_profile_id`
- `required_evidence_classes`
- `collection_cadence`
- `retention_class`
- `owner_ref`.

### RecertificationScheduleV1
The stack MUST define one canonical logical artifact family for `RecertificationScheduleV1`.

It MUST include at minimum:
- `schema_version`
- `recertification_schedule_id`
- `regulatory_regime_profile_id`
- `review_interval_days`
- `trigger_classes`
- `grace_window_days`
- `blocked_state_on_expiry`.

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

### 7.1 `assurance-runtime`
Should own the primary regime, control-map, evidence-collection, and recertification families.

### 7.2 `verification-control` and `verification-adjudication`
Should consume these profiles when determining release readiness and blocked states.

### 7.3 `constitutional-memory`
Should consume these profiles when evidence obligations or retirement windows change.

---

## 8. Build order

### P4-0 — vocabulary freeze
- freeze the four profile families,
- keep them bound to already-existing assurance surfaces.

### P4-1 — publication
- publish schemas and examples,
- map regime requirements to canonical evidence families.

### P4-2 — recertification pilot
- prove one happy release mapping,
- prove one overdue or trigger-driven blocked state.

---

## 9. Explicit non-goals

P4 does **not**:
- create a new assurance plane,
- replace base certification law,
- or legalize “CI green” as a regulated release decision.

---

## 10. Conformance headline

A system conforms to P4 only if, for a materially important path affected by this profile layer, it can answer all of the following without archaeology:

- which profile artifact governed the path,
- which base artifact families it constrained,
- whether an exception or downgrade was used,
- whether the effect remained replayable or auditable,
- and what review or expiry conditions still applied.

If the answer is “that lived in config somewhere,” the system is not yet P4-conforming.

For this document, that means the system can answer at minimum: for a materially important regulated release, which regime profile governed it, which controls satisfied which requirements, what evidence plan applied, and whether recertification remained current.
