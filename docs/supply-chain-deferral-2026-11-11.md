# Supply-chain advisory deferral record — 2026-11-11

## Operator decision

The operator directed (2026-11-11, "Ares audit closure" session): the two
advisories with available patches (lru RUSTSEC-2026-0253, rmcp
RUSTSEC-2026-0462) are fixed immediately (Libraries #83); the eight
**unmaintained** advisories are deferred with tracked exceptions. The GLib
unsoundness was added separately in Libraries #87.

## Deferred maintenance advisories

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

These maintenance advisories track upstream maintenance status. The table
records how the crates enter the workspace graph and possible migration paths.

## GLib unsoundness exception

[RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html) affects
`glib::VariantStrIter` and is fixed in GLib 0.20 and later. The desktop Tauri
and GTK dependency chain resolves GLib 0.18.5. Reachability of the affected
iterator from the shipped desktop feature sets remains unverified. The
RecursiveIntell supply-chain owner should review the exception by May 2027,
or earlier if the desktop dependency graph can adopt the fixed range.

## Where the ignores live

- `fib-quant/deny.toml` → `[advisories] ignore` (bincode, paste — the two that
  fib-quant's own release receipt gate sees).
- The root `deny.toml` is used by the hardening workflow's locked
  default-feature advisory check. This check did not report the GLib
  advisory in local validation, although `cargo tree --locked -i glib@0.18.5`
  resolved the desktop dependency path.
- The broader all-features scan reports `RUSTSEC-2024-0449` on
  `unsafe-libyaml` through the optional `serde_yaml` dependency of
  `llm-output-parser`. It remains open and has no ignore entry.

## Exit criteria (when to revisit)

Re-evaluate on any of: tauri v2 major bump (drops gtk3/glib 0.18 → removes
proc-macro-error + unic chain), nalgebra/simba update (drops paste), a
bincode-2 migration of cea-core/fib-quant, or a new vulnerability advisory
against any deferred crate (immediate action then).
Review owner: RecursiveIntell supply-chain lane. Review date: 2027-05 (6mo).
