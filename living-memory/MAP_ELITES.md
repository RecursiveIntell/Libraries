# MAP_ELITES.md
# MAP-Elites Archive

## Purpose
Maintain a population of high-quality candidates across *diverse* strategy styles.
The goal is not to converge to the single best algebra — it is to populate every cell
in the diversity space with the best candidate that fits that cell's niche.

Over time, CEA augments the archive with causal fingerprints, adding a structural
diversity dimension on top of behavioral diversity.

---

## Cell dimensions (v1)

### Dimension 1: novelty_bin
| Bin  | Range         |
|------|---------------|
| low  | [0.00, 0.33)  |
| med  | [0.33, 0.66)  |
| high | [0.66, 1.00]  |

Derived from `ScoreVector.novelty`.

### Dimension 2: stability_bin
| Bin      | Condition                       |
|----------|---------------------------------|
| stable   | strategy tag variance < 0.15    |
| variable | otherwise                       |

Single-run tasks use `stable` by default (variance undefined).

### Dimension 3: approach_family
Derived from `extract_strategy_tags(patch)`:

| Family           | Trigger tags                                          |
|------------------|-------------------------------------------------------|
| mechanical       | `replace_heavy`, `fn_level_edit` only, `single_file`  |
| pattern_refactor | `extract_function`, `introduce_trait`, `module_split` |
| architectural    | `new_file`, `trait_level_edit`, `multi_file`          |
| perf             | tags contain "perf" or "async_boundary" + multi-file  |
| safety           | tags contain "error_type_edit" or "macro_level_edit"  |

Tie-breaking: pick the first matching family in the order above.
No match → family = `mechanical` (default).

### Correctness gate
Only candidates with `correctness >= 0.95` are eligible for archive insertion.
Candidates below this gate are discarded (not stored in archive, still stored in eval_runs).

### Cell key
```
cell_key = "{novelty_bin}:{stability_bin}:{approach_family}"
```
Example: `"high:stable:pattern_refactor"`

Maximum cells: 3 × 2 × 5 = 30

---

## Archive update rule

```
archive_insert(candidate, score_vector, patch, cea_prediction):
  cell_key = compute_cell_key(score_vector, patch)
  
  if score_vector.correctness < correctness_gate:
    return ArchiveUpdate::BelowGate
  
  score_summary = compute_score_summary(score_vector)
  
  existing = archive_cells.get(cell_key)
  
  if existing is None or score_summary > existing.score_summary:
    archive_cells.upsert(cell_key, {
      candidate_id:       candidate.id,
      score_summary_json: score_summary,
      cea_fingerprint:    cea_prediction?.causal_fingerprint,
      updated_at:         now(),
    })
    return ArchiveUpdate::Inserted | ArchiveUpdate::Replaced
  
  return ArchiveUpdate::NoChange
```

### score_summary computation
```
score_summary = 0.70 * correctness + 0.20 * novelty + 0.10 * stability
```

---

## Causal diversity (CEA augmentation)

When CEA is enabled and an advisory prediction has adequate exact coverage:
- an archive cell may store `cea_fingerprint` as a digest of sorted dominant observational
  `EditOpSignature` hashes; and
- emitter selection may use fingerprint distance as a diversity heuristic.

This is association diversity, not proof of causal diversity, and it does not affect the
mandatory check-execution gate.

---

## Candidate generation — Emitters

### E1: Param mutation
Randomly perturb numeric parameters in `AlgebraSpec`:
- Sample perturbation scale from `[0.05, 0.30]` uniformly.
- Clamp to valid parameter bounds after mutation.
- Target parameters: `k`, operator weights, `delta_amp_*`, `evidence_budget`, `orthogonality_target`.
- Produce one mutant per parent.

### E2: Crossover
Select two parents from different archive cells:
- Take `basis_constructor` and `mindstate_token_budget` from parent A.
- Take `delta_policy` (all `delta_amp_*` values) from parent B.
- Average all other numeric parameters (midpoint crossover).
- Prefer crossing parents from cells with different `approach_family`.
- If CEA enabled: prefer crossing parents with different `cea_fingerprint`.

### E3: LLM mutator (optional; lab mode only)
- Prompt a model to produce a modified `AlgebraSpec` as JSON.
- System prompt must include: "output only valid AlgebraSpec JSON; no other text."
- Output must pass `invariants::validate_algebra_spec(spec)` before evaluation.
- On invalid output: discard and log; do not retry more than 2 times.

---

## Generation batch loop

```
ForgeLab::run_generation(batch_size, task_suite, config):
  parents = sample_parents_from_archive(n = batch_size / 2)
  
  candidates = []
  for i in 0..batch_size:
    emitter = choose_emitter(config, i)  # round-robin E1/E2/E3
    spec = emitter.emit(parents)
    candidates.push(Candidate::new(spec, parents=[...]))
  
  results = parallel_evaluate(candidates, task_suite, config.lab.eval_parallelism)
  
  for (candidate, result) in results:
    archive_insert(candidate, result.score_vector, result.patch, result.cea_prediction)
  
  return GenerationReport { ... }
```
