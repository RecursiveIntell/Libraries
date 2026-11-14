mod common;
use common::{
    base_loop_config, import_v3_bundle, install_permissive_governance, open_forge_store,
    open_memory_store, point_config_at_dir, resources, sample_bundle, tempdir, write_source_file,
};
use forge_pilot::{LoopConfig, LoopRunner};
use knowledge_runtime::Scope;
use verification_policy::PolicySnapshot;

#[test]
fn loop_defaults_do_not_mint_a_permissive_policy() {
    assert!(LoopConfig::default().policy_snapshots.is_empty());
}

#[tokio::test]
async fn full_loop_missing_or_inapplicable_policy_refuses_instead_of_falling_back() {
    for state in ["missing", "future", "expired", "malformed"] {
        let dir = tempdir();
        let memory = open_memory_store(dir.path());
        let forge = open_forge_store(dir.path());
        let mut cfg = base_loop_config(Scope::new(format!("rsi-policy-{state}")));
        point_config_at_dir(&mut cfg, dir.path());
        cfg.max_iterations = 1;
        write_source_file(
            dir.path(),
            "src/lib.rs",
            "pub fn policy_fixture() -> bool { true }\n",
        );
        import_v3_bundle(
            &memory,
            &forge,
            &cfg.scope.namespace,
            &sample_bundle("rsi-policy"),
        )
        .await;
        install_permissive_governance(&memory).await;
        cfg.policy_snapshots = match state {
            "missing" => Vec::new(),
            "future" => vec![PolicySnapshot::permissive("future", "2099-01-01T00:00:00Z")],
            "expired" => {
                let mut p = PolicySnapshot::permissive("expired", "2020-01-01T00:00:00Z");
                p.effective_to = Some("2021-01-01T00:00:00Z".into());
                vec![p]
            }
            "malformed" => vec![PolicySnapshot::permissive(
                "malformed",
                "0000-not-an-instant",
            )],
            _ => unreachable!(),
        };
        let resources = resources(memory, forge, &cfg);
        let mut runner = LoopRunner::new(cfg, resources);
        let result = runner.run().await;
        assert!(
            runner
                .history()
                .entries()
                .all(|(_, entry)| entry.retry_count == 0),
            "unavailable policy consumed target retry state: {state}"
        );
        assert!(
            matches!(result, Err(ref error) if error.kind() == "policy_unavailable"),
            "accepted {state}: {result:?}"
        );
    }
}
