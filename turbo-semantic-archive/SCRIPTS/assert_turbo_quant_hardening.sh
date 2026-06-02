#!/usr/bin/env bash
set -euo pipefail

TQ="${1:-./turbo-quant}"

if [[ ! -d "$TQ/src" ]]; then
  echo "ERROR: turbo-quant src not found at $TQ" >&2
  exit 2
fi

echo "[check] turbo-quant hardening surfaces"

required_patterns=(
  "TurboQuantCodecProfileV1"
  "EncodedVectorArtifactV1"
  "profile_digest"
  "prepare_query"
  "cosine"
  "checksum"
  "ProfileMismatch"
)

bad=0
for pat in "${required_patterns[@]}"; do
  if ! grep -RIn --include='*.rs' "$pat" "$TQ/src" "$TQ/tests" >/dev/null; then
    echo "ERROR: missing required pattern in turbo-quant: $pat" >&2
    bad=1
  fi
done

exit "$bad"
