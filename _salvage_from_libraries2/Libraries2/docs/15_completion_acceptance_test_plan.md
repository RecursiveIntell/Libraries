# Completion Acceptance Test Plan

## Purpose

This is the release contract for finishing Forge Workbench from the current repo state.

The repository already proves some foundational behaviors. This plan focuses on what must be added or tightened so the app can ship as a **Verified Rust Repair Agent**, not just as a polished baseline browser.

## Release rule

Do not call the product release-ready until:

- all open P0 issues are closed,
- the acceptance matrix below passes,
- the manual smoke path passes,
- docs and readiness badges match the real shipped capability.

## A. Foundation regression suite

These tests protect already-landed work and must keep passing throughout implementation.

### A1. Routing and run detail
- open an existing run by `run_id`
- load run details successfully
- load run events successfully
- create a new run and land on its detail page
- open historical runs without switching to the newest run

### A2. Settings persistence
- save settings
- reopen app
- verify defaults and provider metadata survive restart
- verify `SettingsView` is redacted

### A3. Secret storage
- verify no plaintext secrets in SQLite
- verify `clear_provider_secret` removes runtime usability
- verify fallback mode is opt-in only

## B. Setup and provider hardening

### B1. Setup readiness truth
Scenarios:
- provider enabled but not tested → setup blocked
- provider test failed → setup blocked
- provider ready + model confirmed → setup ready
- provider becomes stale after base URL change → setup blocked
- provider becomes stale after secret change → setup blocked

### B2. Provider/model resolution
Scenarios:
- default provider + default model
- default provider + profile model
- run model override
- run provider override with provider-specific selected model
- invalid mixed-provider model inheritance is rejected

### B3. Provider integration fixtures
Per provider, mock or fixture:
- successful model list
- auth failure
- bad base URL
- redacted error surface

## C. Repo preflight and intake

### C1. Preflight service
Must cover:
- missing path
- path is a file
- unreadable directory
- non-Cargo directory
- valid Cargo repo
- nested workspace root
- workspace-preparation failure

### C2. Intake UI
Must prove:
- Create run disabled on fatal preflight failure
- canonical path displayed
- blocking reason displayed
- folder picker populates repo field cleanly

## D. Run-lane truth and diagnostics

### D1. Failed-run diagnostics
Must prove:
- `run.error_json` failure class and message render
- timeline payloads are expandable
- diagnostics can be copied/exported if implemented

### D2. Phase semantics
Must prove:
- persisted phase order matches actual worker order
- baseline-only success is not called `completed`
- failed-without-baseline shows `failed before capture`, not `pending`

### D3. Live progress
Must prove:
- active run progress updates without manual refresh
- coalesced timeline remains readable under event bursts

## E. Candidate generation and review

### E1. Candidate persistence
Must prove:
- candidates persist durably
- candidate statuses survive restart
- diagnostics survive restart

### E2. Model output hardening
Must prove:
- parse fail → retry → success
- validation fail → retry → success
- retry exhaustion is bounded and visible

### E3. Patch policy
Must prove:
- forbidden path touches are rejected
- size caps are rejected
- invalid candidates never reach verification

### E4. Candidates UI
Must prove:
- summary list renders
- diff preview renders
- touched symbols render
- invalid/recommended/rejected states render distinctly

## F. Verification and decisioning

### F1. Paired verification
Fixture types:
- candidate improves failing tests and introduces no regressions
- candidate is a no-op
- candidate regresses previously passing checks
- candidate breaks formatting or clippy while fixing tests

### F2. Verdict logic
Must prove:
- verified improvement → recommended
- regression → rejected
- invalid candidate → not eligible
- explanation payload stays structured and concise

### F3. Gates
Must prove:
- unverified candidate cannot be approved for apply
- rejected candidate cannot be applied
- UI bypass cannot defeat backend gates

## G. Memory and audit

### G1. Canonical import path
Must prove:
- verified run exports
- bridge transform succeeds
- memory import succeeds
- import status persists on the run

### G2. Retrieval
Must prove:
- later similar run retrieves prior incident in scope
- out-of-scope run does not retrieve irrelevant data
- normal retrieval does not expose raw evidence

### G3. Audit mode
Must prove:
- audit queries return evidence references explicitly
- audit data stays out of normal retrieval UI
- authority-boundary copy remains visible

## H. Apply and recovery

### H1. Final apply
Must prove:
- only verified + approved candidate can apply
- apply writes the expected diff to a temp repo
- cancellation before apply prevents write
- rejected/unverified candidate cannot apply

### H2. Recovery
Must prove:
- restart after queued state
- restart after active work
- stale/orphaned job reclaim
- no duplicate execution on restart

### H3. Cancellation
Must prove:
- cancel queued run
- cancel active baseline capture if supported
- cancel later verification/apply phases
- terminal cancelled state renders truthfully

## I. Security regressions

### I1. Storage
- SQLite contains no plaintext secrets
- normal exported views contain no plaintext secrets
- explicit fallback files exist only when configured

### I2. Logging
- provider-test failures do not log secrets
- settings-save failures do not log secrets
- model-refresh failures do not log secrets
- run events do not carry secrets

### I3. Error rendering
- UI error callouts never echo raw secrets
- copyable diagnostics never include secrets

## J. Manual smoke path

Run this manually before release:

1. fresh app data directory
2. first-run setup
3. configure Ollama or a mocked cloud provider
4. validate provider and refresh models
5. choose a valid Cargo repo via folder picker
6. preflight and create run
7. inspect baseline capture
8. inspect candidates
9. inspect verification result
10. inspect memory and audit panels
11. approve a verified candidate
12. apply to a temp repo copy
13. restart app and confirm recovery state remains coherent

## K. Commands and checks

When toolchain and environment are available, run:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

Then run any fixture harnesses or smoke scripts introduced by:
- `FW-043`
- `FW-044`
- `FW-045`

## Exit condition

Release is acceptable only when the acceptance matrix passes **and** the manual smoke path confirms that the product surface and the actual lane now tell the same story.
