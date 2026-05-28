# DEMO-001: v21 → v22 → v23 stitched operator narrative

This is the single operator-visible demonstration for `effect-runtime` → `authority-delegation` → `assurance-runtime` with a typed end-to-end closure artifact.

## What this demo covers

- `v21`: effect preflight and execution completion.
- `v22`: delegation and acting-on-behalf authority chain.
- `v23`: release assurance and readiness decision.

## Evidence bundle

Run from repo root:

```bash
cargo test -p verification-control --test e2e_effect_authority_assurance_release
```

The stitched bundle is:

- `contracts/fixtures/demo/effect_authority_assurance_release.bundle.json`

It contains only existing schema families from:

- `contracts/fixtures/v21/effect_happy_path.bundle.json`
- `contracts/fixtures/v22/delegated_effect_happy_path.bundle.json`
- `contracts/fixtures/v23/release_happy_path.bundle.json`

## How the chain is typed

1. The v21 execution artifact `EffectExecutionReceiptV1.fxr_001` is validated as authorized and completed.
2. v22 emits `ActingOnBehalfReceiptV1.aob_001` that references the same execution receipt id.
3. v23 emits `ReleaseReadinessDecisionV1.rrd_001` with:
   - `advisory_only == false`
   - `blocking_gaps == []`
   - a non-empty `required_monitors`

These three points prove the chain is a single, replayable operator path rather than a disjoint document collection.

## Manual check

```bash
python3 - <<'PY'
import json
from pathlib import Path

root = Path(".")
bundle = json.loads((root / "contracts/fixtures/demo/effect_authority_assurance_release.bundle.json").read_text())
print(bundle["fixture_name"], bundle["wave"])
print("effects:", bundle["chain"]["effect_to_release"][0]["artifacts"])
print("release decision:", bundle["artifacts"]["ReleaseReadinessDecisionV1"]["release_readiness_decision_id"])
PY

Automated check:

```bash
cargo test -p verification-control --test e2e_effect_authority_assurance_release -- --nocapture
```
```
