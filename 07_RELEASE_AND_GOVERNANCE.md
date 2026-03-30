# 07_RELEASE_AND_GOVERNANCE

## Release bar

A release is publishable only when all of the following are simultaneously true:

1. the front-door pack is complete and `check_pack_truth.sh` passes
2. the archive manifest passes
3. the dashboard describes the current repo truth, not a historical snapshot pretending to be current
4. the support lane, Makefile lane, and receipt lane are the same lane
5. canonical specs are either real canonical docs or clearly labeled excerpts
6. degraded execution reasons survive to runtime-facing artifacts
7. “source-clean” contains no build outputs

## Governance rules

### 1. One taught story
Compatibility paths may exist.
They must not own the user's mental model.

### 2. Narrow guards must have narrow names
If a script only checks `contract-schema-gen`, do not label the result as repo-wide panic safety.

### 3. Historical truth must be labeled as historical truth
If a status or receipt was true on 2026-03-22 but is not reproducible from HEAD, say so.
Do not relabel that as “green now.”

### 4. Crate names must match crate substance
A schema bundle is not a runtime just because it wants to be one someday.

### 5. Horizon work stays fenced
Do not poison the finish line by smuggling v10 geometry into release-hardening work.
