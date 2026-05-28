# Repo State Audit

## Executive verdict

Forge Workbench now looks like a **real product surface**. The shell, routing, setup flow, provider configuration, and local secret handling are all materially better than the earlier build.

The problem shifted:

- the **UI/control-plane foundation is mostly real**
- the **repair lane is still only baseline-first**
- the **current failure is still happening in the repo/baseline lane**
- the **UI is not yet exposing enough truth when that lane fails**

This matters because the app can now *feel* more complete than it actually is.

## What is already landed

### 1. Control-plane routing and run inspection
The run-detail boundary bug is fixed and the app now routes to concrete run IDs.

**Evidence**
- `apps/forge-workbench/ui/src/hooks/useRunDetail.ts`
- `apps/forge-workbench/src-tauri/src/commands/runs.rs`
- `apps/forge-workbench/ui/src/app/routes.tsx`
- `apps/forge-workbench/ui/src/pages/HomePage.tsx`

### 2. Settings persistence and provider onboarding
Settings are no longer read-only. Provider config, defaults, and model selection live in the core crate and are persisted durably.

**Evidence**
- `crates/forge-workbench-core/src/services/settings_service.rs`
- `crates/forge-workbench-core/src/persistence/control_db.rs`
- `apps/forge-workbench/ui/src/components/SettingsEditor.tsx`
- `apps/forge-workbench/src-tauri/src/commands/settings.rs`

### 3. Secret handling is materially better
Secrets are kept out of SQLite and out of normal `SettingsView` responses.

**Evidence**
- `crates/forge-workbench-core/src/services/secret_store.rs`
- `crates/forge-workbench-core/tests/settings.rs`

### 4. Run creation is now provider/model-aware
`create_run` resolves provider/model in Rust and persists the resolved values onto the run.

**Evidence**
- `crates/forge-workbench-core/src/services/settings_service.rs`
- `crates/forge-workbench-core/src/app/state.rs`
- `apps/forge-workbench/ui/src/pages/NewRunPage.tsx`

## What is still wrong

### 1. Setup readiness is too permissive
`SettingsService::compute_setup_state` does **not** require `connection_status == Ready` or a successful provider test. A provider can therefore be treated as usable even when it has never been validated successfully.

**Why it matters**
- the UI claims provider setup is a hard prerequisite
- the backend does not fully enforce that claim yet
- later run failures will be harder to diagnose because the app allowed them too early

**Primary fix**
- `FW-013`
- `FW-014`
- `FW-047`

### 2. Run intake is too trusting
`AppState::create_run` currently queues a run before proving that the path exists, is readable, and is actually a Cargo workspace root.

**Why it matters**
- the app lets an operator type an invalid repo root
- the failure then happens later in the worker
- that makes the product feel less deterministic than it should

**Primary fix**
- `FW-015`
- `FW-016`

### 3. The current failure is poorly surfaced
The screenshot shows a failed run, but the Run page still hides the actual failure details that are already present in `run.error_json` and event payloads.

**Why it matters**
- the operator knows it failed
- the operator does not know *why* it failed without going spelunking
- this is precisely the kind of trust gap that hurts a workbench

**Primary fix**
- `FW-017`
- `FW-021`
- `FW-022`

### 4. The worker phase model is misleading
`RepairRunJob::execute` updates `DetectingRepo`, then runs baseline capture, then only sets `CapturingBaseline` *after* the baseline succeeded, and then marks the run `Completed`.

**Why it matters**
- the timeline does not match reality
- the phase labels are hard to trust
- baseline-only success is currently mislabeled as full completion

**Primary fix**
- `FW-019`
- `FW-020`

### 5. The app is still baseline-first
The worker stops after baseline capture. `RunDetails` still returns:
- `candidates: Vec::new()`
- `verification: None`
- `export_import: None`
- `operator_actions: Vec::new()`

**Why it matters**
- the product message is now ahead of the implementation
- the missing pieces are not cosmetic; they are the core repair loop

**Primary fix**
- `FW-023` through `FW-045`

## The current screenshot: what it most likely means

The run shown in the latest screenshot has:

- a resolved provider/model
- a queued run
- a progress event
- a terminal failure event
- no baseline snapshot

Given the current code path, that means the failure almost certainly occurred **before or during baseline capture**.

That narrows the likely causes to things like:

- invalid repo root
- non-Cargo root
- workspace preparation failure
- check-command failure severe enough to abort baseline capture

It is **not** evidence that the LLM provider/model path failed. The worker does not reach model candidate generation yet.

## Recommended order from the current state

1. make setup readiness strict and truthful
2. add repo preflight before queueing
3. expose failure diagnostics on the Run page
4. repair the phase/timeline semantics
5. stop calling baseline-only success “completed”
6. only then land candidate generation and verification

## Summary

The foundation is good enough now that every next issue should optimize for **truth**, not for more decoration.

The app no longer needs a prettier shell first.

It needs:
- stricter readiness,
- earlier repo validation,
- more honest failure rendering,
- a truthful state model,
- then the real candidate/verification lane.
