# Canonical Stack Profile Spec P7 — Incident Taxonomy, Escalation, and Pager Routing Profile

**Status:** Proposed post-v24 profile supplement  
**Relationship to v6-v24:** All prior law remains in force. This document does not weaken authority asymmetry, bitemporality, execution evidence, assurance law, or continuity law.  
**Scope:** profile-layer typed overlays for incident taxonomy, escalation, and pager routing profile.

---

## 0. Purpose

This document defines a profile-layer target state after the general-purpose v24 closeout.

### 0.1 Design basis (non-normative)

This supplement is synthesized from v24 incident and continuity law, the repo’s existing continuity policy profiles, and the explicit backlog item for incident taxonomy and pager-routing profiles.

The design basis compresses to one sentence:

> The stack MUST treat incident classification, severity, pager routing, and escalation timing as typed overlay law rather than external-service-only state.

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

The stack MUST treat incident classification, severity, pager routing, and escalation timing as typed overlay law rather than external-service-only state.

This profile layer is lawful only if it remains:
- typed,
- replayable where materially important,
- reviewable,
- bounded by expiry or compatibility windows where exceptions exist,
- and explicitly subordinate to the existing base law.

---

## 3. Profile artifact law

### IncidentTaxonomyV1
The stack MUST define one canonical logical artifact family for `IncidentTaxonomyV1`.

It MUST include at minimum:
- `schema_version`
- `incident_taxonomy_id`
- `incident_classes`
- `default_routes`
- `required_artifact_families`.

### SeverityMatrixV1
The stack MUST define one canonical logical artifact family for `SeverityMatrixV1`.

It MUST include at minimum:
- `schema_version`
- `severity_matrix_id`
- `severity_rules`
- `customer_impact_rubric`
- `internal_impact_rubric`
- `override_rule`.

### PagerRouteProfileV1
The stack MUST define one canonical logical artifact family for `PagerRouteProfileV1`.

It MUST include at minimum:
- `schema_version`
- `pager_route_profile_id`
- `rotation_refs`
- `handoff_rules`
- `ack_timeout_minutes`
- `max_levels`.

### EscalationClockPolicyV1
The stack MUST define one canonical logical artifact family for `EscalationClockPolicyV1`.

It MUST include at minimum:
- `schema_version`
- `escalation_clock_policy_id`
- `severity_matrix_id`
- `response_clock_minutes`
- `postmortem_clock_hours`
- `pause_rules`
- `exception_path`.

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

### 7.1 `continuity-runtime`
Should own the primary incident taxonomy, severity, routing, and clock-policy families.

### 7.2 `verification-policy`
Should consume these profiles when continuity exceptions or escalation policies need policy-level review.

### 7.3 `llm-tool-runtime`
May consume these profiles when dispatch or notification tooling needs typed route and escalation metadata.

---

## 8. Build order

### P7-0 — vocabulary freeze
- freeze the four profile families,
- bind them to existing continuity artifacts.

### P7-1 — publication
- publish schemas and examples,
- keep taxonomy and route semantics queryable.

### P7-2 — escalation pilot
- prove one happy incident classification path,
- prove one timeout-driven escalation path.

---

## 9. Explicit non-goals

P7 does **not**:
- replace v24 incident law,
- legalize hidden SaaS-only pager semantics,
- or permit unbounded emergency escalation clocks with no review path.

---

## 10. Conformance headline

A system conforms to P7 only if, for a materially important path affected by this profile layer, it can answer all of the following without archaeology:

- which profile artifact governed the path,
- which base artifact families it constrained,
- whether an exception or downgrade was used,
- whether the effect remained replayable or auditable,
- and what review or expiry conditions still applied.

If the answer is “that lived in config somewhere,” the system is not yet P7-conforming.

For this document, that means the system can answer at minimum: for a materially important incident, which taxonomy classified it, which severity rule applied, which pager route was used, and which escalation clocks were running or exhausted.
