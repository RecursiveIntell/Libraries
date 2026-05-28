# Canonical Stack Profile Spec P3 — Role Catalog, Duty Segregation, and Approval Matrix

**Status:** Proposed post-v24 profile supplement  
**Relationship to v6-v24:** All prior law remains in force. This document does not weaken authority asymmetry, bitemporality, execution evidence, assurance law, or continuity law.  
**Scope:** profile-layer typed overlays for role catalog, duty segregation, and approval matrix.

---

## 0. Purpose

This document defines a profile-layer target state after the general-purpose v24 closeout.

### 0.1 Design basis (non-normative)

This supplement is synthesized from v22 delegated-authority law, v21 live-effect approval requirements, and the existing policy-profile and separation-of-duties scaffolding in the repo.

The design basis compresses to one sentence:

> The stack MUST treat role catalogs, approval matrices, delegation constraints, and conflict classes as typed overlay law rather than spreadsheet-only governance.

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

The stack MUST treat role catalogs, approval matrices, delegation constraints, and conflict classes as typed overlay law rather than spreadsheet-only governance.

This profile layer is lawful only if it remains:
- typed,
- replayable where materially important,
- reviewable,
- bounded by expiry or compatibility windows where exceptions exist,
- and explicitly subordinate to the existing base law.

---

## 3. Profile artifact law

### RoleCatalogV1
The stack MUST define one canonical logical artifact family for `RoleCatalogV1`.

It MUST include at minimum:
- `schema_version`
- `role_catalog_id`
- `role_definitions`
- `default_autonomy_ceiling`
- `scope_rule`
- `review_cycle_days`.

### DelegationMatrixV1
The stack MUST define one canonical logical artifact family for `DelegationMatrixV1`.

It MUST include at minimum:
- `schema_version`
- `delegation_matrix_id`
- `allowed_edges`
- `max_delegation_depth`
- `required_lease_classes`
- `forbidden_chain_patterns`.

### ApprovalMatrixV1
The stack MUST define one canonical logical artifact family for `ApprovalMatrixV1`.

It MUST include at minimum:
- `schema_version`
- `approval_matrix_id`
- `approval_rules`
- `default_quorum`
- `requires_independent_review_for`
- `break_glass_post_hoc_review_hours`.

### ConflictClassCatalogV1
The stack MUST define one canonical logical artifact family for `ConflictClassCatalogV1`.

It MUST include at minimum:
- `schema_version`
- `conflict_class_catalog_id`
- `conflict_classes`
- `default_recusal_behavior`
- `override_path`
- `disclosure_required`.

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

### 7.1 `authority-delegation`
Should own the primary role, delegation, approval, and conflict profile families.

### 7.2 `verification-control`
Should consume these profiles when opening or adjudicating control cases.

### 7.3 `effect-runtime` and `continuity-runtime`
Should consume these profiles when effects or continuity exceptions require typed approvals.

---

## 8. Build order

### P3-0 — vocabulary freeze
- freeze the four profile families,
- bind them to existing lease / chain / SoD semantics.

### P3-1 — publication
- publish schemas and examples,
- prove one canonical owner story.

### P3-2 — authority pilot
- prove one happy-path delegated approval,
- prove one blocked self-approval / conflict path.

---

## 9. Explicit non-goals

P3 does **not**:
- replace v22 authority-chain law,
- legalize ambient role inference,
- or permit hidden human-memory approval paths.

---

## 10. Conformance headline

A system conforms to P3 only if, for a materially important path affected by this profile layer, it can answer all of the following without archaeology:

- which profile artifact governed the path,
- which base artifact families it constrained,
- whether an exception or downgrade was used,
- whether the effect remained replayable or auditable,
- and what review or expiry conditions still applied.

If the answer is “that lived in config somewhere,” the system is not yet P3-conforming.

For this document, that means the system can answer at minimum: for a materially important effect, release, or continuity action, which roles were eligible, which approvals were required, what delegation edge was allowed, and what conflict rule blocked or permitted the action.
