# Medusa learning-coding-agent corpus v1

This is a small, deterministic, operator-authored local fixture corpus. It contains five Rust task families with one development, calibration, and holdout fixture per family. The holdout oracle is kept in `oracles.json` and is not copied into the runner-facing task manifest.

## Immutability

v1 is immutable after treatment observes it. If fixtures or policy must change, supersede this corpus with v2; do not rewrite v1.

## Verification

Run `python3 AiDENs/scripts/validate_learning_corpus.py AiDENs/fixtures/learning-coding-agent/v1` and `pytest -q AiDENs/scripts/tests/test_learning_corpus.py`. The validator checks pinned toolchain/verifier identity, fixture digests, split isolation, negative-category coverage, and holdout-oracle separation.

Claims are limited to these local fixtures; this corpus is not evidence of real-world agent capability, security, or benchmark performance.
