# RSI remaining-work resume — verified Rust gate repairs, program incomplete

## Verdict

The selected Rust substrate gate failures are closed in the isolated candidate: **344 passed, 0 failed, 1 ignored across 59 test suites**. Seven-package Clippy with `-D warnings`, changed-source formatting and `git diff --check` passed. This is not bounded-RSI campaign readiness, a recursive cycle, an adoption approval or a live-runtime verification.

**RSI and unattended/live adoption remain OFF.** No commit, push, install, shared config change, credential change, live memory import, candidate search or deployment was performed.

## Source and scope

- Libraries base: `d7cf668b35f18fbc3dd8ce9e2627521e63a82583`, detached and intentionally dirty at `/home/sikmindz/.ares/runs/rsi-tranche-20261112T020205-0600/Libraries`.
- Final candidate: 28 changed/new source files retained in `resume-source/`; exact identities are in `RESUME_SNAPSHOT.json`.
- 13 source files differ from the beginning of this resume, including scoped formatter changes. Behavioral repair surfaces are the policy dependency manifest, experiment invariant handling, action input binding, loop binding callsite and oracle regression test.
- Python measured: `3.14.8 (main, Oct  2 2026, 10:23:31) [GCC 16.2.1 20260810]`. No older-interpreter fallback or runtime-floor change.
- Current runtime observed: `4d3334ffb196a85ad4b480bf01d3792ed6da02f8`, clean at observation. This differs from frozen-contract runtime `13b7fc3e17a8a0da7f25455e52dfb4c36bd008c3`. Its identity was recorded, not installed/modified by this work. No current-runtime integration qualification is claimed.
- Controller tests use an offline, read-only-source bwrap namespace. This is not certification of the product's untrusted-candidate backend. Docker access was not broadened.

## Repairs executed in this resume

1. Moved existing `serde_json` from dev-only to normal dependencies: `ExecutionPermit` now uses it in the library. The resumed aggregate build reproduced this compile blocker before repair.
2. Replaced two production `expect()` calls in the repeat loop with typed `ExperimentIncomplete` errors carrying retained trials. Clippy's failure is preserved; no lint suppression was added.
3. Independently tested the Explorer's observation-input concern. The negative test reached action execution with substituted compiled input and failed with `MissingCompiledContext` before the repair. The existing CheckPlan execution input now binds the serialized observation as well as plan/configuration at issuance and consumption. The unchanged-input canonical V3 roundtrip still passes. This is input binding, not a claim that an observation equals one target, and not a new evidence schema or policy owner.
4. Formatted only already changed/new Rust files with `skip_children=true`, then reran both final gates against the same source identities.
5. Updated the run-owned gate helper to admit explicit retry linkage and reject reused output labels. Earlier receipts are retained with their actual metadata; no historical retry fields were rewritten.

## Final gates and evidence

| Gate | Result | Evidence |
|---|---|---|
| Exact seven-package regression | 344 passed / 0 failed / 1 ignored | `substrate-regression-final-02.result.json`; full raw output below and `resume-logs/` |
| Seven-package Clippy `-D warnings` | PASS, source stable | `substrate-clippy-final-01.result.json` |
| Scoped Rust formatter | PASS, 26 files in existing changed scope | `RESUME_FINAL_FORMAT.json` |
| Patch whitespace | PASS | receipt validation gate `resume-diff-check` |
| Selected receipt integrity | 13 retained receipts verified, including failed attempts | `RESUME_GATES.json`, per-attempt verification output |
| Final source/gate identity | PASS | test and lint before/after hashes equal `RESUME_SNAPSHOT.json` |
| Continuation rollback applicability | reverse `--check` PASS; no reverse mutation performed | `remaining-continuation.patch`, `resume-rollback-check.txt` |
| Cross-family final implementation verifier | NOT EXECUTED | Explorer is same-family advisory; final-source certification not claimed |
| Campaign / independent evaluator / recursive adoption | NOT QUALIFIED | remaining-work matrix below |

The ignored test is `wave1_tests::select_backend_falls_back_to_host_when_allowed`, explicitly marked environment-dependent. Cargo continues to warn that a non-root package profile is ignored; that warning was not hidden or treated as a Rust lint failure. The three selected failed receipts are the dependency compile blocker, production-expect Clippy blocker, and intentional observation-substitution RED witness; all retain failure status and verified integrity. Historical earlier-run failures outside the selected receipt set remain preserved and are not reclassified by this report.

## Specialist review

