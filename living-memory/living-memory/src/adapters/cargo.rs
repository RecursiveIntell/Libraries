use std::path::Path;

use crate::config::ForgeConfig;
use crate::exec::backend::*;

/// ProjectAdapter trait — adapts Forge to specific project types.
pub trait ProjectAdapter: Send + Sync {
    fn detect(workspace: &Path) -> bool
    where
        Self: Sized;
    fn name(&self) -> &str;
    fn check_commands(&self, config: &ForgeConfig) -> Vec<CheckCommand>;
    fn parse_check_output(
        &self,
        cmd: &CheckCommand,
        stdout: &str,
        stderr: &str,
        exit_code: i32,
    ) -> ParsedCheckOutput;
}

/// Cargo adapter — for Rust projects.
pub struct CargoAdapter;

impl ProjectAdapter for CargoAdapter {
    fn detect(workspace: &Path) -> bool {
        workspace.join("Cargo.toml").exists()
    }

    fn name(&self) -> &str {
        "cargo"
    }

    fn check_commands(&self, _config: &ForgeConfig) -> Vec<CheckCommand> {
        vec![
            CheckCommand {
                kind: CheckKind::Fmt,
                program: "cargo".to_string(),
                args: vec![
                    "fmt".to_string(),
                    "--all".to_string(),
                    "--".to_string(),
                    "--check".to_string(),
                ],
                env: vec![],
            },
            CheckCommand {
                kind: CheckKind::Clippy,
                program: "cargo".to_string(),
                args: vec![
                    "clippy".to_string(),
                    "--all-targets".to_string(),
                    "--all-features".to_string(),
                    "--message-format=json".to_string(),
                    "--".to_string(),
                    "-D".to_string(),
                    "warnings".to_string(),
                ],
                env: vec![],
            },
            CheckCommand {
                kind: CheckKind::Test,
                program: "cargo".to_string(),
                args: vec![
                    "test".to_string(),
                    "--all".to_string(),
                    "--all-features".to_string(),
                ],
                env: vec![],
            },
        ]
    }

    fn parse_check_output(
        &self,
        cmd: &CheckCommand,
        stdout: &str,
        stderr: &str,
        exit_code: i32,
    ) -> ParsedCheckOutput {
        match cmd.kind {
            CheckKind::Fmt => parse_fmt_output(stdout, stderr, exit_code),
            CheckKind::Clippy => parse_clippy_output(stdout, stderr, exit_code),
            CheckKind::Test => parse_test_output(stdout, stderr, exit_code),
        }
    }
}

/// Parse cargo fmt output.
fn parse_fmt_output(stdout: &str, stderr: &str, exit_code: i32) -> ParsedCheckOutput {
    let mut effects = Vec::new();

    // Parse lines like "Diff in src/lib.rs at line 42:"
    for line in stdout.lines().chain(stderr.lines()) {
        if line.starts_with("Diff in ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let file = parts[2].trim_end_matches(':');
                effects.push(LocatedEffect {
                    file: Some(std::path::PathBuf::from(file)),
                    line: None,
                    col: None,
                    message: line.to_string(),
                    sig: EffectSignature {
                        check_kind: "fmt".to_string(),
                        outcome: "fail".to_string(),
                        severity: "warning".to_string(),
                        message_class: file.to_string(),
                        line_offset_from_edit: None,
                    },
                });
            }
        }
    }

    ParsedCheckOutput {
        check_kind: CheckKind::Fmt,
        exit_code,
        effects,
        raw_stdout: stdout.to_string(),
        raw_stderr: stderr.to_string(),
    }
}

