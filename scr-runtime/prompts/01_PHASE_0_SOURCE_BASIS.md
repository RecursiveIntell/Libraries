# Phase 0 — Source Basis and Quarantine

Do not implement code in this phase.

Produce:

```text
docs/SOURCE_BASIS.md
docs/CANONICAL_OWNERS.md
docs/NON_GOALS.md
docs/QUARANTINED_TERMS.md
```

## Required content

`docs/SOURCE_BASIS.md` must identify:

- target repository root
- workspace layout
- Rust workspace status
- existing lint/test conventions
- existing schema generation conventions
- existing ID/artifact/provenance/receipt/time/error crates or modules
- whether this is standalone or part of a larger workspace
- assumptions and unresolved ambiguities

`docs/CANONICAL_OWNERS.md` must map:

```text
Concept                  Canonical owner or adapter plan
IDs                      ...
Artifacts                ...
Evidence references      ...
Provenance references    ...
Receipts                 ...
Policies                 ...
Schemas                  ...
Time/bitemporal fields   ...
Errors                   ...
```

If unknown, write `UNKNOWN` and create `SourceTruthAmbiguityRecord`.

`docs/QUARANTINED_TERMS.md` must list forbidden production terms:

- FEUT
- EEG
- theta/alpha/gamma constants
- intelligence field
- P = NP
- Clay proof
- Riemann proof
- universal entropy law
- black-hole echo
- neuro-calibrated constants

## Gate

Do not proceed until these docs exist.