`resume-review/` used the actual read-only Explorer profile runner and separate receipt verification; run-owned session archival and artifact verification are runner-enforced. The worker had file tools only and explicitly did not execute builds/tests or calculate hashes. Its findings are advisory. The controller reproduced one concern, repaired it and reran gates. The unresolved exact-bounded oracle-slice question is not declared fixed. See `CONTROLLER_REVIEW.md`; source changed after review, so review integrity is not final-source semantic certification.

## Full remaining-work denominator

This matrix covers all 18 required tracking entries, including optional R15/R16 distinctions and program closeout. Passing the substrate regression does not complete these packages.

| ID | State | Remaining acceptance |
|---|---|---|
| R00 | partial | Bind actual mission/root task and approved campaign ceiling; reconcile live runtime drift before runtime integration. The private contract is not that authority. |
| R01 | partial | Complete ingress/source-replacement/TOCTOU conformance beyond the tested regular-file snapshot and policy controls; do not infer hostile sandbox admission. |
| R02 | partial | No qualified performance statistics, independently randomized task-level trials or environment/cache intervention controls. Physical correctness repeats are not statistical significance. |
| R03 | partial | Complete operator-owned immutable approval/source/expiry admission across every caller. Review exact-bounded oracle slice selection before stronger exact-target claims. |
| R04 | partial | Complete raw-artifact custody/staleness/tamper conformance and independent acceptance integration; no promotion readiness inferred from zero/unavailable scores. |
| R05 | partial | Product sealed-backend containment, hostile setsid/escape/resource/output/disk witnesses, campaign-wide deadline/stop/readback remain unqualified. Controller bwrap is not a product backend. Docker permission remains denied; no host fallback or broad grant authorized. |
| R06 | missing | Implement independent evaluator custody, frozen protocol/data, black-box candidate isolation, negative controls and durable holdout access accounting. |
| R07 | partial | Operator ratification of exact target/semantics/editable surface; property/confirmation cases and traced actual producer consumption. |
| R08 | missing | Version and integrate canonical durable send/token/cost reservations, settle-once/unknown-effect handling and restart/concurrency/stale-holder tests. Existing per-send bounds do not implement campaign totals. |
| R09 | missing | Qualify exact write-capable candidate producer after R05/R08; pin actual route/effort/output/timeout and cumulative resource charges. No shared route changes or coding canary performed. |
| R10 | missing | Implement thin owner-bound campaign adapter, admitted Python/bridge inputs, actual positive/negative campaign roundtrips and restart-safe readback. |
| R11 | blocked | Requires implemented R09/R10 plus explicit target/evaluator/resource admission; missing implementation is a blocker independent of operator approval. |
| R12 | blocked | Requires R11, exact-patch disposable adoption approval, active identity/backout and genuine successor consumption. Live deployment forbidden. |
| R13 | blocked | Implement prerequisite evaluator/cycle; separately authorize a preregistered matched-budget held-out study, or record inconclusive/no gain. |
| R14 | missing | Final cross-family frozen-implementation verification, crash/stop/fault/soak and operator-selected restricted unattended level. No automatic adoption. |
| R15 | partial | Canonical Python campaign adapter and strictly admitted bridge integration; independent evaluator and broader runtime adoption are not supplied by ProjectAdapter. |
| R16 | deferred | Keep deferred until data/lineage/dependency gates and evidence of value; do not add it as a new readiness prerequisite. |
| closeout | partial | This gate-repair tranche is verified; whole-program closeout remains open until mandatory task acceptance gates are met or explicitly dispositioned. |

## Next real integration boundary

Finish product execution-boundary/deadline/resource qualification and independent evaluator custody, then implement durable cumulative accounting under the existing native/Ares owners. Target ratification and a total operator-approved resource ceiling are additional campaign-admission requirements, not substitutes for the missing implementation. No amount of green component tests authorizes self-modification.

No revised hours estimate was computed from this resume. The prior estimate is not upgraded into a delivery promise.

## Rollback and artifact handling

`resume-candidate.patch` is the full tracked candidate delta from HEAD; it includes inherited work and must NOT be blindly reversed. New-file bytes are retained in `resume-source/`.

`remaining-continuation.patch` is derived against the remaining run's inherited-source snapshot (or the frozen HEAD for previously unchanged files). Its inverse was checked without application. Review it before any rollback; reversing it would return the continuation to the retained inherited state, not necessarily to the start of this resume. Never reset the canonical dirty checkout, installed runtime, live stores or user work. No database migration was executed.

