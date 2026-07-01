# ProveKV PPL replay tools

Active scaffolding for ProveKV/poly-kv PPL replay receipts.

Current status: validator/schema scaffolding is active. The archived full replay driver remains in `/home/sikmindz/kv-lossless-11x/proveKV/scripts/ppl_validate.py` until ported.

Validate an archived receipt:

```bash
python3 scripts/validate_provekv_ppl_state.py /home/sikmindz/kv-lossless-11x/results/bench/ppl/smollm2-1.7b/wikitext-2/state.json
```
