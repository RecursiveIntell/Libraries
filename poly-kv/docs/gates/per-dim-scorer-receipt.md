# Per-dimension scorer gate receipt

## What shipped

- `compressed-scorer/src/per_dim_impl.rs`: Rust `PerDimScorer` with asymmetric
  per-dimension uniform quantization over unit-normalized keys.
  `no_std`/`alloc` compatible; tested on host, RISC-V, and ESP32-S3.
- `poly-kv/scripts/compressed_attention_forward_ppl.py`: added `--scorer per-dim`
  using the same unit-normalized asymmetric quantization model.

## Rust verification

```bash
cd /home/sikmindz/Coding/Libraries
cargo test -p compressed-scorer
cargo test -p compressed-scorer --no-default-features --features no_std
cargo check -p compressed-scorer --no-default-features --features no_std --target riscv32imc-unknown-none-elf
cargo +esp check -p compressed-scorer --no-default-features --features no_std --target xtensa-esp32s3-none-elf -Z build-std=core,alloc
```

Results:
- default tests: 21 passed
- `no_std` tests: 17 passed; 1 ignored
- RISC-V check: passed
- ESP32-S3 check: passed

## Python forward-pass gates

Run on MSI with `HuggingFaceTB/SmolLM2-1.7B-Instruct` and `wikitext-2`.

### 256 tokens

```bash
python3 -u scripts/compressed_attention_forward_ppl.py \
  --model HuggingFaceTB/SmolLM2-1.7B-Instruct --corpus wikitext-2 \
  --n-tokens 256 --ppl-frac 0.3 --scorer per-dim \
  --adaptive-budget --budget-target-cosine 0.98 \
  --budget-min-k 64 --budget-max-k 256 --budget-ref-k 256 \
  --recent-guard 16 --quant-bits 8 \
  --output bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-perdim-adaptive-256-unit-q8-t98-rk256/regression/receipt.json \
  --device cuda
```

- baseline PPL: 11.9009
- compressed PPL: 11.9127
- delta PPL: +0.10%
- logit cosine p05: 0.99999
- KL p95: 3.4e-05
- **passed: true**

Receipt: `/home/sikmindz/Coding/Libraries/poly-kv/bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-perdim-adaptive-256-unit-q8-t98-rk256/regression/receipt.json`

### 512 tokens

```bash
python3 -u scripts/compressed_attention_forward_ppl.py \
  --model HuggingFaceTB/SmolLM2-1.7B-Instruct --corpus wikitext-2 \
  --n-tokens 512 --ppl-frac 0.3 --scorer per-dim \
  --adaptive-budget --budget-target-cosine 0.98 \
  --budget-min-k 64 --budget-max-k 256 --budget-ref-k 256 \
  --recent-guard 16 --quant-bits 8 \
  --output bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-perdim-adaptive-512-unit-q8-t98-rk256/regression/receipt.json \
  --device cuda
```

- baseline PPL: 5.1442
- compressed PPL: 5.1183
- delta PPL: -0.50%
- logit cosine p05: 0.9970
- KL p95: 0.0112
- **passed: true**

Receipt: `/home/sikmindz/Coding/Libraries/poly-kv/bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-perdim-adaptive-512-unit-q8-t98-rk256/regression/receipt.json`

## Claim boundary

The per-dim scorer ranks keys using per-dimension quantized unit-normalized keys
without decoding full keys. Only the selected top-k keys and values are used for
the exact softmax and output projection. This gate measures forward-pass quality
drift only; it is not a speed or memory claim.

Tighter 4-bit budgets and lower target cosines drift at 512 tokens, so the
passing configuration uses 8-bit per-dim quantization with a generous adaptive
budget (target cosine 0.98, ref-k 256).
