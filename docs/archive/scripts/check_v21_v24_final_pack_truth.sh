#!/usr/bin/env bash
set -euo pipefail

BASE_DIR="$(cd "$(dirname "$0")/.." && pwd)"

required=(
  "README.md"
  "00_START_HERE.md"
  "01_EXECUTIVE_SUMMARY.md"
  "02_SOURCE_BASIS.md"
  "06_FINAL_MASTER_ISSUE_MATRIX.md"
  "07_FINAL_IMPLEMENTATION_SEQUENCE.md"
  "08_FINAL_TEST_AND_CONFORMANCE_PLAN.md"
  "09_FINAL_RELEASE_BAR_AND_GOVERNANCE.md"
  "11_CRATE_OWNERSHIP_AND_BOUNDARY_PLAN.md"
  "12_SCHEMA_REGISTRY_AND_COMPATIBILITY_PLAN.md"
  "14_REFERENCE_INTERPRETER_AND_CONFORMANCE_PLAN.md"
  "20_TERMINAL_DESIGN_POSITION.md"
  "PACK_MANIFEST.json"
)

for f in "${required[@]}"; do
  if [[ ! -f "$BASE_DIR/$f" ]]; then
    echo "missing: $f" >&2
    exit 1
  fi
done

schema_count=$(find "$BASE_DIR/schemas" -maxdepth 1 -name '*.schema.json' | wc -l | tr -d ' ')
example_count=$(find "$BASE_DIR/examples" -maxdepth 1 -name '*.example.json' | wc -l | tr -d ' ')
fixture_count=$(find "$BASE_DIR/contracts/fixtures" -type f -name '*.bundle.json' | wc -l | tr -d ' ')

if [[ "$schema_count" -lt 39 ]]; then
  echo "expected at least 39 schema files, found $schema_count" >&2
  exit 1
fi

if [[ "$example_count" -lt 39 ]]; then
  echo "expected at least 39 example files, found $example_count" >&2
  exit 1
fi

if [[ "$fixture_count" -lt 20 ]]; then
  echo "expected at least 20 fixture bundles, found $fixture_count" >&2
  exit 1
fi

BASE_DIR="$BASE_DIR" python3 - <<'PY'
import json, pathlib, os
base = pathlib.Path(os.environ["BASE_DIR"])
manifest = json.loads((base/"PACK_MANIFEST.json").read_text())
assert manifest["pack"] == "v21_v24_final_closeout_pack"
assert manifest["basis"]["workspace_members"] == 25
assert manifest["draft_schema_files"] >= 39
assert manifest["draft_example_files"] >= 39
assert manifest["fixture_bundle_files"] >= 20

workspace = (base / "Cargo.toml").read_text()
for crate in (
    "effect-runtime",
    "authority-delegation",
    "assurance-runtime",
    "continuity-runtime",
):
    assert f'"{crate}"' in workspace, f"workspace missing {crate}"

schema_dir = base / "schemas"
example_dir = base / "examples"
pack_schema_names = []
for wave in ("v21", "v22", "v23", "v24"):
    wave_manifest = json.loads((base / "contracts" / "schemas" / wave / "manifest.json").read_text())
    assert "owner_crate" in wave_manifest, f"manifest missing owner_crate for {wave}"
    assert "schema_files" in wave_manifest and wave_manifest["schema_files"], f"manifest missing schema_files for {wave}"
    pack_schema_names.extend(wave_manifest["schema_files"])

for schema_name in pack_schema_names:
    schema_path = schema_dir / schema_name
    assert schema_path.exists(), f"missing schema {schema_name}"
    stem = schema_name.replace(".schema.json", "")
    schema = json.loads(schema_path.read_text())
    candidates = [example_dir / f"{stem}.example.json"]
    title = schema.get("title")
    if title:
        candidates.append(example_dir / f"{title}.example.json")
    example_path = next((path for path in candidates if path.exists()), None)
    assert example_path is not None, f"missing example for {schema_name}"
    example = json.loads(example_path.read_text())
    for required in schema.get("required", []):
        assert required in example, f"example {example_path.name} missing required field {required}"
print("manifest and v21-v24 schema/example checks ok")
PY

echo "v21/v24 final closeout pack truth check passed"
