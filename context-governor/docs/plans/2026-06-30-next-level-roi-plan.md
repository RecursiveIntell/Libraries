# Context Governor Next-Level ROI Plan

Date: 2026-06-30

## Goal

Move `context-governor` from a useful receipt-backed compactor to a repeatably
certified context engine for Hermes and other agents.

## Build Order

1. One-command certification
   - Add `scripts/certify_all.py`.
   - Emit both JSON and Markdown receipts.
   - Cover Rust gates, Python tooling gates, CLI roundtrip, adversarial fixtures,
     task-success evaluation, and optional Hermes plugin tests.

2. Task-success evaluation
   - Keep anchor visibility/recoverability metrics.
   - Add deterministic answerability questions that score whether a compacted
     context still supports operational decisions.
   - Track incorrect-action risk separately from missing context.

3. Token accounting visibility
   - Keep `approx_chars` for compatibility.
   - Add an explicit provider-oriented approximate mode so receipts distinguish
     generic char/4 estimates from chat-provider budgeting heuristics.
   - Leave exact tokenizer integration as the next dependency-bearing step.

4. Hermes integration truth
   - Update docs from integration sketch to the current plugin behavior.
   - Document configured vs loaded engine, receipt store, exposed tools, and
     semantic-memory archival boundary.

5. Semantic-memory archival
   - Design the Hermes `MemorySink` adapter before implementation.
   - Archive only durable verified decisions/facts with receipt and item IDs.
   - Keep speculative/tool-noise content out of default archival.

## Acceptance Gates

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets`
- `python3 -m pytest tests_py -q`
- `python3 scripts/certify_all.py --quick --skip-hermes`

## Claim Boundary

This plan does not claim learned compression or provider KV-cache compression.
The new certification proves deterministic preservation, exact fallback,
operational answerability on fixtures, and runtime-adapter health when Hermes
tests are enabled.
