# PPL Validation Report — HuggingFaceTB/SmolLM2-1.7B-Instruct on wikitext-2

- **Generated:** 2026-06-29T07:11:58.481800-05:00
- **Model:** `HuggingFaceTB/SmolLM2-1.7B-Instruct`
- **Corpus:** `wikitext-2` (n_tokens=256, ppl_frac=0.3)
- **Seed:** 42

## Headline

- **Oracle PPL:** 11.9263
- **Roundtrip PPL:** 11.9263
- **Δ PPL:** +0.00%
- **Compression ratio:** 11.13x
- **Pool size:** 9,043,968 bytes
- **Total compressed:** 9,043,968 bytes

## Methodology

1. **Phase 0 (oracle):** use_cache=True forward pass over first n_tokens of WikiText-2 test split. fp16, cuda. PPL computed over last 30% of tokens (HF causal-LM recipe: shift, logsumexp, gather, exp(mean)).
2. **Phase 1 (compressed roundtrip):**
   - Extract per-token K/V vectors from the DynamicCache
   - Build poly-kv corpus JSON
   - Invoke `poly_kv_fast_roundtrip` CLI (composite build + parallel decompress) (23.9s)
   - Read decompressed layers from the roundtrip.bin output
   - Pre-populate a fresh `DynamicCache` and forward with it
   - PPL over the same window as Phase 0
3. **Phase 2 (report):** per-layer byte accounting; this file.

## Per-layer accounting

| Layer | Oracle bytes (fp16 KV) | Roundtrip layer bytes (JSON+len) |
|------:|-----------------------:|---------------------------------:|
| 0 | 2,097,152 | 12,286,824 |
| 1 | 2,097,152 | 11,720,218 |
| 2 | 2,097,152 | 11,624,816 |
| 3 | 2,097,152 | 11,589,566 |
| 4 | 2,097,152 | 11,574,005 |
| 5 | 2,097,152 | 11,562,665 |
| 6 | 2,097,152 | 11,549,866 |
| 7 | 2,097,152 | 11,524,393 |
| 8 | 2,097,152 | 11,499,024 |
| 9 | 2,097,152 | 11,518,100 |
| 10 | 2,097,152 | 11,483,295 |
| 11 | 2,097,152 | 11,479,872 |
| 12 | 2,097,152 | 11,456,843 |
| 13 | 2,097,152 | 11,477,416 |
| 14 | 2,097,152 | 11,476,934 |
| 15 | 2,097,152 | 11,429,694 |
| 16 | 2,097,152 | 11,427,701 |
| 17 | 2,097,152 | 11,394,120 |
| 18 | 2,097,152 | 11,367,052 |
| 19 | 2,097,152 | 11,326,268 |
| 20 | 2,097,152 | 11,307,514 |
| 21 | 2,097,152 | 11,283,092 |
| 22 | 2,097,152 | 11,271,402 |
| 23 | 2,097,152 | 11,238,895 |

## Receipts

- `state.json` — full machine-readable state
- `cache_oracle.pt` — Phase 0 DynamicCache (fp16 K/V tensors)
- `poly_kv_corpus.json` — Phase 1 input to the poly-kv CLI
- `roundtrip.bin` — Phase 1 binary output (manifest + 24 layer blobs)
- `manifest` (in `roundtrip.bin`) — pool manifest from poly-kv

## Caveats

- The fib-quant decoder is single-block per call; for n_tokens=1024 × 24 layers the decode work is ~24M codeword lookups, which serial-decoded in Rust takes >30 min. For this initial validation, n_tokens can be reduced to keep roundtrip time under 5 min; for a public release the codec needs a vectorized batched decode implementation.
- transformers 5.1.0, torch 2.10.0+cu126, device cuda
- Model config: num_layers=24 num_heads=32 num_kv_heads=32 head_dim=64 hidden_size=2048
- Phase 0 forward: 0.4s
- Phase 1 roundtrip CLI: 23.9s
- Phase 1 forward with pre-populated cache: 0.0s
