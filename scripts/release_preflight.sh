#!/usr/bin/env bash
set -euo pipefail

crate=${1:?usage: scripts/release_preflight.sh CRATE [COMMIT]}
commit=${2:-HEAD}
root=$(git rev-parse --show-toplevel)
cd "$root"

# A package assembled from local path dependencies is not reproducible if the
# declared requirement does not match the package at that path. Run this before
# checking tree cleanliness so release preflight always reports contract drift.
python3 scripts/check_path_dependency_versions.py "$root"

git rev-parse --verify "${commit}^{commit}" >/dev/null
for mode in unstaged staged; do
  if [[ $mode == unstaged ]]; then
    dirty=$(git diff --name-only -- "$crate" Cargo.toml Cargo.lock)
  else
    dirty=$(git diff --cached --name-only -- "$crate" Cargo.toml Cargo.lock)
  fi
  [[ -z $dirty ]] || { echo "release preflight: $mode relevant changes:" >&2; echo "$dirty" >&2; exit 1; }
done
untracked=$(git ls-files --others --exclude-standard -- "$crate" Cargo.toml Cargo.lock)
[[ -z $untracked ]] || { echo "release preflight: untracked relevant files:" >&2; echo "$untracked" >&2; exit 1; }

git diff --quiet "$commit" -- "$crate" Cargo.toml Cargo.lock || {
  echo "release preflight: relevant tree differs from $commit" >&2
  exit 1
}

cargo package -p "$crate" --locked
package=$(find target/package -maxdepth 1 -type f -name "$crate-*.crate" -printf '%T@ %p\n' | sort -nr | head -1 | cut -d' ' -f2-)
[[ -n $package ]] || { echo "release preflight: package archive not found" >&2; exit 1; }

if [[ -n ${PACKAGE_VERIFY_HOOK:-} ]]; then
  "$PACKAGE_VERIFY_HOOK" "$package" "$commit" "$crate"
else
  echo "release preflight: package-vs-commit hook not configured; set PACKAGE_VERIFY_HOOK" >&2
  exit 1
fi

echo "release preflight passed for $crate at $(git rev-parse "$commit^{commit}")"
