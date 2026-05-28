# Phase 02 injection — testkit topology repair

Make `aidens-testkit` pure or split it.

Required:

- remove normal production crate deps from `aidens-testkit`;
- move production-integrating tests into `aidens-integration-tests`, root tests, or package-local tests;
- preserve hostile/reference tests instead of deleting them.

Acceptance:

- dependency audit reports no production normal deps inside `aidens-testkit`;
- cargo test still includes the moved integration tests.
