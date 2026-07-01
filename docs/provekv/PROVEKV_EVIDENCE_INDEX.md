# ProveKV Evidence Index

This index records prior ProveKV/poly-kv evidence without upgrading it into a new V2 runtime claim.

## Archived receipt roots

- `/home/sikmindz/kv-lossless-11x/results/bench/ppl/smollm2-1.7b/wikitext-2/state.json`
- `/home/sikmindz/kv-lossless-11x/results/bench/ppl/smollm2-1.7b/wikitext-2/report.md`
- `/home/sikmindz/kv-lossless-11x/results/bench/multi_agent/qwen2.5-0.5b/scaling_summary.json`
- `/home/sikmindz/kv-lossless-11x/results/bench/multi_agent/hot_tier_summary.json`

## PPL receipt: SmolLM2-1.7B / WikiText-2

- Oracle PPL: `4.7607620871`
- Roundtrip PPL: `4.7607620871`
- Delta PPL: `+0.00%`
- Compression ratio: `11.1304x`
- Pool size: `36,175,872` bytes

Claim boundary: this proves a specific archived roundtrip replay, not universal losslessness and not a finished V2 compressed-attention runtime.

## Multi-agent shared-prefix receipt: Qwen2.5-0.5B

- N=2: `1.8x` memory reduction
- N=3: `2.69x`
- N=4: `3.59x`
- N=6: `5.39x`
- N=8: `7.19x`

Claim boundary: this supports shared immutable pool amortization under the archived methodology. It does not prove multi-tenant side-channel safety.

## Hot-tier warning

The archived hot-tier quality was PPL-invariant across b=2/4/8 on the tested fixtures, but long shell tails were memory-negative because JSON block envelopes dominated payload bytes. V2 must use compact shell payloads before claiming shell memory wins.

## GTX 1070 lesson

The old GPU kernel passed parity but only won ~1.5-2.7% because per-call transfer overhead dominated. V2 GPU work must operate at page/batch granularity with resident compressed pages.
