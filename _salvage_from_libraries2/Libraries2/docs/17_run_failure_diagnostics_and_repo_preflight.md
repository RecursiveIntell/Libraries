# Run Failure Diagnostics and Repo Preflight

## Purpose

This document addresses the **current highest-value defect class** in the repo:

> runs can fail during the repo/baseline lane, but the app does not explain the failure early enough or clearly enough.

This is the work tracked by:

- `FW-015`
- `FW-016`
- `FW-017`
- `FW-019`
- `FW-021`
- `FW-022`

## Current failure shape

Today the likely sequence is:

1. operator enters a repo path manually,
2. `create_run` queues the run,
3. worker starts baseline capture,
4. repo detection or baseline capture fails,
5. run becomes `failed`,
6. the UI shows failure but with weak diagnostics.

That is backwards. The app should reject obviously bad intake **before** queueing whenever possible.

## Immediate design goal

Split the problem into two layers:

### Layer 1 — intake preflight
Catch invalid repo conditions before the run is queued.

### Layer 2 — run diagnostics
If runtime failure still happens, render the failure class and message directly in Run detail.

## Preflight contract

Add a backend preflight response with fields like:

- `repo_root_input`
- `repo_root_canonical`
- `exists`
- `readable`
- `cargo_toml_present`
- `adapter`
- `workspace_prepare_ok`
- `status`
- `failure_class`
- `message`
- `warnings[]`

### Recommended statuses
- `ready`
- `warning`
- `error`

### Recommended failure classes
- `repo_not_found`
- `repo_not_readable`
- `unsupported_repo`
- `workspace_prepare_failed`
- `path_not_directory`
- `cargo_root_ambiguous`

The exact class names can vary, but they must be precise and stable enough for UI use.

## Where preflight belongs

Preflight should live in the core crate and be callable from:
- intake UI before `create_run`
- `create_run` itself as a final guard
- future retry flows if the repo path changed or became invalid

Do not make preflight a React-only concern.

## New Run UI requirements

### Repo field
Keep a text field for power users, but add:
- folder picker
- canonicalized path display
- preflight result callout
- clear blocking message before queue

### Create button
Disable Create run when preflight is in `error`.

### Warnings
Allow create only for explicit non-fatal warnings, not for fatal failures.

## Run detail diagnostics requirements

When `run.phase === failed`, render:

- failure class
- failure message
- relevant path/provider/model context
- copyable diagnostics block

The event timeline should let the operator expand `payload_json` per row.

## Timeline requirements

The timeline should tell the truth:

- queued
- detecting repo
- capturing baseline
- failed/completed/etc.

If a failure occurred before baseline capture completed, the UI must not imply baseline is simply pending.

## Example failure rendering

### Header callout
**Baseline capture failed**  
`unsupported_repo`  
Expected a Cargo workspace root at `/path/...`

### Timeline row
`repair-run://failed`
- class: `unsupported_repo`
- phase: `detecting_repo`
- repo_root: `/path/...`

## Tests required

### Preflight service
- missing path
- file instead of directory
- unreadable directory
- non-Cargo directory
- valid Cargo root
- nested workspace root
- workspace-prepare failure fixture

### UI
- create button blocks on preflight error
- failure callout renders from `run.error_json`
- timeline payload expander shows structured diagnostics
- baseline panel shows `failed before capture`, not `pending`

## Success condition

A user should be able to answer **“why did this run fail?”** from the desktop UI in one pass, without opening SQLite or reading source code.
