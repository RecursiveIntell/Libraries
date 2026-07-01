# Remote MSI Real-LLM Compressed Cache Gate — 2026-06-29

## Host

- SSH host: `msi`
- User: `jstevenson`
- Address from SSH config: `192.168.50.69`
- GPU: NVIDIA GeForce GTX 1070, 8192 MiB
- Driver: 580.159.04
- Python: 3.14.5
- torch: 2.10.0+cu126
- transformers: 5.1.0
- datasets: 4.8.5
- Rust: rustc 1.93.1, cargo 1.93.1

## Commands run

```bash
ssh msi 'cd /home/jstevenson/Coding/Libraries/poly-kv; cargo build --release --example poly_kv_fast_roundtrip'
ssh msi 'cd /home/jstevenson/Coding/Libraries/poly-kv; python3 scripts/ppl_smoke.py --model HuggingFaceTB/SmolLM2-1.7B-Instruct'
ssh msi 'cd /home/jstevenson/Coding/Libraries/poly-kv; python3 -u scripts/ppl_validate.py --model HuggingFaceTB/SmolLM2-1.7B-Instruct --model-slug smollm2-1.7b --corpus wikitext-2 --n-tokens 256 --ppl-frac 0.3 --output bench/ppl/smollm2-1.7b/wikitext-2/compressed-topk-gate-256/state.json --phase1-timeout 1800'
ssh msi 'cd /home/jstevenson/Coding/Libraries/poly-kv; python3 -u scripts/real_cache_compressed_attention.py --cache bench/ppl/smollm2-1.7b/wikitext-2/compressed-topk-gate-256/cache_oracle.pt --output bench/ppl/smollm2-1.7b/wikitext-2/compressed-topk-gate-256/real_cache_compressed_attention_top64.json --top-k 64 --recent-guard 16 --layers 24 --queries-per-head 8 --quant-bits 4 --device cuda'
python3 scripts/validate_provekv_ppl_state.py quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-256/state.json
python3 tools/provekv_ppl/ppl_receipt.py quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-256/state.json --max-abs-delta-pct 0.1
```

## Local copied receipt paths

- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-256/state.json`
- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-256/report.md`
- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-256/real_cache_compressed_attention_top64.json`
- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-256/ppl_receipt_summary.json`

Remote artifact directory:

- `/home/jstevenson/Coding/Libraries/poly-kv/bench/ppl/smollm2-1.7b/wikitext-2/compressed-topk-gate-256/`

## Real-LLM roundtrip PPL result

Model/corpus:

- Model: `HuggingFaceTB/SmolLM2-1.7B-Instruct`
- Corpus: WikiText-2 raw test split
- Tokens: 256
- PPL eval window: tokens 178..255

Measured:

- Oracle PPL: `11.926321457678194`
- Roundtrip PPL: `11.926321457678194`
- Delta PPL: `0.0%`
- Compression ratio: `11.130434782608695x`
- Pool size bytes: `9,043,968`
- Total compressed bytes: `9,043,968`
- Roundtrip CLI time: `23.85692596435547s`
- Forward with overwritten cache: `0.028177499771118164s`
- Validation: `provekv ppl state: PASS model=HuggingFaceTB/SmolLM2-1.7B-Instruct corpus=wikitext-2 ratio=11.1304x`

## Real-cache compressed-attention quality gate

This uses real K/V tensors from the same SmolLM2/WikiText cache, then compares:

- Full attention: `softmax(q @ K^T) @ V`
- Compressed-topk proxy: quantized key scores -> top-k -> exact selected-value attention

Configuration:

- Layers evaluated: 24
- Heads: 32 KV heads per layer
- Samples: 6,144
- top_k: 64
- recent_guard: 16
- quant_bits: 4
- decoded_keys: 0

Measured:

- Passed aggregate thresholds: `true`
- Attention output cosine mean: `0.9996757385815727`
- Attention output cosine p05: `0.9999186247587204`
- Attention MSE mean: `1.8636054309552297e-05`
- Attention MSE p95: `4.488884314923757e-05`
- Top-k overlap mean: `0.9405695597330729`
- Top-k overlap p05: `0.875`
- Decoded values mean: `67.66259765625`
- Decoded values p95: `73.0`
- Raw fp16 key bytes: `25,165,824`
- Estimated compressed key bytes: `7,077,888`
- Estimated key compression ratio: `3.5555555555555554x`
- Failure count under per-sample thresholds: `59 / 6144`

Note on failure count:

The aggregate gate passed, but 59 individual layer/head/position samples tripped per-sample thresholds. These are mostly early-layer/head outliers. This is useful: it tells us the next policy should be layer/head adaptive rather than uniform top-k.

## Claim boundary

Safe claim from this run:

- SmolLM2-1.7B on WikiText-2 at 256 tokens preserves PPL exactly through the existing poly-kv roundtrip path at `11.13x` compression.
- A real-cache compressed-top-k attention proxy over the same K/V cache keeps high aggregate attention-output similarity while decoding zero keys and ~68 values per query/head on average.
- The method has real evidence beyond synthetic fixtures now.

Not safe yet:

- This is not full compressed-attention PPL replacement. The compressed-attention script uses cached key vectors as deterministic query probes because the PPL harness does not yet save post-RoPE query states.
- It does not yet patch the model forward pass to replace live attention and measure logit/PPL drift from compressed top-k attention.
- It does not prove throughput wins; top-k proxy latency was slower than full attention in this Python harness due Python/Torch overhead and no fused kernel.

## Next exact kill gate

Patch the model attention forward path or capture post-RoPE query states so the next receipt can report:

- baseline logits
- compressed-attention logits
- logit cosine/KL/max drift
- PPL delta under compressed top-k attention
- per-layer/head adaptive top-k policy

Uniform top-k=64 looks promising but not final. The outliers say the next method should keep larger budgets for early fragile heads and smaller budgets for later stable heads.
