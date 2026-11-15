# R08 route/cost admission gaps — free OpenRouter models (evidence-backed, reuse-first)

**Date:** 2026-11-13  
**Authority context:** `CAMPAIGN_AUTHORITY_ADDENDUM_02.md` authorizes **$0.00 paid spend** / free OpenRouter models only, supervised and fail-closed; campaign admission/enforcement at canonical physical-send owners remains **not admitted**. `APPROVAL_ADDENDUM.md` remains frozen historical evidence. **RSI activation OFF.**

This note records inspected existing owners/surfaces and the concrete gaps that must be closed before any model-backed bounded-RSI step is claimed admitted. It creates no new repos/crates, no provider configuration, no live grants, and no network calls.

## Existing surfaces inspected (reuse, do not recreate)

### Ares (provider/routing/model metadata)
- **OpenRouter provider/model support exists:** `hermes_cli` contains OpenRouter configuration/model selection surfaces; `hermes_cli/models.py` includes OpenRouter fetch/pricing helpers (`fetch_openrouter_models`, `_fetch_openrouter_reasoning_caps`, `_openrouter_model_is_free`, `_is_model_free`) and free-model labeling/caching behavior for supported providers.
- **Free-tier/local classification helpers exist:** `agent/credits_tracker.py::is_free_tier_model` is a local-data-only heuristic (`:free` suffix, `stealth/` prefix, optional pricing-cache peek; fail-open to `False` on uncertainty). This is useful as a **non-authoritative hint**, not as durable monetary enforcement.
- **Runtime dispatch governor plugin exists in candidates:** `runtime-integration-r08/Ares/plugins/dispatch_governor/__init__.py` commits durable `send_started`/settlement transitions through `SessionDB` execution records in enforce/observe modes and can deny on missing/terminal/unqualified records. This is relevant to egress observation/admission plumbing but must be composed with the authoritative campaign/monetary owners.

### Native recursive-agent (policy/authority/budget)
- **Campaign monetary accounting primitives exist:** `native-composition-r08/recursive-agent/crates/recursive-agent-policy/src/campaign_inference.rs` implements V2 campaign inference accounting (association/root/terms-checked reservations, once-only send/settlement transitions, conservative charging/recovery paths) with qualified freeze evidence in `campaign-bank-01/qualified.freeze.json`.
- **Terms/limits are rate-based and strict:** `crates/recursive-agent-policy/src/inference_terms.rs` defines `InferenceBudgetLimitsV1` with `input_nanodollars_per_token`, `output_nanodollars_per_token`, `max_total_nanodollars`, and validation that rejects zero sends/rates/totals and overrun quotes. This currently models bounded *positive* monetary reservations, not an explicit zero-paid/free-tier exemption path.
- **Family/root/session control owners exist:** funding/terms/root association and normal-chat/session control modules are present in the same policy crate and were part of prior qualified compositions; any route admission must bind through these owners rather than a parallel store.

## Concrete gaps for “free OpenRouter, $0 paid” admission

1. **Canonical route qualification is undefined:** select/qualify an exact OpenRouter route (base URL/protocol/provider id), model id, API mode, and reasoning-effort mapping permitted by existing Ares/native owners. “Any free OpenRouter model” is operator intent but not yet an exact, enforceable per-send contract.
2. **$0-paid semantics vs existing budget model:** current `InferenceBudgetLimitsV1` validation requires non-zero rates and `max_total_nanodollars > 0`. A truthful free-tier admission must either (a) represent zero paid spend through an explicitly admitted contract extension in the existing owner (with conservative refusal on any paid/unknown billing path), or (b) qualify a bounded positive ceiling as an implementation detail while still enforcing fail-closed on non-free/unknown-cost routes — the latter conflicts with the stated “$0.00 paid spend authorized” boundary unless separately approved. This is an owner-contract decision, not an adapter-level workaround.
3. **Authoritative free/paid determination:** `is_free_tier_model` and pricing-cache “free” labels are not sufficient alone for enforcement. Admission needs a canonical, fail-closed basis per exact route/model/send (e.g. verified `:free` model identity + qualified route constraints + no paid fallback), with unknown pricing/route/billing treated as **not zero-cost**.
4. **Physical-send boundary binding:** every actual physical send (including retries, auxiliaries, tool/model subcalls as applicable) must reserve/release/settle through the canonical monetary/execution owners before bytes leave the process. The Ares dispatch governor and native campaign accounting must be composed so no path can bypass the monetary gate; current evidence does not establish end-to-end $0/free OpenRouter enforcement.
5. **Exact request/tariff evidence:** terms require a `tariff_digest` and fixed bounds; for free routes the retained evidence must honestly represent why paid spend is impossible/blocked for the admitted route, or the campaign cannot be truthfully admitted.
6. **Rate/availability limits are operational, not monetary:** free OpenRouter limits/throttling/queueing must fail closed or be explicitly bounded by admitted nonmonetary controls; lack of paid spend does not authorize unlimited attempts.
7. **No shared config mutation:** any admission must avoid mutating installed/shared provider credentials/config/routing defaults; use isolated, receipt-backed qualification artifacts and explicit operator-approved runtime configuration only if/when admitted.

## Recommended next concrete engineering step (reuse-only)

1. Define a minimal **admitted free-route contract** in existing owner terms (exact provider/model/api mode/reasoning effort/request bounds/tariff evidence) for one or a small explicitly named set of OpenRouter `:free` models, with fail-closed refusal on missing/paid/unknown pricing or route mismatch.
2. If zero-rate/zero-ceiling cannot be represented honestly by current `InferenceBudgetLimitsV1`, extend the **existing policy owner** in an isolated candidate with explicit tests/receipts (no shadow bank), preserving all prior invariants and conservative charging semantics.
3. Bind that contract to the existing Ares physical-send/dispatch-governor and native reservation/settlement path, then prove offline/receipt-backed refusal and (only if safely admissible) a bounded supervised canary path — without activation, install, or shared config changes.

**No model-backed campaign step is admitted by this document.** Unknown cost/route/billing remains non-zero and non-authorizing until canonical enforcement is proven.