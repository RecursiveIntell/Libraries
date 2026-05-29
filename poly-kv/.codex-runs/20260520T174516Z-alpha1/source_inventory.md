# Source Inventory

Initial tracked implementation state:

- No `Cargo.toml` workspace manifest present.
- No `crates/` directory present.
- Documentation and acceptance scripts are present under `docs/`, `scripts/`, and `codex/`.
- Existing untracked `.codex-runs/_hook_receipts/` files were present before implementation and are treated as run-system artifacts.

Canonical source documents read before implementation:

- `AGENTS.md`
- `docs/SOURCE_OF_TRUTH_MAP.md`
- `docs/QUANT_CODEC_CORE_SPEC.md`
- `docs/POLY_KV_IMPLEMENTATION_SPEC.md`
- `docs/API_PROPOSAL.md`
- `docs/ACCEPTANCE_GATES.md`
- `docs/PUBLIC_CLAIM_BOUNDARY.md`
- `docs/BENCHMARK_PLAN.md`

Source-of-truth ownership:

- `quant-codec-core`: codec IDs, profile digests, dtype, KV shape/layout, codec traits, eval reports.
- `poly-kv`: shared pool manifests/readers/receipts, exact fallback, q8 key reference codec, value-codec boundary, memory accounting.
- TurboQuant/FibQuant math: not implemented locally.
- Governor/runtime/app integrations: out of scope.
