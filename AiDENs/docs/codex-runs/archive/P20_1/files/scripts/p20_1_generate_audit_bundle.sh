#!/usr/bin/env bash
set -euo pipefail
OUT="target/aidens-p20-1-final-audit"
mkdir -p "$OUT"
{
  date -u
  echo "root=$(pwd)"
  command -v rustc || true
  command -v cargo || true
  rustc --version 2>/dev/null || true
  cargo --version 2>/dev/null || true
} > "$OUT/toolchain.txt"
python3 scripts/p20_1_hard_code_audit.py --out "$OUT/hard-code-audit.json" --markdown "$OUT/hard-code-audit.md" || true
python3 scripts/p20_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl > "$OUT/agency-eval-validation.log" 2>&1 || true
if command -v cargo >/dev/null 2>&1; then
  cargo fmt --all --check > "$OUT/fmt.log" 2>&1 || echo $? > "$OUT/fmt.exit"
  cargo check --workspace --all-targets --all-features > "$OUT/check.log" 2>&1 || echo $? > "$OUT/check.exit"
  cargo test --workspace --all-targets --all-features > "$OUT/test.log" 2>&1 || echo $? > "$OUT/test.exit"
  cargo clippy --workspace --all-targets --all-features -- -D warnings > "$OUT/clippy.log" 2>&1 || echo $? > "$OUT/clippy.exit"
  cargo metadata --format-version=1 > "$OUT/cargo-metadata.json" 2> "$OUT/cargo-metadata.stderr" || true
  cargo tree --workspace > "$OUT/cargo-tree.txt" 2> "$OUT/cargo-tree.stderr" || true
fi
cat > "$OUT/README.md" <<'EOF'
# AiDENs P20.1 final audit bundle

This bundle records package integrity, agency eval validation, and cargo gate outputs where available.
EOF
echo "Wrote $OUT"
