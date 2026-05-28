use aidens_cli::{run, Cli};
use aidens_contracts::{AiDENsCompiledPlanV1, AiDENsDoctorReportV1, MemoryModeV1};
use clap::Parser;

#[test]
fn profile_explain_coding_agent_is_visible_and_non_magical() {
    let cli = Cli::parse_from(["aidens", "profile", "explain", "coding-agent"]);
    let output = run(cli).expect("profile explain works");
    assert!(output.contains("coding-agent"));
    assert!(output.contains("requires approval") || output.contains("permit"));
    assert!(output.contains("shell") || output.contains("write"));
    assert!(!output
        .to_ascii_lowercase()
        .contains("auto-approved by default"));
}

#[test]
fn plan_validate_compile_and_doctor_accept_explicit_mock_config() {
    let dir = std::env::temp_dir().join(format!("aidens-next-cli-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cfg = dir.join("aidens.toml");
    let out = dir.join("plan.json");
    std::fs::write(
        &cfg,
        r#"
app_id = "next-cli"
memory_mode = "disabled"
receipt_level = "full"

[provider]
kind = "mock"
model = "mock-model"
mock_response = "mock response"
"#,
    )
    .unwrap();

    let validate = run(Cli::parse_from([
        "aidens",
        "plan",
        "validate",
        "--config",
        cfg.to_str().unwrap(),
    ]))
    .expect("plan validate");
    assert!(validate.to_ascii_lowercase().contains("valid"));

    let compile = run(Cli::parse_from([
        "aidens",
        "plan",
        "compile",
        "--config",
        cfg.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]))
    .expect("plan compile");
    assert!(compile.to_ascii_lowercase().contains("compiled") || out.exists());
    assert!(out.exists());

    let doctor = run(Cli::parse_from([
        "aidens",
        "doctor",
        "--config",
        cfg.to_str().unwrap(),
    ]))
    .expect("doctor");
    assert!(doctor.contains("mock"));
    assert!(!doctor.to_ascii_lowercase().contains("skeletal"));

    let compiled: AiDENsCompiledPlanV1 =
        serde_json::from_str(&std::fs::read_to_string(&out).unwrap()).unwrap();
    let doctor_report: AiDENsDoctorReportV1 = serde_json::from_str(&doctor).unwrap();
    assert!(compiled.parity_report.is_passing());
    assert_eq!(
        compiled.config_apply_receipt.memory_mode,
        MemoryModeV1::Disabled
    );
    assert_eq!(
        compiled.doctor.sections.get("provider_route"),
        doctor_report.sections.get("provider_route")
    );
    assert_eq!(
        compiled.doctor.sections.get("tools"),
        doctor_report.sections.get("tools")
    );
    assert_eq!(
        compiled.doctor.sections.get("memory"),
        doctor_report.sections.get("memory")
    );
    assert_eq!(
        compiled.doctor.sections.get("scaffold_surfaces"),
        doctor_report.sections.get("scaffold_surfaces")
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn plan_validate_rejects_memory_required_without_memory_store() {
    let dir = std::env::temp_dir().join(format!("aidens-next-cli-memory-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cfg = dir.join("aidens.toml");
    std::fs::write(
        &cfg,
        r#"
app_id = "next-cli"
memory_mode = "required"
receipt_level = "full"

[provider]
kind = "mock"
model = "mock-model"
mock_response = "mock response"
"#,
    )
    .unwrap();

    let error = run(Cli::parse_from([
        "aidens",
        "plan",
        "validate",
        "--config",
        cfg.to_str().unwrap(),
    ]))
    .unwrap_err();

    assert!(error.to_string().contains("memory_mode=required"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn plan_validate_doctor_and_compile_accept_memory_required_with_store() {
    let dir = std::env::temp_dir().join(format!(
        "aidens-next-cli-memory-store-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cfg = dir.join("aidens.toml");
    let out = dir.join("plan.json");
    std::fs::write(
        &cfg,
        r#"
app_id = "next-cli-memory"
memory_mode = "required"
receipt_level = "full"

[provider]
kind = "mock"
model = "mock-model"
mock_response = "mock response"

[memory]
store_root = "memory"
"#,
    )
    .unwrap();

    let validate = run(Cli::parse_from([
        "aidens",
        "plan",
        "validate",
        "--config",
        cfg.to_str().unwrap(),
    ]))
    .expect("plan validate with memory store");
    assert!(validate.contains("valid"));

    let doctor = run(Cli::parse_from([
        "aidens",
        "doctor",
        "--config",
        cfg.to_str().unwrap(),
    ]))
    .expect("doctor with memory store");
    assert!(doctor.contains("memory_mode=required"));
    assert!(doctor.contains("durable memory store configured"));

    run(Cli::parse_from([
        "aidens",
        "plan",
        "compile",
        "--config",
        cfg.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]))
    .expect("plan compile with memory store");
    let compiled: AiDENsCompiledPlanV1 =
        serde_json::from_str(&std::fs::read_to_string(&out).unwrap()).unwrap();
    assert!(compiled.parity_report.is_passing());
    assert_eq!(
        compiled.config_apply_receipt.memory_mode,
        MemoryModeV1::Required
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn plan_validate_rejects_disabled_provider_when_provider_is_required() {
    let dir = std::env::temp_dir().join(format!("aidens-next-cli-disabled-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cfg = dir.join("aidens.toml");
    std::fs::write(
        &cfg,
        r#"
app_id = "next-cli"
profile_id = "coding-agent"
memory_mode = "disabled"
receipt_level = "full"

[provider]
kind = "disabled"
model = "none"
"#,
    )
    .unwrap();

    let error = run(Cli::parse_from([
        "aidens",
        "plan",
        "validate",
        "--config",
        cfg.to_str().unwrap(),
    ]))
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("provider_required plan cannot treat disabled provider as valid"));
    let _ = std::fs::remove_dir_all(&dir);
}
