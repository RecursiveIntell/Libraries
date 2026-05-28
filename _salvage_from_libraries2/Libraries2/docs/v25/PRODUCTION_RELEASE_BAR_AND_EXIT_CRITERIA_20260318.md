# Production release bar and exit criteria — 2026-03-18

## Exit bar

This closure pass is only complete if all of the following are true:

- `effect-runtime` has no raw `String` IDs for its owned IDs,
- `effect-runtime`, `verification-control`, `verification-policy`, and `verification-adjudication` all cite one composite constitutional lane,
- `remote-oracle-admission` and `federated-settlement` preserve local constitutional citations,
- every touched external artifact has schema + example + test coverage,
- the no-local-recomposition gate passes,
- the final production-closure gate passes,
- `.github/workflows/ci.yml` runs the same checks,
- cargo-backed schema generation and tests pass,
- and `libraries-source/` is synced from the active repo root.

## Disqualifying shortcuts

Any of the following means the pass is **not** done:

- effect artifacts still use raw `String` IDs,
- policy decisions cite only free-form strings instead of the v25 lane IDs,
- remote admission or settlement artifacts lose the local constitutional context,
- missing example JSON is excused as “obvious,”
- CI omits the no-local-recomposition check,
- or a new helper quietly reintroduces raw profile handling into a consumer.
