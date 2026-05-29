# Risk Register

Known unresolved risks:

- q8 quality is validated only on synthetic fixtures; no real model quality or attention drift evidence exists.
- Manifest JSON schema proposal remains a proposal and is not generated from Rust types.
- Optional TurboQuant/FibQuant adapters are unsupported stubs until external APIs are inspected.
- `cargo-semver-checks` is not installed in this environment.
- Crate-name availability was checked by `cargo search`, but publish remains out of scope and requires operator approval.

Release blockers:

- No publish approval.
- No real model benchmark evidence.
- No external adapter API inspection.
- No semver check result.