/// Parse cargo clippy JSON output.
fn parse_clippy_output(stdout: &str, stderr: &str, exit_code: i32) -> ParsedCheckOutput {
    let mut effects = Vec::new();

    // Clippy outputs JSON on stderr when --message-format=json is used
    for line in stderr.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('{') {
            continue;
        }

        // Parse JSON, tolerating malformed input (CEA security concern)
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
        let value = match parsed {
            Ok(v) => v,
            Err(_) => continue, // skip malformed JSON gracefully
        };

        // Only process compiler-message reason
        if value.get("reason").and_then(|r| r.as_str()) != Some("compiler-message") {
            continue;
        }

        let message = match value.get("message") {
            Some(m) => m,
            None => continue,
        };

        // Skip if no code
        let code = match message
            .get("code")
            .and_then(|c| c.get("code"))
            .and_then(|c| c.as_str())
        {
            Some(c) => c,
            None => continue,
        };

        let level = message
            .get("level")
            .and_then(|l| l.as_str())
            .unwrap_or("warning");

        let msg_text = message
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("");

        // Extract first span
        let spans = message.get("spans").and_then(|s| s.as_array());
        let (file, line_num, col_num) = if let Some(spans) = spans {
            if let Some(first) = spans.first() {
                let f = first
                    .get("file_name")
                    .and_then(|f| f.as_str())
                    .map(std::path::PathBuf::from);
                let l = first
                    .get("line_start")
                    .and_then(|l| l.as_u64())
                    .map(|l| l as u32);
                let c = first
                    .get("column_start")
                    .and_then(|c| c.as_u64())
                    .map(|c| c as u32);
                (f, l, c)
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        let severity = match level {
            "error" => "error",
            _ => "warning",
        };

        effects.push(LocatedEffect {
            file,
            line: line_num,
            col: col_num,
            message: msg_text.to_string(),
            sig: EffectSignature {
                check_kind: "clippy".to_string(),
                outcome: "fail".to_string(),
                severity: severity.to_string(),
                message_class: code.to_string(),
                line_offset_from_edit: None,
            },
        });
    }

    ParsedCheckOutput {
        check_kind: CheckKind::Clippy,
        exit_code,
        effects,
        raw_stdout: stdout.to_string(),
        raw_stderr: stderr.to_string(),
    }
}

/// Parse cargo test output.
fn parse_test_output(stdout: &str, stderr: &str, exit_code: i32) -> ParsedCheckOutput {
    let mut effects = Vec::new();

    // Try JSON format first (each line might be JSON)
    let mut found_json = false;
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('{') {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
            if value.get("type").and_then(|t| t.as_str()) == Some("test")
                && value.get("event").and_then(|e| e.as_str()) == Some("failed")
            {
                let name = value
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown");
                effects.push(LocatedEffect {
                    file: None,
                    line: None,
                    col: None,
                    message: format!("test {name} FAILED"),
                    sig: EffectSignature {
                        check_kind: "test".to_string(),
                        outcome: "fail".to_string(),
                        severity: "test_fail".to_string(),
                        message_class: name.to_string(),
                        line_offset_from_edit: None,
                    },
                });
                found_json = true;
            }
        }
    }

    // Fallback: parse text output for "test <name> ... FAILED"
    if !found_json {
        let re = regex::Regex::new(r"test\s+(\S+)\s+\.\.\.\s+FAILED").ok();
        for line in stdout.lines().chain(stderr.lines()) {
            if let Some(re) = &re {
                if let Some(caps) = re.captures(line) {
                    let name = caps.get(1).map(|m| m.as_str()).unwrap_or("unknown");
                    effects.push(LocatedEffect {
                        file: None,
                        line: None,
                        col: None,
                        message: line.to_string(),
                        sig: EffectSignature {
                            check_kind: "test".to_string(),
                            outcome: "fail".to_string(),
                            severity: "test_fail".to_string(),
                            message_class: name.to_string(),
                            line_offset_from_edit: None,
                        },
                    });
                }
            }
        }
    }

    ParsedCheckOutput {
        check_kind: CheckKind::Test,
        exit_code,
        effects,
        raw_stdout: stdout.to_string(),
        raw_stderr: stderr.to_string(),
    }
}
