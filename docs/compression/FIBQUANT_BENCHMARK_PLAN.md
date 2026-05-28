# FibQuant Benchmark Plan

Created: 2026-05-16

No benchmark claims are made by this implementation pass.

Required future benchmark lanes before any performance claim:

- canonical spherical-Beta MSE by `(d, k, N)` against scalar baselines;
- per-vector reconstruction MSE and cosine on local KV-cache captures;
- attention-output cosine on a fixed local model/prompt set;
- end-to-end perplexity or task accuracy only with reproducible scripts, seeds, model revision, and hardware notes;
- storage accounting that includes norm headers and fixed-rate wire padding.

Every benchmark result must distinguish paper-reported numbers from local measurements.
