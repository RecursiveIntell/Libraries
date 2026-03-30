# Release gates and acceptance

## New constitutional gates that must exist

1. `check_commit_permit_paths.py`
   - fails if any effectful execution path does not require the permit type
2. `check_constitutional_vocab.py`
   - fails on uncontrolled controlled-vocab `String` fields
3. `check_mandatory_artifact_refs.py`
   - fails when promotable/risk-bearing outputs lack required verification plans or execution/episode refs
4. `check_generated_surface_admission.py`
   - fails if generated spec surfaces can affect runtime without proof/veto/challenge closure
5. hotspot budget check
6. panic audit with production-only scope
7. conformance/refint differential suite

## Front-door rule

`make gate` and CI are not allowed to stay green while any of the following are true:
- raw approval strings still exist,
- direct execution without a permit still exists,
- controlled-vocab strings are still open,
- promotable/risk-bearing outputs can skip verification plans,
- endstate-only claims are made without the corresponding artifact families.

## Suggested gate order

1. format + clippy
2. constitutional lints
3. owner-crate unit tests
4. differential conformance
5. end-to-end blocked-execution tests
6. hotspot/panic checks
7. release receipt generation only after everything above is green
