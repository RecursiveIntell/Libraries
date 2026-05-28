# P25 Rollback and Quarantine Plan

## z.py changes

If root Markdown archiving fails or moves protected docs:
1. restore moved files from archive manifest,
2. quarantine the archive manifest,
3. emit violation report,
4. disable strict archive mode until fixed.

## Phase-gate changes

If a phase gate is crossed without injection:
1. mark run violation,
2. quarantine post-gate changes,
3. stop,
4. wait for operator approval.

## Current-run docs

If classification cleanup causes ambiguity:
1. keep old docs,
2. add quarantine record,
3. do not delete,
4. emit unresolved risk.

## Flagship demo

If demo cannot pass replay:
1. mark as experimental/failed,
2. do not claim supported-local demo,
3. emit known limitation.

## Large files

Do not refactor large files in P25 unless required. If a refactor is attempted and fails, revert and record future split plan.
