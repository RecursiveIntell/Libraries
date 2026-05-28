# P20.1 Implementation Playbook

## Phase order

1. Package/code audit baseline.
2. Restore missing fixture/support files.
3. Split or repair `aidens-testkit` topology.
4. Harden ownership scanner.
5. Run cargo gates and fix compile/test failures.
6. Revalidate runner/provider/agency behavior.
7. Generate final audit bundle and archive integrity proof.

## Do not do

- Do not add new architecture.
- Do not expand providers unless tests are implemented.
- Do not rewrite canonical library behavior locally.
- Do not make docs greener than code.

## Preferred concrete fix for testkit

Create:

```text
crates/aidens-testkit/             # pure reference interpreter + static fixtures only
crates/aidens-integration-tests/   # integration crate, no library API required
```

Move tests that import production crates to `aidens-integration-tests/tests/` or root `tests/`.

## Preferred concrete fix for agency evals

Use the supplied `evals/p20_agency_eval_cases.jsonl` as a minimum seed. Codex may adjust expected outcomes if code behavior proves different, but it must not delete the eval family.

## Preferred concrete fix for scanner

The ownership scanner should calculate:

```json
{
  "canonical_inventory_available": true,
  "canonical_type_count": 123,
  "aidens_contracts_type_count": 185,
  "duplicate_findings": []
}
```

If `canonical_type_count == 0`, the scanner must fail unless explicitly running in `--aidens-overlay-only` mode.
