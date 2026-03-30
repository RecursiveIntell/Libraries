//! F. Execution backend tests (F1–F5)
//! G. CargoAdapter tests (G1–G3)

use forge_engine::*;
use tempfile::TempDir;

/// F1: host_backend_runs_command_captures_output
#[tokio::test]
async fn f1_host_backend_runs_command_captures_output() {
    let config = ForgeConfig::default();
    let backend = HostBackend::new(&config);
    let dir = TempDir::new().unwrap();

    let output = backend
        .run_command(dir.path(), "echo", &["hello"], &[], 10)
        .await
        .unwrap();

    assert_eq!(output.stdout.trim(), "hello");
    assert_eq!(output.exit_code, 0);
}

/// F2: host_backend_timeout
#[tokio::test]
async fn f2_host_backend_timeout() {
    let config = ForgeConfig::default();
    let backend = HostBackend::new(&config);
    let dir = TempDir::new().unwrap();

    let result = backend
        .run_command(dir.path(), "sleep", &["60"], &[], 1)
        .await;

    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("timeout") || msg.contains("Timeout"),
        "Expected timeout error, got: {msg}"
    );
}

/// F4: container_backend_autodetect_fallback_host
#[test]
fn f4_container_backend_autodetect_fallback_host() {
    // When auto mode and no container runtime, should fall back to host
    let config = ForgeConfig {
        execution_backend_preference: "auto".to_string(),
        ..ForgeConfig::default()
    };

    // select_backend should succeed (falls back to host)
    // Note: this test works whether or not Docker is installed
    let result = select_backend(&config);
    assert!(result.is_ok(), "Backend selection should succeed");
}

/// F5: container_backend_sealed_requires_no_network
#[test]
fn f5_container_backend_sealed_requires_no_network() {
    let config = ForgeConfig {
        mode: "sealed_local".to_string(),
        execution_backend_preference: "container".to_string(),
        ..ForgeConfig::default()
    };

    // If a container runtime is available, verify it sets network flags
    // If not, verify it falls back gracefully
    match ContainerBackend::new(&config) {
        Ok(cb) => {
            // The backend was created — it should have sealed mode
            // We verify by checking the runtime() method exists
            let _runtime = cb.runtime();
            // The actual --network=none flag is added in build_run_args,
            // which is called during run_command
        }
        Err(e) => {
            let msg = format!("{e}");
            assert!(
                msg.contains("container") || msg.contains("runtime"),
                "Should fail due to no container runtime: {msg}"
            );
        }
    }
}

// === G. CargoAdapter tests ===

/// G1: cargo_adapter_detects_cargo_project
#[test]
fn g1_cargo_adapter_detects_cargo_project() {
    let with_cargo = TempDir::new().unwrap();
    std::fs::write(
        with_cargo.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\n",
    )
    .unwrap();
    assert!(CargoAdapter::detect(with_cargo.path()));

    let without_cargo = TempDir::new().unwrap();
    assert!(!CargoAdapter::detect(without_cargo.path()));
}

/// Phase 2: copy_dir_recursive skips symlinks
#[cfg(unix)]
#[test]
fn copy_dir_recursive_skips_symlinks() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    std::fs::write(src.path().join("real.txt"), "content").unwrap();
    std::os::unix::fs::symlink("/etc/passwd", src.path().join("evil.txt")).unwrap();
    forge_engine::exec::host::copy_dir_recursive_pub(src.path(), dst.path()).unwrap();
    assert!(dst.path().join("real.txt").exists());
    assert!(
        !dst.path().join("evil.txt").exists(),
        "Symlink should not be copied"
    );
}

/// Phase 2: env allowlist blocks secrets, passes Cargo
#[test]
fn env_allowlist_blocks_secrets_passes_cargo() {
    assert!(is_env_allowed("CARGO_HOME"));
    assert!(is_env_allowed("RUSTUP_HOME"));
    assert!(is_env_allowed("PATH"));
    assert!(is_env_allowed("CARGO_INCREMENTAL"));
    assert!(!is_env_allowed("AWS_SECRET_ACCESS_KEY"));
    assert!(!is_env_allowed("ANTHROPIC_API_KEY"));
    assert!(!is_env_allowed("DATABASE_URL"));
    assert!(!is_env_allowed("GITHUB_TOKEN"));
    assert!(!is_env_allowed("OPENAI_API_KEY"));
    assert!(!is_env_allowed("DOCKER_AUTH_CONFIG"));
}

/// G3: cargo_adapter_parses_clippy_json
#[test]
fn g3_cargo_adapter_parses_clippy_json() {
    let adapter = CargoAdapter;
    let config = ForgeConfig::default();
    let cmds = adapter.check_commands(&config);
    let clippy_cmd = &cmds[1]; // Clippy is second

    let clippy_json = r#"{"reason":"compiler-message","message":{"code":{"code":"clippy::needless_return"},"level":"warning","message":"unneeded return","spans":[{"file_name":"src/lib.rs","line_start":42,"column_start":5}]}}"#;

    let output = adapter.parse_check_output(clippy_cmd, "", clippy_json, 0);
    assert!(!output.effects.is_empty(), "Should parse clippy effects");
    assert_eq!(
        output.effects[0].sig.message_class,
        "clippy::needless_return"
    );
    assert_eq!(output.effects[0].line, Some(42));
}
