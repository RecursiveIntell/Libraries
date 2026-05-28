# Per-crate apply plan — post-v24 profiles

## `stack-ids`
- add the new ID newtypes listed in `snippets/stack_ids_profile_additions.rs`
- expose serde / parsing / formatting in the same style as existing profile IDs
- do not add business logic

## `contract-schema-gen`
- register the 28 new profile schemas
- add per-profile manifests if the generator owns manifest emission
- keep filenames stable and kebab-cased

## `verification-policy`
- own P1 privacy / retention / redaction policy types
- own P2 locality / tenancy / residency policy types
- ensure cross-links to disclosure and admission surfaces remain explicit

## `authority-delegation`
- own P3 role, delegation, approval, and conflict profile types
- expose bindings to existing lease / authority-chain surfaces

## `assurance-runtime`
- own P4 regulated regime mappings and recertification schedule types
- own P5 hazard libraries, scenario catalogs, monitors, and mitigation playbooks

## `attestation-exchange`
- own P6 vendor adapter and external evidence translation types
- bind trust-root and revocation semantics without hiding caveats

## `continuity-runtime`
- own P7 incident taxonomy, severity matrix, pager routes, and escalation clocks
- bind the profile layer to incident cases, containment, recovery, and postmortem surfaces
