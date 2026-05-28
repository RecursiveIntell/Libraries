#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
SM="$ROOT/semantic-memory"

if [[ ! -d "$SM/src" ]]; then
  echo "ERROR: semantic-memory src not found at $SM" >&2
  exit 2
fi

echo "[check] semantic-memory integration surfaces"

required_patterns=(
  "VectorCodec"
  "VectorCodecConfig"
  "shadow_turbo"
  "VectorScoreProvenance"
  "ApproximationClass"
  "profile_digest"
  "degradation"
)

bad=0
for pat in "${required_patterns[@]}"; do
  if ! grep -RIn --include='*.rs' "$pat" "$SM/src" "$SM/tests" >/dev/null; then
    echo "ERROR: missing required pattern in semantic-memory: $pat" >&2
    bad=1
  fi
done

# Turbo feature should exist if adapter was implemented.
if grep -RIn --include='*.rs' 'TurboQuantCodec' "$SM/src" >/dev/null; then
  if ! grep -n 'turbo-quant-codec' "$SM/Cargo.toml" >/dev/null; then
    echo "ERROR: TurboQuantCodec present but feature flag missing" >&2
    bad=1
  fi
fi

exit "$bad"
