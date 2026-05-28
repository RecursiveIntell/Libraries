use schemars::schema_for;
use scr_audit_adapter::AuditFixtureCaseV1;
use scr_kernel::{ControlDecisionReceiptV1, ControlEvaluationInputV1, ScrError};
use scr_reference::{evaluate_with_policy, load_policy_from_toml};
use serde::Serialize;
use serde_json::Value;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), ScrError> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("canonicalize-policy") => {
            let policy_path = args
                .next()
                .unwrap_or_else(|| "policies/audit_policy_v1.toml".into());
            let out_path = args
                .next()
                .unwrap_or_else(|| "policies/audit_policy_v1.canonical.json".into());
            let source =
                fs::read_to_string(&policy_path).map_err(|err| io_error(&policy_path, err))?;
            let policy = load_policy_from_toml(&source)?;
            write_text(&out_path, &(policy.canonical_json + "\n"))?;
            println!("{}", policy.canonical_hash);
            Ok(())
        }
        Some("generate-schemas") => {
            let out_dir = args.next().unwrap_or_else(|| "schemas/generated".into());
            generate_schemas(Path::new(&out_dir))
        }
        Some("generate-fixtures") => {
            let case_dir = args.next().unwrap_or_else(|| "fixtures/audit/cases".into());
            let expected_dir = args
                .next()
                .unwrap_or_else(|| "fixtures/audit/expected".into());
            let policy_path = args
                .next()
                .unwrap_or_else(|| "policies/audit_policy_v1.toml".into());
            eval_fixtures(
                Path::new(&case_dir),
                Path::new(&expected_dir),
                Path::new(&policy_path),
                FixtureMode::Generate,
            )
        }
        Some("verify-fixtures") => {
            let case_dir = args.next().unwrap_or_else(|| "fixtures/audit/cases".into());
            let expected_dir = args
                .next()
                .unwrap_or_else(|| "fixtures/audit/expected".into());
            let policy_path = args
                .next()
                .unwrap_or_else(|| "policies/audit_policy_v1.toml".into());
            eval_fixtures(
                Path::new(&case_dir),
                Path::new(&expected_dir),
                Path::new(&policy_path),
                FixtureMode::Verify,
            )
        }
        Some("eval-fixture") => {
            let case_path = args.next().ok_or_else(|| {
                ScrError::PolicyValidationFailed("eval-fixture requires case path".to_string())
            })?;
            let policy_path = args
                .next()
                .unwrap_or_else(|| "policies/audit_policy_v1.toml".into());
            let source =
                fs::read_to_string(&policy_path).map_err(|err| io_error(&policy_path, err))?;
            let policy = load_policy_from_toml(&source)?;
            let case_body =
                fs::read_to_string(&case_path).map_err(|err| io_error(&case_path, err))?;
            let case: AuditFixtureCaseV1 = serde_json::from_str(&case_body)
                .map_err(|err| ScrError::SerializationFailed(err.to_string()))?;
            let receipt = evaluate_with_policy(case.into_input()?, &policy)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&receipt)
                    .map_err(|err| ScrError::SerializationFailed(err.to_string()))?
            );
            Ok(())
        }
        Some("eval-fixtures") => {
            let case_dir = args.next().unwrap_or_else(|| "fixtures/audit/cases".into());
            let expected_dir = args
                .next()
                .unwrap_or_else(|| "fixtures/audit/expected".into());
            let policy_path = args
                .next()
                .unwrap_or_else(|| "policies/audit_policy_v1.toml".into());
            eval_fixtures(
                Path::new(&case_dir),
                Path::new(&expected_dir),
                Path::new(&policy_path),
                FixtureMode::Verify,
            )
        }
        Some("explain-receipt") => {
            let receipt_path = args.next().ok_or_else(|| {
                ScrError::PolicyValidationFailed("explain-receipt requires receipt path".to_string())
            })?;
            explain_receipt(Path::new(&receipt_path))
        }
        _ => Err(ScrError::PolicyValidationFailed(
            "usage: scr-cli canonicalize-policy|generate-schemas|generate-fixtures|verify-fixtures|eval-fixture|eval-fixtures|explain-receipt".to_string(),
        )),
    }
}

fn generate_schemas(out_dir: &Path) -> Result<(), ScrError> {
    fs::create_dir_all(out_dir).map_err(|err| io_error_path(out_dir, err))?;
    write_schema(
        out_dir.join("control-evaluation-input-v1.schema.json"),
        &schema_for!(ControlEvaluationInputV1),
        ControlEvaluationInputV1::SCHEMA_VERSION,
    )?;
    write_schema(
        out_dir.join("control-decision-receipt-v1.schema.json"),
        &schema_for!(ControlDecisionReceiptV1),
        ControlDecisionReceiptV1::SCHEMA_VERSION,
    )?;
    write_schema(
        out_dir.join("audit-fixture-case-v1.schema.json"),
        &schema_for!(AuditFixtureCaseV1),
        AuditFixtureCaseV1::SCHEMA_VERSION,
    )?;
    Ok(())
}

fn write_schema(
    path: PathBuf,
    schema: &impl Serialize,
    schema_version: &str,
) -> Result<(), ScrError> {
    let mut schema = serde_json::to_value(schema)
        .map_err(|err| ScrError::SerializationFailed(err.to_string()))?;
    enforce_schema_contracts(&mut schema, schema_version);
    let encoded = serde_json::to_string_pretty(&schema)
        .map_err(|err| ScrError::SerializationFailed(err.to_string()))?;
    write_text(path, &(encoded + "\n"))
}