Review session-state directories may contain runtime-owned credential copies; they are deliberately excluded from the downloadable packet. The packet contains allowlisted reports, source snapshots, execution receipts/blobs and logs, not HOME/config/auth/session-state trees or build outputs. Receipts retain absolute original artifact references; the package also has a relative checksum manifest. Artifact hashes prove integrity, not authority or correctness.

## Exact auditor reruns

From the retained workspace (use fresh labels; the helper refuses overwrite):

```bash
python3 /home/sikmindz/.ares/runs/rsi-remaining-20261112/run_gate.py auditor-substrate-01 cargo test --locked --offline -p forge-engine -p forge-pilot -p verification-policy -p check-runner -p sandbox-workspace -p typed-patch -p forge-policy --lib --tests -- --test-threads=1
python3 /home/sikmindz/.ares/runs/rsi-remaining-20261112/run_gate.py auditor-clippy-01 cargo clippy --locked --offline -p forge-engine -p forge-pilot -p verification-policy -p check-runner -p sandbox-workspace -p typed-patch -p forge-policy --lib --tests -- -D warnings
python3 /home/sikmindz/.ares/runs/rsi-remaining-20261112/collect_resume.py
```

`collect_resume.py` verifies the retained selected receipts and binds the final snapshot; it does not rerun tests. Later source changes invalidate the snapshot binding rather than silently refreshing it.

## Changed bytes since this resume began

- `Primitives/check-runner/src/lib.rs`
- `Primitives/check-runner/tests/cancellation_tests.rs`
- `forge-pilot/src/act.rs`
- `forge-pilot/src/loop_runner.rs`
- `forge-pilot/src/loop_runner_helpers.rs`
- `forge-pilot/tests/evidence_honesty_tests.rs`
- `forge-pilot/tests/oracle_plan_tests.rs`
- `living-memory/living-memory/src/experiment.rs`
- `living-memory/living-memory/src/lab/evidence.rs`
- `verification-policy/Cargo.toml`
- `verification-policy/src/lib.rs`
- `verification-policy/src/permit.rs`
- `verification-policy/tests/execution_binding_tests.rs`

## Verbatim final regression stdout

