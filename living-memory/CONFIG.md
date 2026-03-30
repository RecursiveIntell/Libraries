# CONFIG.md
# ForgeConfig — Schema and Defaults

## Overview
`ForgeConfig` is the single source of truth for all runtime behavior.
It must be serializable as both JSON and TOML.
All defaults are safe and conservative.

---

## Full Schema with Defaults

```toml
# forge.toml (or pass ForgeConfig::default() in code)

mode = "standard"                          # "standard" | "sealed_local"
execution_backend_preference = "auto"      # "auto" | "host" | "container"
container_runtime_preference = "auto"      # "auto" | "docker" | "podman" | "nerdctl"
allow_test_modifications = false
sealed_allow_host_backend = false          # footgun; logs loud warning if true

forbidden_paths = [
  "tests/**",
  "**/*_test.rs",
  "**/fixtures/**",
  "**/*.snap",
  "Cargo.lock",
  ".github/**",
]

[caps]
max_files_changed = 8
max_total_lines_changed = 400
max_lines_changed_per_file = 200

[mindstate]
token_budget = 1800
evidence_budget = 8
max_steps = 8

[novelty]
delta_amp_default    = 0.7
delta_amp_stabilize1 = 0.2
delta_amp_stabilize2 = 0.1
delta_amp_clamp      = 0.0
orthogonality_target          = 0.10   # min cosine distance in strategy signature space
min_traces_for_orthogonality  = 2

[stabilization]
max_attempts = 4                          # fixed; attempts 1-4 always
stabilize1_force_family = "mechanical"   # or "pattern_refactor" if configured
stabilize2_force_minimal_diff = true
increase_stabilize_weight_factor = 2.5

[container]
rust_image = "rust:1.78-slim"            # configurable; pinned by default
command_timeout_secs = 120

[lab]
generation_batch_size = 32
eval_parallelism = 4
promotion_min_suite_pass_rate = 0.95
promotion_min_weighted_improvement = 0.05

[lab.archive]
novelty_bins = [
  { name = "low",  lo = 0.00, hi = 0.33 },
  { name = "med",  lo = 0.33, hi = 0.66 },
  { name = "high", lo = 0.66, hi = 1.00 },
]
stability_variance_threshold = 0.15
approach_families = ["mechanical", "pattern_refactor", "architectural", "perf", "safety"]
correctness_gate = 0.95

[cea]
enabled = true
enable_zero_shot = false                  # MUST be false by default; requires explicit opt-in
zero_shot_coverage_threshold = 0.80       # fraction of edit op sigs that must have graph edges
risk_confidence_threshold = 0.60          # confidence above which a (cause, effect) is a risk flag
max_line_distance_for_attribution = 50    # max lines between edit op and attributed effect
attribution_decay_factor = 10.0           # distance decay: 1/(1 + dist/factor)
causal_drift_warning_threshold = 0.25     # fractional edge weight change that triggers drift warning
min_runs_before_prediction = 5            # minimum eval runs before predictions are surfaced

[danger]
allow_semantic_memory_write = false       # must also be gated by Cargo feature "danger-sm-write"; compatibility-only
```

---

## Field notes

### `mode`
- `standard`: default; host backend allowed; no network restrictions
- `sealed_local`: container-only with `--network=none`; model router blocks remote

### `cea.enable_zero_shot`
Disabled by default. When enabled, `ForgeRuntime::run_attempts()` will skip live checks
for patches where `CausalPrediction.zero_shot_eligible == true` and use the predicted
score instead. This is an advanced feature — enable only after the CEA graph has accumulated
at least `cea.min_runs_before_prediction` runs.

### `danger.allow_semantic_memory_write`
Cannot be enabled at runtime if the crate was compiled without the `danger-sm-write` feature.
Even with the feature, this is off by default and only enables the compatibility
direct-import escape hatch. The canonical path remains Forge export envelope
generation followed by bridge transformation and memory import.

---

## Config loading order (precedence, highest first)
1. Environment variables: `FORGE_MODE`, `FORGE_CEA_ENABLED`, etc.
2. `forge.toml` in the working directory
3. `ForgeConfig::default()`

All env vars are optional. Unknown keys in `forge.toml` are ignored with a warning (not an error).
