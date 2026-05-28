# Provider Setup and Model Selection

## Purpose

This document describes the **current target contract** for provider setup and model selection in Forge Workbench from the repo's present state.

The repository already has:

- durable provider settings
- provider test commands
- model discovery commands
- secret storage
- first-run setup and settings UI
- core-owned provider/model resolution

The remaining work is not “add setup.” The remaining work is to make setup **strict, truthful, and verification-safe**.

## Current landed surfaces

### Backend
- `SettingsService::save_settings`
- `SettingsService::compute_setup_state`
- `SettingsService::resolve_run_execution`
- `ProviderService::test_connection`
- `ProviderService::list_models`

### UI
- `SetupPage.tsx`
- `SettingsEditor.tsx`
- `NewRunPage.tsx`

## Non-negotiable invariants

- Provider/model routing remains in the core crate.
- Setup is not complete until at least one provider is actually usable.
- A provider is not usable just because it is enabled.
- Secrets remain out of SQLite.
- A run override must never silently inherit an incompatible model.
- Ollama remains the first-class local path, but cloud providers remain explicit and opt-in.

## What “setup complete” must mean

A provider can count toward setup completion only if **all** of the following are true:

1. provider configuration exists,
2. provider is enabled,
3. credentials are present if the provider requires them,
4. the provider passed a connection/model-discovery validation path,
5. the selected default model is valid for that provider,
6. the model selection path is still fresh enough to trust.

The current repo satisfies 1, 2, and 3.
It partially satisfies 5.
It does **not** yet enforce 4 or 6 strongly enough.

That gap is tracked by:
- `FW-013`
- `FW-014`

## Required provider status model

The provider lifecycle should be explicit.

Recommended backend states:

- `disabled`
- `not_tested`
- `ready`
- `stale`
- `error`

### Meaning

#### `disabled`
The provider is intentionally unavailable and should not participate in readiness.

#### `not_tested`
The provider is enabled but has never been validated with the current config.

#### `ready`
The provider was successfully tested with the current base URL, current credentials, and current selected model context.

#### `stale`
The provider was previously ready, but something changed that invalidates confidence in the current readiness.

#### `error`
A provider test or model refresh using the current config failed.

## What should cause `stale`

Move to `stale` when any of the following change:

- base URL
- credentials
- provider selection used for defaults
- selected model
- provider-scoped profile model
- model catalog age exceeds the chosen freshness threshold

Do **not** move to `stale` for unrelated settings changes that do not affect provider connectivity or model validity.

This is the core of `FW-014`.

## Model catalog contract

Each provider needs a durable model catalog view with enough metadata to decide whether a model is usable.

Minimum fields:

- `provider`
- `model_id`
- `display_name`
- `available`
- `supports_candidate_generation`
- `supports_explanation`
- `catalog_refreshed_at`
- `source` (`provider_api`, `operator_confirmed_manual_entry`, etc.)

The current repo already stores the first six fields.
The freshness/source additions are the next hardening step.

## Default resolution contract

The current repo stores:

- `default_provider`
- `default_model_id`
- `fast_profile_model_id`
- `standard_profile_model_id`
- `conservative_profile_model_id`
- `providers[].selected_model_id`

This is enough for one-provider flows, but becomes ambiguous when the operator overrides the provider at run creation.

### The bug to avoid

If the default provider is `open_ai` and the `standard_profile_model_id` is an OpenAI model, then a run that overrides the provider to `ollama` must not inherit that OpenAI profile model silently.

That is why `FW-047` exists.

## Recommended resolution precedence

### Without provider override
1. `model_override`
2. profile-specific model for the default provider
3. provider-selected model for the default provider
4. `default_model_id` for the default provider

### With provider override
1. `model_override`
2. profile-specific model **for that overridden provider**
3. provider-selected model for that overridden provider
4. provider-specific default model for that overridden provider

If the repo does not yet store provider-scoped profile models, then provider override must fall back to the overridden provider’s selected/default model and reject any inherited model from another provider.

## Recommended data-model change

Prefer one of these two approaches:

### Option A — provider-scoped profile models
Each provider record owns:
- `selected_model_id`
- `fast_profile_model_id`
- `standard_profile_model_id`
- `conservative_profile_model_id`

This is the cleanest model.

### Option B — single default provider only
Keep profile models global, but disallow provider override unless:
- the operator explicitly supplies `model_override`, or
- the overridden provider has a provider-selected model and the system ignores incompatible global profile defaults.

Option A is cleaner and safer.

## Setup UI requirements

The UI should keep the current structure, but harden semantics:

### Setup page
- show blocking reason
- show readiness state per provider
- show whether the provider is `ready`, `stale`, `not_tested`, or `error`
- show the selected default provider/model pair
- keep run creation blocked until readiness is real

### Settings page
- retain provider configuration controls
- show last successful validation time
- show model catalog refresh time
- make stale status visible
- make model override/preference logic explainable

### New Run page
- show the resolved provider/model pair
- show when a provider override invalidates the current default model
- never allow a create action that would resolve to an incompatible model

## Tests required

### Backend
- untested provider does not satisfy setup completion
- error provider does not satisfy setup completion
- ready provider satisfies setup completion
- stale provider does not satisfy setup completion
- provider override cannot inherit an incompatible profile model
- model override wins when valid for the effective provider

### UI
- setup page stays blocked for `not_tested` and `error`
- setup page unlocks for `ready`
- New Run warns or blocks on incompatible override/model combinations

## Issue mapping

- `FW-013` — strict setup gating
- `FW-014` — readiness freshness/stale model
- `FW-047` — provider-scoped model precedence and override validation