```text

running 12 tests
test tests::check_result_requires_all_checks_to_pass ... ok
test tests::collect_logs_preserves_outputs_and_timings ... ok
test tests::env_allowlist_accepts_safe_prefixes_and_rejects_unknowns ... ok
test tests::host_backend_clears_disallowed_environment_variables ... ok
test tests::host_backend_run_command_times_out ... ok
test tests::host_backend_timeout_kills_spawned_child_process_group ... ok
test tests::select_backend_rejects_host_in_sealed_mode_without_opt_in ... ok
test tests::select_backend_returns_host_when_allowed ... ok
test wave1_tests::check_result_all_pass_requires_every_check ... ok
test wave1_tests::host_backend_collect_logs_preserves_outputs_and_timings ... ok
test wave1_tests::select_backend_falls_back_to_host_when_allowed ... ignored, environment-dependent: fails when docker/podman is available because auto→container succeeds before host fallback
test wave1_tests::select_backend_rejects_unapproved_host_fallback_in_sealed_mode ... ok

test result: ok. 11 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 2.16s


running 1 test
test cancellation_stops_owned_host_descendant ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 2 tests
test forge_engine_docs_state_the_authority_boundary ... ok
test forge_engine_manifest_pins_local_authority_boundary ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 14 tests
test attribute_effects_uses_mapped_positions ... ok
test i1_cea_instrumentation_extracts_effect_signatures ... ok
test i2_cea_edit_op_signature_no_raw_source ... ok
test i3_cea_graph_update_is_idempotent ... ok
test i3b_cea_graph_update_is_atomic_when_run_log_insert_fails ... ok
test i3c_cea_graph_update_is_atomic_when_run_log_insert_fails_with_store_conn ... ok
test i3d_cea_graph_update_is_atomic_with_store_transaction_connection ... ok
test i3e_cea_graph_load_graph_works_with_store_transaction_connection ... ok
test i3f_cea_graph_restart_preserves_edge_stats_and_predictions ... ok
test i3g_cea_update_graph_persists_negative_evidence_for_absent_effects ... ok
test i3h_cea_restart_preserves_negative_evidence_predictions ... ok
test i3i_cea_legacy_edge_rows_are_migrated_forward_with_stats ... ok
test i4_cea_prediction_returns_neutral_for_unknown_sigs ... ok
test i5_cea_zero_shot_requires_explicit_enable ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s


running 12 tests
test backward_compat_old_json_without_new_fields ... ok
test backward_compat_old_status_names ... ok
test canonical_bundle_roundtrip_preserves_authoritative_contract ... ok
test claim_strength_display ... ok
test confidence_bounds_on_hypothesis ... ok
test episode_conversion_from_evidence_bundle ... ok
test episode_meta_and_content_capture_verification_artifacts ... ok
test forge_limits_defaults ... ok
test local_experiment_runner_construction ... ok
test sealed_bundle_immutability ... ok
test serialization_round_trip_evidence_bundle ... ok
test verification_type_variants ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 8 tests
test copy_dir_recursive_skips_symlinks ... ok
test env_allowlist_blocks_secrets_passes_cargo ... ok
test f1_host_backend_runs_command_captures_output ... ok
test f2_host_backend_timeout ... ok
test f4_container_backend_autodetect_fallback_host ... ok
test f5_container_backend_sealed_requires_no_network ... ok
test g1_cargo_adapter_detects_cargo_project ... ok
test g3_cargo_adapter_parses_clippy_json ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s


running 31 tests
test baseline_descriptor_fingerprint_deterministic ... ok
test build_hypothesis_edges_from_diff ... ok
test comparability_class_filtering ... ok
test compute_confidence_basic ... ok
test derive_status_from_counts ... ok
test evidence_assessment_categories ... ok
test experiment_config_defaults ... ok
test experiment_diff_from_all_pass_both ... ok
test experiment_diff_from_baseline_fail_patched_pass ... ok
test experiment_diff_from_baseline_pass_patched_fail ... ok
test experiment_diff_single_pair_not_statistically_meaningful ... ok
test experiment_diff_stable_failures_are_neutral ... ok
test experiment_diff_with_detailed_effects ... ok
test local_experiment_runner_does_not_fabricate ... ok
test multiple_effects_one_primary_claim ... ok
test objective_policy_bug_fix ... ok
test objective_policy_performance_no_scalarization ... ok
test objective_policy_stability_excluded_insufficient_trials ... ok
test pair_comparability_timeout_asymmetry ... ok
test pair_comparability_valid ... ok
test pair_comparability_workload_mismatch ... ok
test patch_execution_plan_cargo_default ... ok
test receipt_content_hash_mismatch_detected ... ok
test receipt_content_hash_verification ... ok
test update_hypotheses_from_diff_new_effect ... ok
test update_hypotheses_stable_failure_is_neutral ... ok
test verification_plan_basic_generation ... ok
test verification_plan_has_budget ... ok
test verification_plan_respects_step_limit ... ok
test verification_plan_with_ablation ... ok
test workspace_policy_defaults ... ok

test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 27 tests
test already_exported_check ... ok
test caller_managed_and_helper_produce_identical_payloads ... ok
test claim_and_relation_records_carry_stable_version_ids_and_bundle_valid_time ... ok
test claim_metadata_carries_projected_verification_summary ... ok
test claim_record_carries_supersedes_claim_version_id_when_present ... ok
test claim_record_does_not_synthesize_claim_lineage ... ok
test claim_semantics_intentionally_group_without_constraint_seed_kind ... ok
test episode_export_builds_canonical_forge_envelope ... ok
test episode_export_builds_canonical_forge_v2_envelope ... ok
test episode_export_from_bundle_populates_all_fields ... ok
test evidence_refs_bind_to_exported_claim_version ... ok
test export_bundle_idempotent_skip ... ok
test export_bundle_projects_store_promotions_into_claim_metadata ... ok
test export_bundle_returns_canonical_v3_envelope ... ok
test export_includes_experiment_diff_relations ... ok
test export_includes_receipt_evidence_refs ... ok
test export_includes_verification_trials_and_refutation_artifacts ... ok
test export_key_is_deterministic ... ok
test export_key_varies_with_inputs ... ok
test export_receipt_idempotent ... ok
test export_receipt_with_write_through_status ... ok
test export_with_hypothesis_edges_includes_relations ... ok
test export_without_attempt_or_run_keeps_single_evidence_ref ... ok
test relation_records_carry_supersedes_relation_version_id_when_hinted ... ok
test relation_records_do_not_synthesize_relation_lineage_when_unknown ... ok
test rendering_version_is_three ... ok
test to_export_envelope_v3_survives_bridge_transform_and_memory_import ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s


running 6 tests
test h1_evaluate_one_candidate_one_task_mocked_patch ... ok
test h2_archive_insert_replaces_lower_score ... ok
test h3_archive_insert_below_gate_discarded ... ok
test h4_archive_preserves_higher_score ... ok
test h5_raw_scoring_model ... ok
test h6_anti_fluff_guard ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s


running 18 tests
test evidence_bundle_crud ... ok
test evidence_bundle_insert_or_replace ... ok
test evidence_bundle_not_found ... ok
test experiment_run_insert ... ok
test export_receipt_insert_and_check ... ok
test failure_class_from_error ... ok
test failure_class_retriability ... ok
test failure_class_serde_round_trip ... ok
test fresh_db_has_v2_and_v3_tables ... ok
test legacy_split_only_evidence_row_rebuilds_canonical_bundle ... ok
test migration_v2_idempotent_reopen ... ok
test run_failure_empty_for_unknown_run ... ok
test run_failure_insert_and_query ... ok
test schema_hash_is_stable ... ok
test schema_hash_matches_forge_schema_hash ... ok
test user_version_is_v4_after_open ... ok
test v1_db_upgraded_to_latest ... ok
test verification_plan_insert ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s


running 22 tests
test c1_patch_rejects_forbidden_path_tests ... ok
test c2_patch_rejects_forbidden_path_snap ... ok
test c3_patch_rejects_cap_files ... ok
test c4_patch_rejects_cap_total_lines ... ok
test c5_patch_accepts_well_formed ... ok
test c6_patch_validation_returns_all_violations ... ok
test c7_patch_rejects_path_traversal ... ok
test d1_apply_insert_after_line ... ok
test d2_apply_replace_range ... ok
test d3_apply_delete_range ... ok
test d4_apply_match_anchor ... ok
test d5_apply_fails_on_ambiguous_context ... ok
test d6_apply_is_atomic ... ok
test d7_apply_returns_line_attribution_map ... ok
test d8_apply_rejects_path_traversal ... ok
test e1_diff_render_produces_valid_unified_format ... ok
test e2_diff_render_is_stable ... ok
test e4_diff_fallback_renders_correctly ... ok
test insert_after_line_1_does_not_shift_line_1_mapping ... ok
test negative_offset_returns_error_not_wrap ... ok
test patch_rejects_symlink_directory_in_workspace ... ok
test patch_rejects_symlink_file_in_workspace ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 24 tests
test attempt_id_is_not_trace_id ... ok
test baseline_timeout_patched_success_invalid ... ok
test bundle_has_all_required_fields ... ok
test bundle_serialization_round_trip_with_phase5_fields ... ok
test causal_question_format ... ok
test covariates_has_all_required_fields ... ok
test derive_status_phase5_semantics ... ok
test episode_content_includes_required_fields ... ok
test episode_meta_includes_phase5_fields ... ok
test forge_limits_has_patch_and_graph_limits ... ok
test hypothesis_edge_fields ... ok
test hypothesis_edge_kinds_complete ... ok
test pair_comparability_backend_mismatch ... ok
test pair_comparability_checks_mismatch ... ok
test pair_comparability_config_flags_mismatch ... ok
test patched_timeout_baseline_success_is_failure_outcome ... ok
test receipt_kind_variants ... ok
test receipt_ref_has_required_fields ... ok
test receipt_storage_variants ... ok
test sealed_bundle_blocks_reseal ... ok
test sealed_bundle_check_not_sealed ... ok
test stable_failure_edge_has_zero_confidence ... ok
test unit_definition_describes_paired_run ... ok
test verification_plan_records_dropped_steps ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 5 tests
test promotion_reads_assessment_from_canonical_bundle_not_legacy_split_column ... ok
test promotion_rejects_contradicted_evidence_assessment ... ok
test promotion_rejects_missing_verification_evidence ... ok
test promotion_rejects_when_improvement_does_not_clear_baseline ... ok
test promotion_succeeds_with_deterministic_eval_and_evidence_gate ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s


running 4 tests
test candidate_cannot_rewrite_python_evaluator_or_build_manifests ... ok
test empty_python_suite_cannot_be_a_passing_experiment ... ok
test python_tools_are_explicit_and_bad_summaries_refuse ... ok
test real_python_pair_corrects_failure_without_installing_dependencies ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s


running 8 tests
test actual_timeout_retains_completed_and_interrupted_sides_only ... ok
test empty_check_plan_cannot_be_a_passing_experiment ... ok
test empty_or_ignored_only_test_execution_is_not_a_passing_check ... ok
test invalid_modes_counts_and_unqualified_timing_refuse_before_execution ... ok
test patch_policy_refuses_before_workspace_or_commands ... ok
test repeated_pairs_execute_every_side_in_fresh_workspaces ... ok
test symlink_source_refuses_before_backend_dispatch ... ok
test within_policy_patch_runs_real_baseline_and_candidate ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.85s


running 3 tests
test b1_mindstate_render_is_deterministic ... ok
test b2_mindstate_snapshot ... ok
test mindstate_render_returns_result ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 8 tests
test a1_refuse_to_open_non_forge_db ... ok
test a2_refuse_to_open_missing_forge_meta ... ok
test a3_refuse_to_open_wrong_schema_hash ... ok
test a4_refuse_to_open_semantic_memory_signature ... ok
test a5_forge_writes_only_to_forge_db ... ok
test a6_refuse_to_open_reads_only_magic_bytes ... ok
test forge_db_round_trip ... ok
test transaction_commit_and_rollback ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 1 test
test forge_tool_receipt_sink_persists_raw_receipt ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 25 tests
test fixture_map::tests::empty_workspace_returns_no_fixtures ... ok
test fixture_map::tests::extract_package_name_parses_toml ... ok
test fixture_map::tests::resolve_signature_finds_crate ... ok
test governance_gate::tests::active_incident_blocks_execution ... ok
test governance_gate::tests::default_observation_blocks_execution ... ok
test governance_gate::tests::degradation_blocks_promotion_downgrades_to_advisory ... ok
test governance_gate::tests::fail_open_mode_returns_blocked_without_error ... ok
test governance_gate::tests::fail_open_mode_returns_default_on_empty_store ... ok
test governance_gate::tests::governance_receipt_roundtrip ... ok
test governance_gate::tests::invalid_authority_with_preflight_blocks ... ok
test governance_gate::tests::invalid_authority_without_preflight_allows ... ok
test governance_gate::tests::non_blocking_degradation_allows_execution ... ok
test governance_gate::tests::observe_governance_active_incident_blocks ... ok
test governance_gate::tests::observe_governance_empty_store_returns_error ... ok
test governance_gate::tests::observe_governance_reads_real_artifacts ... ok
test governance_gate::tests::observed_empty_observation_allows_execution ... ok
test governance_gate::tests::parse_bool_claim_values ... ok
test governance_gate::tests::pending_amendment_downgrades_to_advisory ... ok
test governance_gate::tests::strict_mode_allows_advisory_only ... ok
test governance_gate::tests::strict_mode_allows_on_allow_gate_result ... ok
test governance_gate::tests::strict_mode_allows_populated_store ... ok
test governance_gate::tests::strict_mode_blocks_on_blocked_gate_result ... ok
test governance_gate::tests::strict_mode_errors_on_empty_store ... ok
test receipts::tests::repair_record_preserves_variant_specific_semantics ... ok
test receipts::tests::verification_plan_artifact_carries_replay_recipe_and_blockers ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s


running 5 tests
test main_support::tests::detect_project_root_walks_up_from_nested_source_folder ... ok
test main_support::tests::managed_storage_root_prefers_project_root_over_source_subdir ... ok
test main_support::tests::normalize_user_path_expands_tilde ... ok
test main_support::tests::normalize_user_path_makes_relative_paths_absolute ... ok
test main_support::tests::prepare_runtime_layout_moves_managed_defaults_out_of_source_tree ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test local_edit_preserves_unaffected_chunk_ids_when_practical ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s


running 1 test
test bootstrap_source_creates_deterministic_chunk_records ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s


running 1 test
test bootstrap_source_deletions_are_manifest_driven_and_non_destructive ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s


running 1 test
test bootstrap_source_delta_reimports_only_changed_local_subtree ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s


running 1 test
test observation_reports_bootstrap_richness_across_thin_chunked_and_symbolized_states ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s


running 1 test
test bootstrap_source_marks_rust_line_scanner_as_degraded_for_unsupported_surface ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s


running 4 tests
test bootstrap_source_imports_workspace_into_canonical_observation_state ... ok
test bootstrap_source_is_idempotent_for_unchanged_workspace ... ok
test bootstrap_source_reports_empty_scan_cleanly ... ok
test bootstrap_source_reports_invalid_utf8_skips ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s


running 1 test
test bootstrap_source_extracts_symbols_for_rust_typescript_python_and_generic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s


running 2 tests
test derived_policy_changes_do_not_count_as_source_changes ... ok
test observation_current_state_comes_from_latest_manifest_only ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s


running 3 tests
test decision_audit_records_cheapest_and_blocked_steps_deterministically ... ok
test loop_iteration_report_carries_normalization_and_lineage_receipts ... ok
test target_taxonomy_normalizes_to_canonical_case_classes ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s


running 3 tests
test loop_runner_bounds_repeated_degraded_thin_export_cycles ... ok
test observation_degrades_honestly_when_kernel_payload_is_missing ... ok
test thin_export_yields_explicit_target_instead_of_invented_structure ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s


running 3 tests
test missing_oracle_outcome_cannot_fabricate_support_pair_or_completed_trial ... ok
test no_op_and_incomparable_results_do_not_gain_support ... ok
test repeated_raw_trials_and_exact_patch_identity_survive_bundle_and_v3_export ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.13s


running 1 test
test tool_runtime_to_pilot_to_control_lineage_stays_coherent ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 8 tests
test corrupt_forge_db_is_reported_explicitly ... ok
test empty_workspace_stays_empty_instead_of_forcing_import ... ok
test forge_bundles_without_imports_surface_exact_import_command ... ok
test imported_scope_remains_ready ... ok
test missing_storage_is_reported_explicitly ... ok
test namespace_mismatch_reports_dedicated_status_for_requested_namespace ... ok
test pilot_can_bootstrap_memory_from_recent_forge_bundles ... ok
test source_files_without_imports_surface_import_required ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s


running 1 test
test loop_runner_stops_when_time_budget_is_exhausted ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s


running 6 tests
test canonical_roundtrip_records_durable_failure_receipt_when_import_breaks ... ok
test loop_runner_executes_oracle_and_patch_families_with_real_roundtrips ... ok
test observation_only_accepts_explicit_paths ... ok
test observation_only_cannot_escape_into_execution_or_canonical_import ... ok
test observation_only_mode_never_executes_the_oracle_family ... ok
test observation_only_refuses_ambient_default_paths ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.94s


running 1 test
test reconstructs_observation_from_public_runtime_and_import_surfaces ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s


running 1 test
test oracle_plan_executes_and_roundtrips_through_canonical_v3_lane ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s


running 2 tests
test full_loop_missing_or_inapplicable_policy_refuses_instead_of_falling_back ... ok
test loop_defaults_do_not_mint_a_permissive_policy ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s


running 1 test
test repo_chat_ownership_answers_abstain_honestly ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s


running 1 test
test repo_chat_change_questions_use_latest_manifest_delta ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s


running 1 test
test repo_chat_routes_questions_explicitly ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test repo_chat_abstains_when_imported_source_memory_is_missing ... ok
test repo_chat_answers_with_grounded_workspace_source_citations ... ok
test repo_chat_filters_historical_imports_out_of_current_state_answers ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s


running 2 tests
test scoring_and_ordering_are_deterministic_and_retry_decay_applies ... ok
test target_selection_is_compared_against_a_simple_deterministic_baseline ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s


running 7 tests
test full_loop_accepts_scoped_approval_record ... ok
test full_loop_emits_v9_plan_and_consumer_only_proof_artifacts ... ok
test full_loop_policy_deny_blocks_execution_before_action ... ok
test full_loop_refuted_promoted_state_triggers_rollback_invalidation ... ok
test full_loop_requires_approval_when_policy_demands_it ... ok
test full_loop_uncalibrated_path_forces_advisory_only ... ok
test loop_report_boundary_parser_repairs_missing_approval_records ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.72s


running 7 tests
test tests::env_allowlist_matches_expected_prefixes ... ok
test tests::path_guard_rejects_absolute_and_parent_components ... ok
test tests::path_guard_rejects_multiple_escape_shapes ... ok
test tests::reject_symlinks_blocks_existing_symlink_segments ... ok
test tests::relative_path_guard_accepts_safe_relative_shapes ... ok
test tests::relative_path_guard_rejects_parent_escape_shapes ... ok
test tests::resolve_workspace_path_rejects_symlinked_parent_escape ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s


running 11 tests
test tests::create_parent_dirs_rejects_nested_escape_attempts ... ok
test tests::local_patch_fs_rejects_escape_paths ... ok
test tests::local_patch_fs_rejects_generated_escape_paths ... ok
test tests::local_patch_fs_rejects_reads_and_writes_through_symlink_segments ... ok
test tests::local_patch_fs_round_trips_file_contents ... ok
test tests::local_patch_fs_round_trips_safe_relative_paths ... ok
test tests::prepare_workspace_copies_fixture_recursively ... ok
test tests::prepare_workspace_skips_symlink_entries ... ok
test wave1_tests::local_patch_fs_rejects_escape_paths ... ok
test wave1_tests::local_patch_fs_round_trips_and_removes_files ... ok
test wave1_tests::prepare_workspace_copies_fixture_tree ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s


running 6 tests
test tests::apply_patch_rolls_back_when_a_later_edit_fails ... ok
test tests::idempotent_replace_patch_can_be_applied_twice ... ok
test tests::idempotent_replace_patch_remains_stable_for_arbitrary_line ... ok
test tests::parent_dir_paths_are_always_rejected ... ok
test tests::strategy_helpers_count_lines ... ok
test tests::validate_patch_rejects_workspace_escape_and_invalid_occurrence ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s


running 4 tests
test tests::denied_method_cannot_execute ... ok
test tests::duplicate_policy_vocab_is_rejected ... ok
test tests::scoped_approval_can_satisfy_runtime_generated_case_ids ... ok
test tests::v21_v24_policy_profiles_roundtrip ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test changed_plan_or_case_with_reused_ids_cannot_consume_decision ... ok
test modifying_public_decision_flags_cannot_turn_denial_into_permission ... ok
test policy_report_roundtrip_is_projection_not_execution_authority ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 4 tests
test continuity_policy_profile_v1_roundtrips ... ok
test delegation_policy_profile_v1_roundtrips ... ok
test effect_policy_profile_v1_roundtrips ... ok
test release_policy_profile_v1_roundtrips ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test invalid_snapshot_metadata_and_invalid_query_time_cannot_select_authority ... ok
test missing_future_expired_malformed_and_reversed_policies_do_not_apply ... ok
test temporal_policy_selection_compares_instants_not_wire_spelling ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 4 tests
test access_purpose_matrix_v1_roundtrips ... ok
test audit_extraction_policy_v1_roundtrips ... ok
test privacy_retention_profile_v1_roundtrips ... ok
test redaction_rule_set_v1_roundtrips ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test profile_p1_fixture_bundles_parse_into_owned_types ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 4 tests
test cross_boundary_transfer_class_v1_roundtrips ... ok
test locality_exception_v1_roundtrips ... ok
test residency_policy_profile_v1_roundtrips ... ok
test tenant_boundary_profile_v1_roundtrips ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test profile_p2_fixture_bundles_parse_into_owned_types ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test v14_examples_roundtrip_and_validate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test advisory_only_plan_cannot_mint_execution_permit ... ok
test approval_records_influence_admission_requirement ... ok
test policy_decision_carries_v25_citation_shape ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


```

