# Canonical Stack Profile Spec P5 — Sector Hazard Library, Monitor Catalog, and Mitigation Playbook

**Status:** Proposed post-v24 profile supplement  
**Relationship to v6-v24:** All prior law remains in force. This document does not weaken authority asymmetry, bitemporality, execution evidence, assurance law, or continuity law.  
**Scope:** profile-layer typed overlays for sector hazard library, monitor catalog, and mitigation playbook.

---

## 0. Purpose

This document defines a profile-layer target state after the general-purpose v24 closeout.

### 0.1 Design basis (non-normative)

This supplement is synthesized from v23 hazard and monitoring surfaces, v24 continuity artifacts, and the explicit backlog item for sector-specific hazard libraries.

The design basis compresses to one sentence:

> The stack MUST treat sector-specific hazard doctrine, monitor catalogs, and mitigation playbooks as typed overlay law rather than static wikis.

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

The stack MUST treat sector-specific hazard doctrine, monitor catalogs, and mitigation playbooks as typed overlay law rather than static wikis.

This profile layer is lawful only if it remains:
- typed,
- replayable where materially important,
- reviewable,
- bounded by expiry or compatibility windows where exceptions exist,
- and explicitly subordinate to the existing base law.

---

## 3. Profile artifact law

### HazardLibraryV1
The stack MUST define one canonical logical artifact family for `HazardLibraryV1`.

It MUST include at minimum:
- `schema_version`
- `hazard_library_id`
- `sector`
- `hazard_families`
- `scoring_model_ref`
- `linked_operating_envelopes`.

### HazardScenarioV1
The stack MUST define one canonical logical artifact family for `HazardScenarioV1`.

It MUST include at minimum:
- `schema_version`
- `hazard_scenario_id`
- `hazard_library_id`
- `trigger_conditions`
- `affected_surfaces`
- `severity_baseline`
- `required_monitor_refs`.

### MonitorCatalogV1
The stack MUST define one canonical logical artifact family for `MonitorCatalogV1`.

It MUST include at minimum:
- `schema_version`
- `monitor_catalog_id`
- `monitor_definitions`
- `evaluation_cadence`
- `false_positive_budget`
- `owner_ref`.

### MitigationPlaybookV1
The stack MUST define one canonical logical artifact family for `MitigationPlaybookV1`.

It MUST include at minimum:
- `schema_version`
- `mitigation_playbook_id`
- `hazard_refs`
- `containment_steps`
- `recovery_steps`
- `approval_refs`
- `success_criteria`.

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
Should own the primary hazard, scenario, monitor, and playbook families.

### 7.2 `continuity-runtime`
Should consume these profiles when incidents, containment, and recovery are triggered by hazard conditions.

### 7.3 `mechanism-runtime`
May consume these profiles when theory or mechanism families must disclose hazard-specific monitors or failure modes.

---

## 8. Build order

### P5-0 — vocabulary freeze
- freeze the four profile families,
- bind them to existing monitoring and continuity surfaces.

### P5-1 — publication
- publish schemas and examples,
- keep hazard libraries queryable and non-decorative.

### P5-2 — activation pilot
- prove one hazard-triggered playbook activation,
- prove monitor linkage and recovery criteria remain explicit.

---

## 9. Explicit non-goals

P5 does **not**:
- replace generic hazard or assurance law,
- create a new incident plane,
- or permit hazard labels with no monitor or mitigation binding.

---

## 10. Conformance headline

A system conforms to P5 only if, for a materially important path affected by this profile layer, it can answer all of the following without archaeology:

- which profile artifact governed the path,
- which base artifact families it constrained,
- whether an exception or downgrade was used,
- whether the effect remained replayable or auditable,
- and what review or expiry conditions still applied.

If the answer is “that lived in config somewhere,” the system is not yet P5-conforming.

For this document, that means the system can answer at minimum: for a materially important hazard-bearing release or incident, which hazard library and scenario governed it, which monitors were required, and which mitigation playbook was activated.
