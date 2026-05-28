Proceed to Phase 01 only.

Focus: build truth.

Run fmt/check/test/clippy/verify and fix failures. Do not add features. Do not bypass canonical crates to make errors disappear.

Forbidden:

- compatibility shims;
- local replacements for canonical crate behavior;
- docs edits that hide build failures;
- weakening tests to pass.

Required output:

- command outputs/log paths;
- all failures found;
- fixes applied;
- unresolved blockers;
- Phase 01 pass/fail.

Stop after Phase 01.