## Verbatim final Clippy diagnostics

```text
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /home/sikmindz/.ares/runs/rsi-tranche-20261112T020205-0600/Libraries/quant-governor/Cargo.toml
workspace: /home/sikmindz/.ares/runs/rsi-tranche-20261112T020205-0600/Libraries/Cargo.toml
    Checking check-runner v0.1.0 (/home/sikmindz/.ares/runs/rsi-tranche-20261112T020205-0600/Libraries/Primitives/check-runner)
    Checking cea-core v0.1.0 (/home/sikmindz/.ares/runs/rsi-tranche-20261112T020205-0600/Libraries/Primitives/cea-core)
    Checking cea-store v0.1.0 (/home/sikmindz/.ares/runs/rsi-tranche-20261112T020205-0600/Libraries/Primitives/cea-store)
    Checking cea-sqlite v0.1.0 (/home/sikmindz/.ares/runs/rsi-tranche-20261112T020205-0600/Libraries/Primitives/cea-sqlite)
    Checking forge-engine v0.2.0 (/home/sikmindz/.ares/runs/rsi-tranche-20261112T020205-0600/Libraries/living-memory/living-memory)
    Checking verification-policy v0.1.0 (/home/sikmindz/.ares/runs/rsi-tranche-20261112T020205-0600/Libraries/verification-policy)
    Checking verification-adjudication v0.1.0 (/home/sikmindz/.ares/runs/rsi-tranche-20261112T020205-0600/Libraries/verification-adjudication)
    Checking forge-pilot v0.1.0 (/home/sikmindz/.ares/runs/rsi-tranche-20261112T020205-0600/Libraries/forge-pilot)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.50s

```
