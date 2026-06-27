# Evaluation harness

`examples/replay_eval.rs` is a deterministic recoverability harness, not an LLM quality benchmark.

Run:

```bash
cargo run --example replay_eval
```

It compares:

- full transcript
- naive head/tail truncation
- `context-governor` compaction

Metrics:

- `visible`: required anchors directly present in the prompt text
- `recoverable`: required anchors findable through `context_search`
- `total_required`: expected anchor count
- `tokens`: approximate token budget consumed

Claim boundary:

- Passing this harness means critical strings stayed visible or recoverable.
- It does not prove model answer quality.
- It does not benchmark LLMLingua, Ogham, Squeez, provider-native cache behavior, or KV-cache compression.