fn enforce_schema_contracts(schema: &mut Value, schema_version: &str) {
    if let Value::Object(map) = schema {
        if let Some(Value::Object(properties)) = map.get_mut("properties") {
            if let Some(schema_version_prop) = properties.get_mut("schema_version") {
                if let Value::Object(obj) = schema_version_prop {
                    obj.insert(
                        "const".to_string(),
                        Value::String(schema_version.to_string()),
                    );
                } else {
                    *schema_version_prop = serde_json::json!({
                        "type": "string",
                        "const": schema_version,
                    });
                }
            }
        }
        if let Some(Value::Object(definitions)) = map.get_mut("definitions") {
            for def_name in ["ScoreBps", "WeightBps"] {
                if let Some(Value::Object(definition)) = definitions.get_mut(def_name) {
                    definition.insert("type".to_string(), Value::String("integer".to_string()));
                    definition.insert("minimum".to_string(), Value::from(0));
                    definition.insert("maximum".to_string(), Value::from(10_000));
                }
            }
        }
        if let Some(Value::Object(definitions)) = map.get_mut("$defs") {
            for def_name in ["ScoreBps", "WeightBps"] {
                if let Some(Value::Object(definition)) = definitions.get_mut(def_name) {
                    definition.insert("type".to_string(), Value::String("integer".to_string()));
                    definition.insert("minimum".to_string(), Value::from(0));
                    definition.insert("maximum".to_string(), Value::from(10_000));
                }
            }
        }
        for value in map.values_mut() {
            enforce_schema_contracts(value, schema_version);
        }
    } else if let Value::Array(values) = schema {
        for value in values {
            enforce_schema_contracts(value, schema_version);
        }
    }
}

enum FixtureMode {
    Generate,
    Verify,
}

fn eval_fixtures(
    case_dir: &Path,
    expected_dir: &Path,
    policy_path: &Path,
    mode: FixtureMode,
) -> Result<(), ScrError> {
    let source = fs::read_to_string(policy_path).map_err(|err| io_error_path(policy_path, err))?;
    let policy = load_policy_from_toml(&source)?;
    fs::create_dir_all(expected_dir).map_err(|err| io_error_path(expected_dir, err))?;

    let mut entries = fs::read_dir(case_dir)
        .map_err(|err| io_error_path(case_dir, err))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| ScrError::SerializationFailed(err.to_string()))?;
    entries.sort();

    for path in entries {
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let body = fs::read_to_string(&path).map_err(|err| io_error_path(&path, err))?;
        let case: AuditFixtureCaseV1 = serde_json::from_str(&body)
            .map_err(|err| ScrError::SerializationFailed(err.to_string()))?;
        let receipt = evaluate_with_policy(case.into_input()?, &policy)?;
        let output_path = expected_dir.join(path.file_name().ok_or_else(|| {
            ScrError::SerializationFailed("fixture path missing file name".to_string())
        })?);
        let encoded = serde_json::to_string_pretty(&receipt)
            .map_err(|err| ScrError::SerializationFailed(err.to_string()))?;
        match mode {
            FixtureMode::Generate => write_text(output_path, &(encoded + "\n"))?,
            FixtureMode::Verify => {
                let expected = fs::read_to_string(&output_path).map_err(|err| {
                    ScrError::SerializationFailed(format!(
                        "fixture expected missing: {output_path:?}: {err}"
                    ))
                })?;
                let expected_receipt: ControlDecisionReceiptV1 = serde_json::from_str(&expected)
                    .map_err(|err| {
                        ScrError::SerializationFailed(format!(
                            "invalid expected receipt {}: {err}",
                            output_path.display()
                        ))
                    })?;
                if expected_receipt != receipt {
                    return Err(ScrError::SerializationFailed(format!(
                        "fixture mismatch in {}",
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("unknown")
                    )));
                }
            }
        }
    }
    Ok(())
}

fn explain_receipt(path: &Path) -> Result<(), ScrError> {
    let body = fs::read_to_string(path).map_err(|err| io_error_path(path, err))?;
    let receipt: ControlDecisionReceiptV1 = serde_json::from_str(&body)
        .map_err(|err| ScrError::SerializationFailed(err.to_string()))?;
    receipt.validate()?;

    let mut out = String::new();
    writeln!(
        &mut out,
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": receipt.schema_version,
            "input_hash": receipt.input_hash,
            "canonical_policy_hash": receipt.canonical_policy_hash,
            "evaluator_algorithm_id": receipt.evaluator_algorithm_id,
            "evaluator_algorithm_hash": receipt.evaluator_algorithm_hash,
            "hard_rules_checked": receipt.hard_rules_checked,
            "hard_rules_triggered": receipt.hard_rules_triggered,
            "minimum_action_floors_applied": receipt.minimum_action_floors_applied,
            "chosen_action": receipt.chosen_action,
            "rejected_actions": receipt.rejected_actions,
            "reason_codes": receipt.reason_codes,
            "valid_time_basis": receipt.valid_time_basis,
            "recorded_time": receipt.recorded_time,
        }))
        .map_err(|err| ScrError::SerializationFailed(err.to_string()))?
    )
    .map_err(|err| ScrError::SerializationFailed(err.to_string()))?;
    println!("{out}");
    Ok(())
}

fn write_text(path: impl AsRef<Path>, body: &str) -> Result<(), ScrError> {
    fs::write(path.as_ref(), body).map_err(|err| io_error_path(path.as_ref(), err))
}

fn io_error(path: &str, err: std::io::Error) -> ScrError {
    ScrError::SerializationFailed(format!("{path}: {err}"))
}

fn io_error_path(path: &Path, err: std::io::Error) -> ScrError {
    ScrError::SerializationFailed(format!("{}: {err}", path.display()))
}
