mod common;

use common::{
    base_loop_config, import_thin_v3_batch, open_forge_store, open_memory_store,
    point_config_at_dir, resources, sample_bundle, tempdir, write_source_file,
};
use forge_pilot::{bootstrap_source_workspace, observe_scope, BootstrapRichness};
use knowledge_runtime::Scope;

#[tokio::test]
async fn observation_reports_bootstrap_richness_across_thin_chunked_and_symbolized_states() {
    let thin_dir = tempdir();
    let thin_memory = open_memory_store(thin_dir.path());
    let thin_forge = open_forge_store(thin_dir.path());
    let thin_scope = Scope::new("bootstrap-richness-thin");
    let mut thin_config = base_loop_config(thin_scope.clone());
    point_config_at_dir(&mut thin_config, thin_dir.path());
    import_thin_v3_batch(
        &thin_memory,
        &thin_forge,
        &thin_scope.namespace,
        &sample_bundle("thin"),
    )
    .await;
    let thin_resources = resources(thin_memory, thin_forge, &thin_config);
    let thin_observation = observe_scope(
        &thin_resources.runtime,
        &thin_resources.memory_store,
        &thin_config,
    )
    .await
    .unwrap();
    assert_eq!(
        thin_observation.status.bootstrap_richness,
        BootstrapRichness::Thin
    );

    let chunked_dir = tempdir();
    let large_rust = (0..5000)
        .map(|index| format!("pub fn chunked_{index}() -> usize {{ {index} }}\n"))
        .collect::<String>();
    write_source_file(chunked_dir.path(), "src/lib.rs", &large_rust);
    let chunked_memory = open_memory_store(chunked_dir.path());
    let chunked_forge = open_forge_store(chunked_dir.path());
    let chunked_scope = Scope::new("bootstrap-richness-chunked");
    let mut chunked_config = base_loop_config(chunked_scope.clone());
    point_config_at_dir(&mut chunked_config, chunked_dir.path());
    bootstrap_source_workspace(&chunked_memory, &chunked_config)
        .await
        .unwrap();
    let chunked_resources = resources(chunked_memory, chunked_forge, &chunked_config);
    let chunked_observation = observe_scope(
        &chunked_resources.runtime,
        &chunked_resources.memory_store,
        &chunked_config,
    )
    .await
    .unwrap();
    assert_eq!(
        chunked_observation.status.bootstrap_richness,
        BootstrapRichness::Chunked
    );

    let symbolized_dir = tempdir();
    write_source_file(
        symbolized_dir.path(),
        "src/lib.rs",
        "pub fn rich() -> bool { true }\n",
    );
    let symbolized_memory = open_memory_store(symbolized_dir.path());
    let symbolized_forge = open_forge_store(symbolized_dir.path());
    let symbolized_scope = Scope::new("bootstrap-richness-symbolized");
    let mut symbolized_config = base_loop_config(symbolized_scope.clone());
    point_config_at_dir(&mut symbolized_config, symbolized_dir.path());
    bootstrap_source_workspace(&symbolized_memory, &symbolized_config)
        .await
        .unwrap();
    let symbolized_resources = resources(symbolized_memory, symbolized_forge, &symbolized_config);
    let symbolized_observation = observe_scope(
        &symbolized_resources.runtime,
        &symbolized_resources.memory_store,
        &symbolized_config,
    )
    .await
    .unwrap();
    assert_eq!(
        symbolized_observation.status.bootstrap_richness,
        BootstrapRichness::Symbolized
    );
    assert!(symbolized_observation.status.imported_symbol_count > 0);
}
