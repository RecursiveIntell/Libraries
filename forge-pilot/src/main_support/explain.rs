use forge_pilot::{HaltReason, LoopReport};

pub(super) fn explain_observation(observation: &forge_pilot::Observation) -> String {
    let mut lines = vec![format!(
        "I checked namespace {}.",
        observation.scope_key.namespace
    )];
    lines.push(format!(
        "Resolved workspace path: {}.",
        observation.paths.workspace_path
    ));
    lines.push(format!(
        "Memory folder: {} (state: {:?}).",
        observation.paths.memory_dir, observation.paths.memory_dir_state
    ));
    lines.push(format!(
        "Forge DB: {} (state: {:?}).",
        observation.paths.forge_db_path, observation.paths.forge_db_state
    ));
    lines.push(format!(
        "Supported source files: {}. Import records found: {}. Import state: {:?}.",
        observation.status.supported_file_count,
        if observation.status.import_records_found {
            "yes"
        } else {
            "no"
        },
        observation.status.import_record_disposition
    ));
    lines.push(format!(
        "Forge evidence bundles available: {}.",
        observation.status.available_forge_bundle_count
    ));
    lines.push(format!(
        "I found {} claim versions, {} relation versions, and {} episodes.",
        observation.scope_health.total_claim_versions,
        observation.scope_health.total_relation_versions,
        observation.scope_health.total_episodes
    ));
    if observation.status.imported_file_count > 0 {
        lines.push(format!(
            "Imported current state from the latest manifest: {} file(s), {} chunk(s), {} symbol(s). Bootstrap richness: {:?}.",
            observation.status.imported_file_count,
            observation.status.imported_chunk_count,
            observation.status.imported_symbol_count,
            observation.status.bootstrap_richness
        ));
        if observation
            .source_inventory
            .imported_current_state
            .large_file_skip_count
            > 0
        {
            lines.push(format!(
                "Large files skipped and still visible in manifest truth: {}.",
                observation
                    .source_inventory
                    .imported_current_state
                    .large_file_skip_count
            ));
        }
    }
    lines.push(format!(
        "Disposition: {:?}. Next step: {}",
        observation.status.disposition, observation.status.exact_next_step
    ));
    if !observation.status.other_import_namespaces.is_empty() {
        lines.push(format!(
            "Recent imports for other namespaces: {}.",
            observation.status.other_import_namespaces.join(", ")
        ));
    }
    if observation.degradations.is_empty() {
        lines.push("There are no active degradation markers.".into());
    } else {
        lines.push(format!(
            "Active degradations: {}.",
            observation
                .degradations
                .iter()
                .map(|degradation| format!("{} ({})", degradation.kind, degradation.detail))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    lines.join("\n")
}

pub(super) fn explain_candidates(candidates: &[forge_pilot::TargetCandidate]) -> String {
    if candidates.is_empty() {
        return "I do not see any candidates worth investigating right now.".into();
    }

    let mut lines = vec![format!(
        "I found {} candidate(s). Here are the top ones:",
        candidates.len()
    )];
    for candidate in candidates.iter().take(5) {
        lines.push(format!(
            "- {} with urgency {:.2}. {}",
            candidate.stable_key, candidate.urgency, candidate.rationale
        ));
    }
    lines.join("\n")
}

pub(super) fn explain_import_report(report: &forge_pilot::ImportBootstrapReport) -> String {
    if report.forge_bundle_count == 0 {
        return format!(
            "No Forge evidence bundles are available yet for namespace {}. The pilot cannot bootstrap canonical semantic-memory state until Forge has evidence to export.",
            report.namespace
        );
    }

    format!(
        "Imported {} Forge evidence bundle(s) into canonical semantic-memory state for namespace {}.",
        report.imported_bundle_ids.len(),
        report.namespace
    )
}

pub(super) fn explain_bootstrap_report(report: &forge_pilot::BootstrapSourceReport) -> String {
    if report.scanned_file_count == 0 {
        return format!(
            "Bootstrap source scan found no supported files for namespace {} under {}.",
            report.namespace, report.workspace_path
        );
    }

    let mut summary = format!(
        "Bootstrap source imported {} file-backed source claim(s), {} chunk claim(s), and {} symbol claim(s) for namespace {}. Import status: {}.",
        report.imported_file_count,
        report.imported_chunk_count,
        report.imported_symbol_count,
        report.namespace,
        report
            .import_status
            .as_deref()
            .unwrap_or("not_imported")
    );
    if report.was_duplicate {
        summary.push_str(" The snapshot was already imported.");
    }
    if report.skipped_file_count > 0 {
        summary.push_str(&format!(" Skipped {} file(s).", report.skipped_file_count));
    }
    if report.manifest_id.is_some() {
        summary.push_str(&format!(
            " Current manifest: {} file(s), {} chunk(s), {} symbol(s), richness {:?}.",
            report.current_manifest_file_count,
            report.current_manifest_chunk_count,
            report.current_manifest_symbol_count,
            report.richness
        ));
        let source_delta = &report.source_delta;
        summary.push_str(&format!(
            " Source delta: new {}, changed {}, unchanged {}, deleted {}.",
            source_delta.new_file_count,
            source_delta.changed_file_count,
            source_delta.unchanged_file_count,
            source_delta.deleted_file_count
        ));
        if report.derived_delta.file_count > 0 {
            summary.push_str(&format!(
                " Derived-only changes affected {} source-unchanged file(s), {} chunk(s), and {} symbol(s).",
                report.derived_delta.source_unchanged_file_count,
                report.derived_delta.chunk_count,
                report.derived_delta.symbol_count
            ));
        }
    }
    summary
}

pub(super) fn explain_loop_report(report: &LoopReport) -> String {
    if report.iterations.is_empty() {
        if let Some(status) = &report.receipt.observation_status {
            return format!(
                "The closed loop stopped immediately for namespace {}. Status: {:?}. Supported source files: {}. Next step: {}",
                status.namespace_queried,
                status.disposition,
                status.supported_file_count,
                status.exact_next_step
            );
        }
        if report.halt_reason == HaltReason::ImportRequired {
            return "The closed loop stopped immediately because source files were found but this namespace has no usable canonical import yet.".into();
        }
        return format!(
            "The closed loop finished immediately. Halt reason: {:?}.",
            report.halt_reason
        );
    }

    if report.halt_reason == HaltReason::AdvisoryOnlyFallback {
        if let Some(status) = &report.receipt.observation_status {
            return format!(
                "The closed loop ran {} advisory bootstrap iteration(s) for namespace {}. Supported source files: {}. Next step: {}",
                report.iterations_completed,
                status.namespace_queried,
                status.supported_file_count,
                status.exact_next_step
            );
        }
    }

    let first = &report.iterations[0];
    let target = first
        .selected_target_key
        .clone()
        .unwrap_or_else(|| "none".into());
    let action = first
        .action_family
        .map(|family| format!("{family:?}"))
        .unwrap_or_else(|| "none".into());
    format!(
        "The closed loop ran {} iteration(s). It investigated {} using {}. Halt reason: {:?}. Actions executed: {}. Imports completed: {}.",
        report.iterations_completed,
        target,
        action,
        report.halt_reason,
        report.actions_executed,
        report.imports_completed
    )
}

pub(super) fn explain_loop_report_detailed(report: &LoopReport) -> String {
    let mut lines = vec![format!(
        "The loop finished after {} iteration(s). Halt reason: {:?}.",
        report.iterations_completed, report.halt_reason
    )];
    lines.push(format!(
        "It executed {} real action(s), completed {} export(s), and completed {} import(s).",
        report.actions_executed, report.exports_completed, report.imports_completed
    ));
    lines.push(format!(
        "It investigated these targets: {}.",
        if report.targets_investigated.is_empty() {
            "none".into()
        } else {
            report.targets_investigated.join(", ")
        }
    ));
    lines.push(format!(
        "Advisory-only steps were {} and degraded iterations totaled {}.",
        if report.advisory_only_steps {
            "present"
        } else {
            "not needed"
        },
        report.degraded_iterations
    ));

    for (index, iteration) in report.iterations.iter().enumerate() {
        lines.push(String::new());
        lines.push(format!("Iteration {}.", index + 1));
        lines.push(format!(
            "The pilot selected target {}.",
            iteration
                .selected_target_key
                .clone()
                .unwrap_or_else(|| "none".into())
        ));
        lines.push(format!(
            "It used action family {} and outcome {}.",
            iteration
                .action_family
                .map(|family| format!("{family:?}"))
                .unwrap_or_else(|| "none".into()),
            iteration
                .outcome_signature
                .clone()
                .unwrap_or_else(|| "no outcome signature".into())
        ));
        lines.push(format!(
            "This iteration was {} and {}.",
            if iteration.degraded {
                "degraded"
            } else {
                "not degraded"
            },
            if iteration.advisory_only {
                "advisory only"
            } else {
                "allowed to execute normally"
            }
        ));
        if let Some(normalization) = &iteration.target_normalization {
            lines.push(format!(
                "The canonical case class was {:?} with budget class {:?}.",
                normalization.canonical_case_class, normalization.budget_class
            ));
        }
        if let Some(audit) = &iteration.decision_audit {
            lines.push(format!(
                "The cheapest admissible step was {}.",
                audit
                    .cheapest_admissible
                    .map(|step| format!("{step:?}"))
                    .unwrap_or_else(|| "none".into())
            ));
            if !audit.blocked_steps.is_empty() {
                lines.push(format!(
                    "Blocked steps: {}.",
                    audit
                        .blocked_steps
                        .iter()
                        .map(|step| format!("{:?} because {}", step.step_kind, step.reason))
                        .collect::<Vec<_>>()
                        .join("; ")
                ));
            }
        }
        if let Some(stop) = &iteration.stop_rule_evaluation {
            lines.push(format!(
                "Stop-rule view: halt reason {}, retry cap reached {}, cooldown applied {}, damping applied {}.",
                stop.halt_reason,
                stop.retry_cap_reached,
                stop.cooldown_applied,
                stop.damping_applied
            ));
        }
        if let Some(lineage) = &iteration.lineage_receipt {
            lines.push(format!(
                "Lineage: case {}, plan {}, attempt {}.",
                lineage.case_id, lineage.plan_id, lineage.attempt_id
            ));
            if !lineage.degradation_markers.is_empty() {
                lines.push(format!(
                    "Degradation markers: {}.",
                    lineage.degradation_markers.join(", ")
                ));
            }
        }
    }

    lines.join("\n")
}
