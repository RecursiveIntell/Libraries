# Manual Injection 02 — After Phase 2

Before adding the Turbo adapter, prove:

- semantic-memory default behavior is unchanged;
- SQ8/raw codec behavior remains tested;
- VectorCodec abstraction does not encode Turbo-specific assumptions;
- no TurboQuant math exists in semantic-memory;
- all new config defaults are inert.

If the abstraction is dirty or default behavior changed, repair before proceeding.
