# CEA Engine Local Replay Evaluation — 2026-07-13

## Result

The deterministic offline replay harness wrote
`target/cea-eval/receipt.json` with schema `cea_replay_eval_receipt_v1` at
`2026-07-13T22:48:56.669Z`. The measured harness runtime was **2641 ms**.

It executed seven labeled prediction cases plus one tiny real Cargo fixture:

- Exact coverage: 5 cases, mean **1.000**.
- Explicitly enabled structural-fuzzy coverage: 1 case, mean **0.250**.
- Unknown coverage: 1 case, mean **0.000**.
- Labeled-prediction Brier score: **0.09151785714285714**.
- Risk precision/recall: **0.000 / 0.000** (`TP=0`, `FP=0`, `FN=3`). This is
  measured negative evidence: the present risk-flag policy missed all three
  labeled risk cases in this small fixture set.
- Ablation localization: **1.000** accuracy over **2** interventions; only
  operation index `0` was supported.
- False-negative count: **3**.

The requested `full_check_oracle` and `naive_proximity` comparison records are
present but explicitly **unavailable** (`null` metrics). The prediction fixtures
use synthetic labeled graphs, not executable checker workloads with source
locations. Deriving either baseline from those labels would leak ground truth
and fabricate perfect or misleading comparison scores. The real Cargo lane is
used only for the bounded two-operation ablation result.

Non-empty calibration buckets were: `[0.0, 0.1]` (`n=1`, mean prediction
`0.0`, empirical rate `0.0`), `[0.5, 0.6]` (`n=2`, `0.5`, `0.0`), `[0.6,
0.7]` (`n=1`, `0.625`, `1.0`), and `[0.9, 1.0]` (`n=3`, `1.0`, `1.0`).

## Commands

```text
cargo test -p forge-engine --example cea_replay_eval
cargo run -p forge-engine --example cea_replay_eval -- --output target/cea-eval/receipt.json
```

Both commands passed; the example suite reported **5 passed**. Task-owned Rust
files were formatted directly with `rustfmt --edition 2021`. A workspace-wide
`cargo fmt --check` remains a separate repository gate because unrelated dirty
files existed before this CEA pass.

## Claim boundary and limitations

These fixtures validate deterministic local engine mechanics only. They do
not establish external superiority, production readiness, or safe zero-shot
validation. The fixture corpus is tiny and hand-labeled; its Cargo lane is one
offline two-operation example, and no held-out external corpus, real-world
workload distribution, or production environment was evaluated. Fuzzy
matching was enabled only for its dedicated evaluation case. The prediction
gate was exercised as fail-closed (`RunChecks`), rather than as permission to
skip verification.

The zero risk recall is the most important result: prediction must remain
advisory, and check skipping must remain unavailable, until a larger held-out
corpus demonstrates calibrated risk behavior under the same receipt and
comparability rules.
