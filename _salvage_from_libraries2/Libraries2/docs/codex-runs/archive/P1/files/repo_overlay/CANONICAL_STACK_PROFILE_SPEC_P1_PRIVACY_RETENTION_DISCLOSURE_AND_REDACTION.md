# Canonical Stack Profile Spec P1 — Privacy, Retention, Disclosure, and Redaction

**Status:** Proposed post-v24 profile supplement  
**Relationship to v6-v24:** All prior law remains in force. This document does not weaken authority asymmetry, bitemporality, execution evidence, assurance law, or continuity law.  
**Scope:** profile-layer typed overlays for privacy, retention, disclosure, and redaction.

---

## 0. Purpose

This document defines a profile-layer target state after the general-purpose v24 closeout.

### 0.1 Design basis (non-normative)

This supplement is synthesized from v15 disclosure and admission law, v19 archive / retention / curriculum truth, and the v21–v24 operational surfaces that need lawful redaction and audit export behavior.

The design basis compresses to one sentence:

> The stack MUST treat privacy, retention, redaction, and audit extraction as typed overlay law rather than a mix of bespoke flags, admin folklore, and external-tool defaults.

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

The stack MUST treat privacy, retention, redaction, and audit extraction as typed overlay law rather than a mix of bespoke flags, admin folklore, and external-tool defaults.

This profile layer is lawful only if it remains:
- typed,
- replayable where materially important,
- reviewable,
- bounded by expiry or compatibility windows where exceptions exist,
- and explicitly subordinate to the existing base law.

---

## 3. Profile artifact law

### PrivacyRetentionProfileV1
The stack MUST define one canonical logical artifact family for `PrivacyRetentionProfileV1`.

It MUST include at minimum:
- `schema_version`
- `privacy_retention_profile_id`
- `applicable_namespaces`
- `default_retention_class`
- `archive_restore_expectation`
- `cross_border_transfer_default`
- `default_redaction_rule_set_id`
- `compaction_requires_receipt`.

### RedactionRuleSetV1
The stack MUST define one canonical logical artifact family for `RedactionRuleSetV1`.

It MUST include at minimum:
- `schema_version`
- `redaction_rule_set_id`
- `target_artifact_families`
- `field_actions`
- `reversibility_class`
- `approval_requirement`
- `default_disclosure_budget_class`.

### AccessPurposeMatrixV1
The stack MUST define one canonical logical artifact family for `AccessPurposeMatrixV1`.

It MUST include at minimum:
- `schema_version`
- `access_purpose_matrix_id`
- `actor_classes`
- `purpose_rules`
- `default_decision`
- `elevation_path`
- `audit_logging_required`.

### AuditExtractionPolicyV1
The stack MUST define one canonical logical artifact family for `AuditExtractionPolicyV1`.

It MUST include at minimum:
- `schema_version`
- `audit_extraction_policy_id`
- `allowed_artifact_families`
- `required_redaction_rule_set_id`
- `disclosure_budget_class`
- `export_package_format`
- `expiry_hours`
- `evidence_preservation_required`.

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
Should own the primary profile types for privacy, retention, redaction, and audit extraction.

### 7.2 `attestation-exchange`
Should consume these profiles when packaging exported bundles or audit extracts.

### 7.3 `constitutional-memory`
Should consume these profiles when archive restore, compaction, or historical query guarantees are affected.

---

## 8. Build order

### P1-0 — vocabulary freeze
- freeze the four profile families,
- assign one owner and one schema path for each.

### P1-1 — policy publication
- publish schemas and examples,
- bind redaction and access-purpose rules to existing disclosure semantics.

### P1-2 — audit extraction pilot
- prove one redacted export path,
- prove one blocked or escalated path,
- preserve evidence lineage.

---

## 9. Explicit non-goals

P1 does **not**:
- redefine disclosure law from v15,
- replace archive law from v19,
- legalize policy-free export,
- or permit permanent ad hoc operator exceptions.

---

## 10. Conformance headline

A system conforms to P1 only if, for a materially important path affected by this profile layer, it can answer all of the following without archaeology:

- which profile artifact governed the path,
- which base artifact families it constrained,
- whether an exception or downgrade was used,
- whether the effect remained replayable or auditable,
- and what review or expiry conditions still applied.

If the answer is “that lived in config somewhere,” the system is not yet P1-conforming.

For this document, that means the system can answer at minimum: for a materially important audit export or retained artifact, which retention class governed it, which redaction rules were applied, which actor-purpose combinations were allowed, and whether evidence-preserving export remained possible.
