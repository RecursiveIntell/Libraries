# P28 Known Limitations Template

## Current limitations

| ID | Limitation | Class | Blocks completion? | Evidence | Follow-up |
|---|---|---|---:|---|---|

## Required semantics

An empty known-limitations register MUST NOT block completion. A non-empty register blocks completion only when at least one current limitation has class `release_blocking` or `support_downgrade` that affects the claimed support tier.

## Limitation classes

- `release_blocking`
- `support_downgrade`
- `draft_only`
- `doc_only`
- `accepted_non_goal`

## Validation

Tests must cover:

- empty register does not block completion
- release-blocking current limitation blocks completion
- retired limitation does not block completion
- waived limitation is visible as waiver, not proof
