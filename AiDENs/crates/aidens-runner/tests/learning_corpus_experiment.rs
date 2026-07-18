use aidens_runner::learning_experiment::{run_v2_repeated_paired, V2RepeatedPairedConfig};
use forge_engine::{ForgeConfig, TrialSide};
use std::path::{Path, PathBuf};

fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/learning-coding-agent/v2")
}

fn live_config(image: String) -> V2RepeatedPairedConfig {
    V2RepeatedPairedConfig {
        corpus_root: corpus_root(),
        pairs_per_task: 4,
        trial_order_seed: 7,
        forge: ForgeConfig {
            mode: "sealed_local".into(),
            execution_backend_preference: "container".into(),
            container_runtime_preference: "podman".into(),
            sealed_allow_host_backend: false,
            container: forge_engine::config::ContainerConfig {
                rust_image: image,
                ..Default::default()
            },
            ..Default::default()
        },
    }
}

#[tokio::test]
#[ignore = "requires live rootless Podman and AIDENS_PINNED_RUST_IMAGE"]
async fn live_adapter_runs_sixty_pairs_in_sealed_podman() {
    let image = std::env::var("AIDENS_PINNED_RUST_IMAGE").unwrap();
    let result = run_v2_repeated_paired(live_config(image)).await.unwrap();

    let mismatches = result
        .task_runs
        .iter()
        .flat_map(|task| {
            task.experiment.trials.iter().filter_map(|trial| {
                let valid = match trial.side {
                    TrialSide::Baseline => !trial.clippy_pass || !trial.test_pass,
                    TrialSide::Patched => trial.fmt_pass && trial.clippy_pass && trial.test_pass,
                };
                (!valid).then(|| {
                    format!(
                        "{} pair={} side={:?} fmt={} clippy={} test={}",
                        task.task_id,
                        trial.pair_index,
                        trial.side,
                        trial.fmt_pass,
                        trial.clippy_pass,
                        trial.test_pass
                    )
                })
            })
        })
        .collect::<Vec<_>>();

    assert_eq!(result.total_pairs, 60);
    assert_eq!(result.total_sides, 120);
    assert!(mismatches.is_empty(), "mismatched sides: {mismatches:#?}");
}
