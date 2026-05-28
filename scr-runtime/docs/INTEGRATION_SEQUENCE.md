# Integration Sequence

P0A does not integrate SCR into external systems.

Future integration, if approved, should happen in this order:

1. Resolve SCR-specific receipt and time-basis ownership from
   `SourceTruthAmbiguityRecord`.
2. Register any required cross-crate IDs through the existing ID owner.
3. Decide whether SCR schemas remain local or join the existing schema generator.
4. Define adapter traits for upstream evidence, provenance, permit, and artifact
   refs.
5. Add read-only host adapters.
6. Add mutation gates only after receipt replay and golden fixture gates are
   stable.

This pass stops at local reference evaluation and fixture replay.
