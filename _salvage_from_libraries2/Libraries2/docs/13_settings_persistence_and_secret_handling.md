# Settings Persistence and Secret Handling

## Purpose

This document defines the settings and secret-handling contract for Forge Workbench as it moves from “good local shell” to “ship-quality local workbench”.

The repository already landed the core idea correctly:

- provider metadata goes in SQLite,
- secrets do not,
- the core crate owns the save/load logic,
- the UI receives a redacted `SettingsView`.

Now the work is to harden edge cases and lock down regressions.

## Current landed contract

### Durable settings
The control DB has durable tables for:

- `app_settings`
- `provider_configs`

### Current provider-config fields
- `provider`
- `enabled`
- `base_url`
- `selected_model_id`
- `has_secret`
- `secret_storage_mode`
- `connection_status`
- `last_tested_at`
- `last_error_redacted`
- `models_json`

### Current storage behavior
- SQLite stores non-secret metadata only.
- Keyring is the default secret backend.
- Explicit fallback files exist only when configured.
- `SettingsView` returns redacted provider state.

## Non-negotiable invariants

- No plaintext API keys in SQLite.
- No plaintext API keys in normal Tauri responses.
- No plaintext API keys in normal event payloads.
- No plaintext API keys in logs.
- Secrets are cleared explicitly and immediately.
- Any insecure fallback path must be explicit, visible, and test-covered.

## Backend ownership

All save/load/clear operations stay in the core crate:

- `AppState::save_settings`
- `SettingsService::save_settings`
- `SettingsService::clear_provider_secret`
- `SecretStore`

Tauri commands stay thin wrappers.

## Keyring-first policy

The desired default behavior is:

1. write secret to OS keyring,
2. store only metadata in SQLite,
3. expose `has_secret` and storage mode in redacted views,
4. never echo the secret back.

The current repo already follows that design.

## Explicit fallback policy

Fallback storage is allowed only when the app config explicitly enables it.

Requirements:

- fallback is opt-in, not silent default
- fallback location is clearly scoped under app data
- fallback files are private to the local user where the platform supports it
- fallback mode is visible in diagnostics and settings
- fallback remains covered by the same no-leak tests as keyring mode

## Save semantics

### Save must be typed
Normal settings updates must remain typed structs, not loose JSON.

### Save must be merging, not destructive
Saving one provider must not silently wipe unrelated provider state.

### Save must preserve redaction
A successful save returns a redacted `SettingsView`, never a write-echo of the raw secret.

### Save must update readiness carefully
Changing connectivity-relevant inputs should mark provider status stale/not-tested.
Changing unrelated fields should not wipe a previously meaningful readiness state.

That last rule is the main remaining hardening gap and maps to `FW-014`.

## Clear-secret semantics

Clearing a secret must:

- delete the keyring entry if present
- delete any explicit fallback file if present
- mark `has_secret = false`
- update provider readiness so setup can no longer report ready if that provider required credentials
- never leave stale secret-presence state behind

The current repo already covers the core path, but final release still needs dedicated security regressions.

## Redaction rules

### Allowed in `SettingsView`
- provider kind
- base URL
- enabled state
- selected/default model IDs
- redacted connection status
- redacted failure message
- secret presence boolean
- secret storage mode

### Forbidden in `SettingsView`
- raw API keys
- partial API keys
- secret hashes or secret-derived fingerprints that are user-misleading
- raw provider request bodies if they could include credentials

## Logging rules

These are non-negotiable:

- provider-test failure logs must not contain secrets
- settings-save errors must not contain secrets
- model-refresh errors must not contain secrets
- event payloads must not contain secrets
- release tests must scan representative logs for forbidden tokens

This is tracked by `FW-042`.

## Schema expectations going forward

The current schema is enough for v1 if the following are added or clarified as needed:

- provider readiness freshness
- provider-scoped profile-model precedence or equivalent validation path
- catalog refresh timestamps where needed
- migration safety for any added provider fields

## Tests required

### Already landed
- settings round-trip survives restart
- SQLite does not contain raw secrets
- clearing a provider secret removes runtime usability

### Still required
- secrets never appear in failure logs
- secrets never appear in event payloads
- explicit fallback files never leak into normal views
- provider test errors are redacted
- stale/ready transitions behave correctly after save
- provider override resolution never yields a cross-provider model mismatch

## Issue mapping

- `FW-013` — strict setup gating after validation
- `FW-014` — readiness freshness/stale semantics
- `FW-042` — security regression tests
- `FW-047` — provider/model override validity
