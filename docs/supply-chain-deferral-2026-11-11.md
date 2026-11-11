# Supply-chain advisory deferral record — 2026-11-11

## Operator decision

The operator directed (2026-11-11, "Ares audit closure" session): the two
advisories with available patches (lru RUSTSEC-2026-0253, rmcp
RUSTSEC-2026-0462) are fixed immediately (Libraries #83); the eight
**unmaintained** advisories with no patched versions are **deferred** with
tracked ignores so gates stay green, and are documented here for pickup.

## Deferred findings (all transitive, no safe upgrade available upstream)

| Advisory | Crate v | Reachability (puller chain) | Alternative |
|---|---|---|---|
| RUSTSEC-2025-0141 | bincode 1.3.3 | Direct dep of Primitives/cea-core, fib-quant (poly-kv already on bincode 2) | wincode, postcard, bitcode, rkyv |
| RUSTSEC-2024-0436 | paste 1.0.15 | simba → nalgebra → statrs/fib-quant; scr-runtime-compression | pastey (drop-in fork) |
| RUSTSEC-2024-0370 | proc-macro-error 1.0.4 | gtk/glib-macros 0.18 → tauri v2 (ai-batch-queue, stack-monitor-desktop, tauri-queue) | manyhow |
| RUSTSEC-2025-0075 | unic-char-range 0.9.0 | urlpattern → tauri-utils → tauri v2 | icu_properties |
| RUSTSEC-2025-0080 | unic-common 0.9.0 | same tauri chain | icu_properties |
| RUSTSEC-2025-0081 | unic-char-property 0.9.0 | same tauri chain | icu_properties |
| RUSTSEC-2025-0098 | unic-ucd-version 0.9.0 | same tauri chain | icu_properties |
| RUSTSEC-2025-0100 | unic-ucd-ident 0.9.0 | same tauri chain | icu_properties |

Risk rationale: "unmaintained" advisories flag *no upstream maintenance*, not a
known vulnerability in current use. These crates sit behind tauri v2 (desktop
shells) and nalgebra/statrs (math), with no known exploitable path; every
upstream advisory lists "No safe upgrade is available!".

## Where the ignores live

- `fib-quant/deny.toml` → `[advisories] ignore` (bincode, paste — the two that
  fib-quant's own release receipt gate sees).
- A root `deny.toml` is created by this change (the repo had none; the
  hardening gate does not currently run cargo-deny at root, so this config is
  the canonical recorded policy).

## Exit criteria (when to revisit)

Re-evaluate on any of: tauri v2 major bump (drops gtk3/glib 0.18 → removes
proc-macro-error + unic chain), nalgebra/simba update (drops paste), a
bincode-2 migration of cea-core/fib-quant, or a new vulnerability advisory
against any deferred crate (immediate action then).
Review owner: RecursiveIntell supply-chain lane. Review date: 2027-05 (6mo).
