# Open Ambiguities and Stop Rules

## Ambiguity classes

1. **Exact owner ambiguity**: more than one canonical crate appears to define the concept.
2. **Maturity ambiguity**: the concept belongs to future stack law but no current crate owns it.
3. **Wrapper ambiguity**: a display/report type contains enough fields to act as truth.
4. **Schema ambiguity**: a generated schema could be either AiDENs-local or canonical.
5. **Digest ambiguity**: a digest is used for display but could be interpreted as artifact identity.

## Required action

For each ambiguity:

- create a quarantine record;
- do not invent local law;
- do not create compatibility shims;
- stop before proceeding if the ambiguity blocks ownership collapse.

## Allowed temporary state

A type may remain only if:

- it is explicitly non-authoritative;
- it has canonical backpointers;
- it is recorded in the final inventory;
- it is not an exact duplicate public type name of a canonical type;
- ambiguity has been quarantined if relevant.
