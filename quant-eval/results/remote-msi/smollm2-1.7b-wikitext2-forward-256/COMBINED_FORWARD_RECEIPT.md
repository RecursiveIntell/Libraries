# Remote MSI Full Forward Compressed-Attention Gate — 2026-06-29

## Bottom line

The actual SmolLM2/LlamaAttention forward path was monkey-patched after RoPE and before `o_proj` to run compressed-score top-k attention, then logits/PPL were compared against the unmodified baseline forward pass.

This is the first full forward-pass receipt for the new compressed-cache method.

Result:

- 128 tokens, top_k=64: PASS.
- 256 tokens, top_k=64: FAIL by logit/KL drift, despite PPL delta staying under 3%.
- 256 tokens, top_k=128: FAIL strict logit-cosine p05, but PPL/KL mostly acceptable.
- 256 tokens, top_k=192: PASS.

Interpretation:

Uniform top_k=64 is too aggressive at 256 tokens for strict logit preservation. The method is real, but the next implementation needs an adaptive budget policy rather than one global k.

## Host / environment

- Host: `msi`
- GPU: NVIDIA GTX 1070 8GB
- Model: `HuggingFaceTB/SmolLM2-1.7B-Instruct`
- Corpus: WikiText-2 raw test split
- Device: CUDA
- Script: `poly-kv/scripts/compressed_attention_forward_ppl.py`

## Method

The script loads the real HuggingFace model and replaces each `LlamaAttention.forward` method.

Baseline:

- normal model forward
- normal eager attention
- compute logits and PPL window

Compressed variant:

- compute q/k/v projections
- apply RoPE
- rank candidate keys using quantized key scores (`quant_bits=4`)
- select top-k plus recent guard
- compute exact selected softmax over selected keys
- multiply selected values
- pass through original `o_proj`
- compute logits, PPL, KL, cosine, max logit drift, target-logit drift

Important boundary:

- Ranking uses quantized key scores without full-key decode.
- Selected keys and values are used for exact selected softmax/output.
- Python implementation is quality-only; speed is not meaningful here.

## Receipt paths

Local receipts:

- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/top64_receipt.json`
- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/top128_receipt.json`
- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/top192_receipt.json`
- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/top64_128tokens_receipt.json`

Remote receipts:

- `/home/jstevenson/Coding/Libraries/poly-kv/bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-gate-128-top64/receipt.json`
- `/home/jstevenson/Coding/Libraries/poly-kv/bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-gate-256-top64/receipt.json`
- `/home/jstevenson/Coding/Libraries/poly-kv/bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-gate-256-top128/receipt.json`
- `/home/jstevenson/Coding/Libraries/poly-kv/bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-gate-256-top192/receipt.json`

## Thresholds

Declared in every receipt:

- `abs(delta_ppl_pct) <= 3.0`
- `logit_cosine_p05 >= 0.995`
- `kl_p95 <= 0.05`
- `decoded_keys_for_ranking == 0`

## Results

### 128 tokens, top_k=64, recent_guard=16

Receipt: `top64_128tokens_receipt.json`

- Passed: `true`
- Baseline PPL: `5.100948380175517`
- Compressed-attention PPL: `5.0741467484871325`
- Delta PPL: `-0.5254244836616424%`
- Logit cosine mean: `0.9989634156227112`
- Logit cosine p05: `0.9980405509471894`
- KL mean: `0.0031069274991750717`
- KL p95: `0.012650910019874576`
- Argmax match rate: `0.9743589758872986`
- Attention samples: `98,304`
- Decoded keys for ranking: `0`
- Decoded selected keys mean: `77.27189127604167`
- Decoded values mean: `77.27189127604167`
- Attention output cosine mean: `0.9989424814076907`
- Attention output cosine p05: `0.9981527864933014`
- Attention top-k overlap mean: `0.8400201797485352`
- Attention top-k overlap p05: `0.609375`

### 256 tokens, top_k=64, recent_guard=16

Receipt: `top64_receipt.json`

- Passed: `false`
- Baseline PPL: `11.900871279801352`
- Compressed-attention PPL: `12.217243882373811`
- Delta PPL: `+2.6583986595117577%`
- Logit cosine mean: `0.98982834815979`
- Logit cosine p05: `0.957086169719696`
- KL mean: `0.08668430894613266`
- KL p95: `0.564118802547455`
- Argmax match rate: `0.8571428656578064`

Interpretation:

PPL delta alone would pass, but the logits are too different. This is a kill for uniform top_k=64 at 256 tokens under strict logit preservation.

### 256 tokens, top_k=128, recent_guard=16

Receipt: `top128_receipt.json`

- Passed: `false`
- Baseline PPL: `11.900871279801352`
- Compressed-attention PPL: `11.863077813047635`
- Delta PPL: `-0.3175689062183356%`
- Logit cosine mean: `0.9980449080467224`
- Logit cosine p05: `0.9913895130157471`
- KL mean: `0.006328055169433355`
- KL p95: `0.028843563795089726`
- Argmax match rate: `0.9740259647369385`

Interpretation:

Top_k=128 fixes PPL and KL but still misses strict low-tail logit cosine. This is close but not enough for the current gate.

### 256 tokens, top_k=192, recent_guard=16

Receipt: `top192_receipt.json`

- Passed: `true`
- Baseline PPL: `11.902627753089247`
- Compressed-attention PPL: `11.94929024188842`
- Delta PPL: `+0.39203518556700373%`
- Logit cosine mean: `0.9999055862426758`
- Logit cosine p05: `0.99968923330307`
- KL mean: `0.00025727308820933104`
- KL p95: `0.001334615563973785`
- Argmax match rate: `1.0`
- Target logit abs delta mean: `0.04114879295229912`
- Target logit abs delta p95: `0.125`

Interpretation:

This is the first full forward-pass positive result. It selects up to 192 + recent guard from 256, so it is not a large sparsity win yet. It proves the patched method can preserve model behavior when the budget is high enough.

## Honest claim status

Safe claim:

SmolLM2-1.7B on WikiText-2 can run a patched compressed-score top-k attention forward pass with quantized-key ranking and selected key/value decode. At 256 tokens, top_k=192 preserves logits/PPL under the declared gate. At 128 tokens, top_k=64 passes.

Unsafe claim:

Do not claim a useful 4x attention sparsity win yet. At 256 tokens, top_k=64 fails logit/KL drift and top_k=128 still fails the strict logit tail.

## Next engineering step

Implement adaptive per-layer/head/token-position budgets:

- early fragile layers/heads: top_k around 192 at 256 tokens
- stable mid/late heads: top_k around 64–128
- always keep recent guard/sink tokens
- learn the budget from per-layer/head drift receipts, not from hand tuning

Then rerun the same full-forward gate with:

- adaptive average selected values <= 128 at 256 tokens
- pass strict logit/PPL thresholds
- report decoded selected keys/values by layer/head

That is the next real win condition.
