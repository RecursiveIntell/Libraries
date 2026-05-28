#!/usr/bin/env bash
set -euo pipefail
repo="${1:?usage: bash install_p20_2_overlay.sh /path/to/AiDENs}"
cd "$(dirname "$0")"
mkdir -p "$repo"
# Install run docs/prompts into a namespaced handoff area and root prompt dirs.
mkdir -p "$repo/docs/p20_2" "$repo/prompts/p20_2" "$repo/prompts/phase_injections" "$repo/tasks"
cp -R docs/* "$repo/docs/p20_2/"
cp -R audit "$repo/docs/p20_2/"
cp prompts/P20_2_CODEX_RUN_PROMPT.md "$repo/prompts/P20_2_CODEX_RUN_PROMPT.md"
cp -R prompts/phases "$repo/prompts/p20_2/"
cp -R prompts/phase_injections/* "$repo/prompts/phase_injections/"
cp tasks/P20_2_TASK_MATRIX.json "$repo/tasks/P20_2_TASK_MATRIX.json"
# Install repo overlay files required by the next pass.
if [[ -d repo_overlay ]]; then
  cp -R repo_overlay/. "$repo/"
fi
mkdir -p "$repo/supporting/matrices" "$repo/templates"
cp -R supporting/matrices/* "$repo/supporting/matrices/"
cp -R templates/* "$repo/templates/"
echo "Installed P20.2 overlay into $repo"
