Implement the v9 constitutional freeze only.

Scope:
- Freeze canonical Episode / Claim / Evidence package contract.
- Freeze execution-evidence artifact family.
- Publish schemas and compatibility policy through contract-schema-gen.
- Add reference interpreters for bitemporal query semantics, widening semantics, bridge atomicity, and repair invariants.
- Add bridge/import failure artifacts.
- Complete episode-first projection requirements.

Constraints:
- Do not introduce new runtime geometry in this pass.
- Do not contaminate the finish line with federation, subtraction, or mechanism-library work.

Deliver:
- code/spec/doc changes
- tests
- migration notes
- explicit non-goals
