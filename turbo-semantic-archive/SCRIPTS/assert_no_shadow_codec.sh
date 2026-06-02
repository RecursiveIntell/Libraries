#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
SM="$ROOT/semantic-memory"

if [[ ! -d "$SM" ]]; then
  echo "ERROR: semantic-memory directory not found under $ROOT" >&2
  exit 2
fi

echo "[check] no shadow TurboQuant implementation in semantic-memory"

bad=0

# These names are allowed in adapter/tests/docs, but not as local algorithm structs.
if grep -RIn --include='*.rs' \
  -E 'struct (TurboQuantizer|PolarQuantizer|QjlQuantizer|StoredRotation)|fn generate_orthogonal|StandardNormal.*sample|ChaCha8Rng::seed_from_u64' \
  "$SM/src" | grep -v 'vector_codec/turbo' ; then
  echo "ERROR: possible local TurboQuant math implementation detected in semantic-memory/src" >&2
  bad=1
fi

# Adapter may refer to turbo_quant crate. It should not define algorithm internals.
if grep -RIn --include='*.rs' -E 'mod polar|mod qjl|mod rotation|mod turbo' "$SM/src"; then
  echo "ERROR: semantic-memory appears to define TurboQuant internal modules" >&2
  bad=1
fi

exit "$bad"
