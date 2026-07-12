use serde_json::Value;
use std::collections::BTreeSet;

/// A typed anchor extracted by a reducer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub struct ReducerAnchor {
    pub kind: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_span: Option<(usize, usize)>,
}

/// Output of a typed reducer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub struct ReducerOutput {
    pub compacted: String,
    pub anchors: Vec<ReducerAnchor>,
    pub was_truncated: bool,
    pub approx_tokens: usize,
    pub loss_flags: Vec<String>,
}

fn approx_tokens_for_text(text: &str) -> usize {
    text.split_whitespace().count().max(1)
}

fn push_anchor(
    anchors: &mut Vec<ReducerAnchor>,
    kind: &str,
    value: &str,
    start: usize,
    end: usize,
) {
    if !value.trim().is_empty() {
        anchors.push(ReducerAnchor {
            kind: kind.to_string(),
            value: value.trim().to_string(),
            source_span: Some((start, end)),
        });
    }
}

/// Reduce JSON content to keys, known-path values, and errors.
pub fn reduce_json(content: &str, max_chars: usize) -> ReducerOutput {
    let mut anchors = Vec::new();
    let value: Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => {
            return ReducerOutput {
                compacted: content.chars().take(max_chars).collect::<String>(),
                anchors: vec![],
                was_truncated: content.len() > max_chars,
                approx_tokens: approx_tokens_for_text(content),
                loss_flags: vec!["invalid_json".to_string()],
            };
        }
    };
    let mut lines: Vec<String> = Vec::new();
    if let Some(obj) = value.as_object() {
        let keys: Vec<String> = obj.keys().cloned().collect();
        lines.push(format!("json:keys={}", keys.join(",")));
        for key in keys.iter().take(20) {
            if let Some(v) = obj.get(key) {
                let text = match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    Value::Array(arr) => format!("[len={}]", arr.len()),
                    Value::Object(map) => format!(
                        "{{keys={}}}",
                        map.keys().cloned().collect::<Vec<_>>().join(",")
                    ),
                };
                let entry = format!("{}={}", key, text.chars().take(80).collect::<String>());
                push_anchor(
                    &mut anchors,
                    "json_field",
                    &entry,
                    0,
                    content.len().min(max_chars),
                );
                lines.push(entry);
            }
        }
    } else if let Some(arr) = value.as_array() {
        lines.push(format!("json:array_len={}", arr.len()));
        for (idx, item) in arr.iter().take(8).enumerate() {
            let snippet = item.to_string().chars().take(60).collect::<String>();
            lines.push(format!("[{idx}]={snippet}"));
        }
    }
    // error detection
    let lower = content.to_lowercase();
    if lower.contains("error") || lower.contains("err") || lower.contains("message") {
        if let Some(msg) = value
            .get("error")
            .or_else(|| value.get("message"))
            .or_else(|| value.get("detail"))
        {
            let err_text = msg.to_string();
            lines.push(format!(
                "error={}",
                err_text.chars().take(120).collect::<String>()
            ));
            push_anchor(
                &mut anchors,
                "json_error",
                &err_text,
                0,
                content.len().min(max_chars),
            );
        }
    }
    let compacted = lines.join("\n");
    let was_truncated = compacted.len() > max_chars;
    let compacted = compacted.chars().take(max_chars).collect::<String>();
    ReducerOutput {
        approx_tokens: approx_tokens_for_text(&compacted),
        compacted,
        anchors,
        was_truncated,
        loss_flags: if was_truncated {
            vec!["truncated".to_string()]
        } else {
            vec![]
        },
    }
}

/// Reduce unified diff to file paths, hunks, and salient lines.
pub fn reduce_diff(content: &str, max_chars: usize) -> ReducerOutput {
    let mut anchors = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    let mut files = BTreeSet::new();
    let mut hunk_count = 0usize;
    let mut added = 0usize;
    let mut removed = 0usize;
    let mut salient_added: Vec<String> = Vec::new();
    let mut salient_removed: Vec<String> = Vec::new();
    for raw_line in content.lines() {
        let line = raw_line.trim_end();
        if line.starts_with("diff --git") {
            if let Some(part) = line.split_whitespace().nth(2) {
                files.insert(part.to_string());
            }
        } else if line.starts_with("--- a/") || line.starts_with("+++ b/") {
            let path = line
                .trim_start_matches("--- a/")
                .trim_start_matches("+++ b/")
                .trim();
            if !path.is_empty() && path != "dev/null" {
                files.insert(path.to_string());
            }
        } else if line.starts_with("@@") {
            hunk_count += 1;
        } else if line.starts_with('+') && !line.starts_with("+++") {
            added += 1;
            let body = line.chars().skip(1).collect::<String>();
            if !body.trim().is_empty() && salient_added.len() < 12 {
                salient_added.push(body.trim().to_string());
                push_anchor(
                    &mut anchors,
                    "added_line",
                    &body,
                    0,
                    content.len().min(max_chars),
                );
            }
        } else if line.starts_with('-') && !line.starts_with("---") {
            removed += 1;
            let body = line.chars().skip(1).collect::<String>();
            if !body.trim().is_empty() && salient_removed.len() < 12 {
                salient_removed.push(body.trim().to_string());
                push_anchor(
                    &mut anchors,
                    "removed_line",
                    &body,
                    0,
                    content.len().min(max_chars),
                );
            }
        }
    }
    lines.push(format!(
        "diff:files={}",
        files.into_iter().collect::<Vec<_>>().join(",")
    ));
    lines.push(format!("diff:hunks={} +{} -{}", hunk_count, added, removed));
    for line in salient_added.iter().take(8) {
        lines.push(format!("+{}", line));
    }
    for line in salient_removed.iter().take(4) {
        lines.push(format!("-{}", line));
    }
    let compacted = lines.join("\n");
    let was_truncated = compacted.len() > max_chars;
    let compacted = compacted.chars().take(max_chars).collect::<String>();
    let mut loss_flags = Vec::new();
    if was_truncated {
        loss_flags.push("truncated".to_string());
    }
    if added > salient_added.len() || removed > salient_removed.len() {
        loss_flags.push("non_salient_lines_omitted".to_string());
    }
    ReducerOutput {
        approx_tokens: approx_tokens_for_text(&compacted),
        compacted,
        anchors,
        was_truncated,
        loss_flags,
    }
}

/// Reduce compiler/test output to errors, warnings, test counts, and file spans.
pub fn reduce_compiler_output(content: &str, max_chars: usize) -> ReducerOutput {
    let mut anchors = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut file_spans: BTreeSet<String> = BTreeSet::new();
    let mut test_passed = 0usize;
    let mut test_failed = 0usize;
    let mut command: Option<String> = None;
    let mut exit_code: Option<i32> = None;
    for raw_line in content.lines() {
        let line = raw_line.trim_end();
        if line.starts_with('$') || line.starts_with("> ") {
            command = Some(line.to_string());
        }
        if let Some(idx) = line.find("exit:") {
            let tail = line[idx + 5..].trim();
            if let Ok(code) = tail.split_whitespace().next().unwrap_or("").parse::<i32>() {
                exit_code = Some(code);
            }
        }
        if line.starts_with("error[") || line.starts_with("error: ") {
            errors.push(line.to_string());
            push_anchor(&mut anchors, "error", line, 0, content.len().min(max_chars));
        }
        if line.starts_with("warning[") || line.starts_with("warning: ") {
            warnings.push(line.to_string());
            push_anchor(
                &mut anchors,
                "warning",
                line,
                0,
                content.len().min(max_chars),
            );
        }
        if line.contains("test result:") {
            // parse "test result: ok. N passed; M failed" or "test result: FAILED. N passed; M failed"
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, token) in parts.iter().enumerate() {
                if *token == "passed"
                    || token.ends_with("passed;")
                    || token.ends_with("passed,")
                    || token.ends_with("passed.")
                {
                    if let Some(prev) = parts.get(i.saturating_sub(1)) {
                        if let Ok(n) = prev
                            .trim_end_matches(|c: char| !c.is_ascii_digit())
                            .parse::<usize>()
                        {
                            test_passed = n;
                        }
                    }
                }
                if *token == "failed"
                    || token.ends_with("failed;")
                    || token.ends_with("failed,")
                    || token.ends_with("failed.")
                {
                    if let Some(prev) = parts.get(i.saturating_sub(1)) {
                        if let Ok(n) = prev
                            .trim_end_matches(|c: char| !c.is_ascii_digit())
                            .parse::<usize>()
                        {
                            test_failed = n;
                        }
                    }
                }
            }
        }
        // file:line:col references
        for cap in file_span_regex(line) {
            file_spans.insert(cap);
        }
    }
    if let Some(cmd) = command {
        lines.push(format!("cmd={}", cmd));
    }
    if let Some(code) = exit_code {
        lines.push(format!("exit={}", code));
    }
    if !errors.is_empty() {
        lines.push(format!("errors={}", errors.len()));
        for err in errors.iter().take(6) {
            lines.push(err.to_string());
        }
    }
    if !warnings.is_empty() {
        lines.push(format!("warnings={}", warnings.len()));
        for warn in warnings.iter().take(4) {
            lines.push(warn.to_string());
        }
    }
    if test_passed + test_failed > 0 {
        lines.push(format!(
            "tests={} passed, {} failed",
            test_passed, test_failed
        ));
    }
    if !file_spans.is_empty() {
        lines.push(format!(
            "spans={}",
            file_spans.iter().cloned().collect::<Vec<_>>().join(",")
        ));
        for span in file_spans.iter().take(8) {
            push_anchor(
                &mut anchors,
                "file_span",
                span,
                0,
                content.len().min(max_chars),
            );
        }
    }
    let compacted = lines.join("\n");
    let was_truncated = compacted.len() > max_chars;
    let compacted = compacted.chars().take(max_chars).collect::<String>();
    let mut loss_flags = Vec::new();
    if was_truncated {
        loss_flags.push("truncated".to_string());
    }
    if errors.len() > 6 {
        loss_flags.push("additional_errors_omitted".to_string());
    }
    ReducerOutput {
        approx_tokens: approx_tokens_for_text(&compacted),
        compacted,
        anchors,
        was_truncated,
        loss_flags,
    }
}

fn file_span_regex(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    for token in line.split_whitespace() {
        if token.contains('.') && token.contains(':') {
            // crude path:line:col detection
            let parts: Vec<&str> = token.split(':').collect();
            if parts.len() >= 2 {
                let path = parts[0];
                if path.contains('/') && parts[1].chars().all(|c| c.is_ascii_digit()) {
                    out.push(token.to_string());
                }
            }
        }
    }
    out
}

/// Reduce shell log to command, exit status, and stderr tail.
pub fn reduce_shell_log(content: &str, max_chars: usize) -> ReducerOutput {
    let mut anchors = Vec::new();
    let mut command: Option<String> = None;
    let mut exit_code: Option<i32> = None;
    let mut error_lines: Vec<String> = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with('$') || trimmed.starts_with("> ") || trimmed.starts_with("# ") {
            command = Some(trimmed.to_string());
        }
        if trimmed.to_lowercase().contains("exit code")
            || trimmed.to_lowercase().starts_with("exit:")
            || trimmed.to_lowercase().starts_with("exit status")
        {
            if let Some(num) = trimmed
                .split_whitespace()
                .find_map(|t| t.parse::<i32>().ok())
            {
                exit_code = Some(num);
            }
        }
        if trimmed.to_lowercase().contains("error")
            || trimmed.to_lowercase().contains("failed")
            || trimmed.to_lowercase().starts_with("traceback")
        {
            error_lines.push(trimmed.to_string());
        }
    }
    let stderr_tail: Vec<String> = error_lines.into_iter().rev().take(10).rev().collect();
    let mut parts: Vec<String> = Vec::new();
    if let Some(cmd) = command {
        parts.push(format!("cmd={}", cmd));
        push_anchor(
            &mut anchors,
            "command",
            &cmd,
            0,
            content.len().min(max_chars),
        );
    }
    if let Some(code) = exit_code {
        parts.push(format!("exit={}", code));
    }
    for line in &stderr_tail {
        parts.push(line.clone());
        push_anchor(
            &mut anchors,
            "stderr_line",
            line,
            0,
            content.len().min(max_chars),
        );
    }
    let compacted = parts.join("\n");
    let was_truncated = compacted.len() > max_chars;
    let compacted = compacted.chars().take(max_chars).collect::<String>();
    ReducerOutput {
        approx_tokens: approx_tokens_for_text(&compacted),
        compacted,
        anchors,
        was_truncated,
        loss_flags: if was_truncated {
            vec!["truncated".to_string()]
        } else {
            vec![]
        },
    }
}

/// Reduce search/grep results to path, line, matched text.
pub fn reduce_search_results(content: &str, max_chars: usize) -> ReducerOutput {
    let mut anchors = Vec::new();
    let mut entries: Vec<String> = Vec::new();
    for line in content.lines().take(200) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.splitn(3, ':').collect();
        if parts.len() >= 2 && parts[0].contains('/') {
            let path = parts[0];
            let line_no = parts[1];
            let matched = parts.get(2).unwrap_or(&"").trim();
            let entry = format!("{path}:{line_no}:{matched}");
            entries.push(entry.chars().take(140).collect::<String>());
            push_anchor(
                &mut anchors,
                "search_hit",
                matched,
                0,
                content.len().min(max_chars),
            );
        } else {
            entries.push(trimmed.chars().take(140).collect::<String>());
        }
    }
    let header = format!("search:hits={}", entries.len());
    let compacted = std::iter::once(header)
        .chain(entries.into_iter().take(20))
        .collect::<Vec<_>>()
        .join("\n");
    let was_truncated = compacted.len() > max_chars;
    let compacted = compacted.chars().take(max_chars).collect::<String>();
    let mut loss_flags = Vec::new();
    if was_truncated {
        loss_flags.push("truncated".to_string());
    }
    ReducerOutput {
        approx_tokens: approx_tokens_for_text(&compacted),
        compacted,
        anchors,
        was_truncated,
        loss_flags,
    }
}

/// Reduce Markdown to objectives, requirements, decisions, tasks, headings.
pub fn reduce_markdown(content: &str, max_chars: usize) -> ReducerOutput {
    let mut anchors = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    let mut headings: Vec<String> = Vec::new();
    let mut decisions: Vec<String> = Vec::new();
    let mut gates: Vec<String> = Vec::new();
    let mut questions: Vec<String> = Vec::new();
    let mut tasks: Vec<String> = Vec::new();
    let mut requirements: Vec<String> = Vec::new();
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.starts_with("# ") || line.starts_with("## ") || line.starts_with("### ") {
            headings.push(line.to_string());
            push_anchor(
                &mut anchors,
                "heading",
                line,
                0,
                content.len().min(max_chars),
            );
        } else if line.to_lowercase().starts_with("decision:")
            || line.to_lowercase().starts_with("decided:")
        {
            decisions.push(line.to_string());
            push_anchor(
                &mut anchors,
                "decision",
                line,
                0,
                content.len().min(max_chars),
            );
        } else if line.to_lowercase().contains("acceptance gate:")
            || line.to_lowercase().contains("must pass")
            || line.to_lowercase().contains("must remain")
        {
            gates.push(line.to_string());
            push_anchor(
                &mut anchors,
                "acceptance_gate",
                line,
                0,
                content.len().min(max_chars),
            );
        } else if line.to_lowercase().starts_with("unresolved question") || line.contains('?') {
            questions.push(line.to_string());
        } else if line.starts_with("- [ ]") || line.starts_with("- [x]") {
            tasks.push(line.to_string());
            push_anchor(&mut anchors, "task", line, 0, content.len().min(max_chars));
        } else if line.to_lowercase().contains("must")
            || line.to_lowercase().contains("shall")
            || line.to_lowercase().contains("required")
        {
            requirements.push(line.to_string());
        }
    }
    lines.push(format!("md:headings={}", headings.len()));
    lines.extend(headings.into_iter().take(8));
    if !decisions.is_empty() {
        lines.push(format!("md:decisions={}", decisions.len()));
        lines.extend(decisions.into_iter().take(6));
    }
    if !gates.is_empty() {
        lines.push(format!("md:gates={}", gates.len()));
        lines.extend(gates.into_iter().take(6));
    }
    if !tasks.is_empty() {
        lines.push(format!("md:tasks={}", tasks.len()));
        lines.extend(tasks.into_iter().take(10));
    }
    if !questions.is_empty() {
        lines.push(format!("md:questions={}", questions.len()));
        lines.extend(questions.into_iter().take(6));
    }
    if !requirements.is_empty() {
        lines.push(format!("md:requirements={}", requirements.len()));
        lines.extend(requirements.into_iter().take(6));
    }
    let compacted = lines.join("\n");
    let was_truncated = compacted.len() > max_chars;
    let compacted = compacted.chars().take(max_chars).collect::<String>();
    ReducerOutput {
        approx_tokens: approx_tokens_for_text(&compacted),
        compacted,
        anchors,
        was_truncated,
        loss_flags: if was_truncated {
            vec!["truncated".to_string()]
        } else {
            vec![]
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_reducer_extracts_keys_and_errors() {
        let json =
            r#"{"status":"ok","data":{"items":[1,2,3]},"error":"none","path":"/src/lib.rs"}"#;
        let out = reduce_json(json, 200);
        assert!(out.compacted.contains("status"));
        assert!(out.compacted.contains("error"));
        assert!(!out.was_truncated);
    }

    #[test]
    fn diff_reducer_extracts_file_paths_and_hunks() {
        let diff = "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -10,3 +10,4 @@\n+fn new_function() {}\n";
        let out = reduce_diff(diff, 200);
        assert!(out.compacted.contains("src/lib.rs"));
        assert!(out.compacted.contains("new_function"));
    }

    #[test]
    fn compiler_reducer_extracts_errors_and_test_counts() {
        let output = "cargo test --all-targets\nerror[E0425]: cannot find value `x`\ntest result: FAILED. 3 passed; 2 failed\n";
        let out = reduce_compiler_output(output, 300);
        println!("COMPACTED:\n{}", out.compacted);
        assert!(out.compacted.contains("E0425"), "missing E0425");
        assert!(
            out.compacted.contains("tests=3 passed, 2 failed"),
            "expected test count line"
        );
    }

    #[test]
    fn markdown_reducer_extracts_headings_and_decisions() {
        let md = "# Plan\n\nDecision: use JSON parsing\n\nAcceptance gate: cargo test must pass\n";
        let out = reduce_markdown(md, 300);
        assert!(out.compacted.contains("Plan"));
        assert!(out.compacted.contains("Decision"));
        assert!(out.compacted.contains("Acceptance"));
    }

    #[test]
    fn search_reducer_extracts_paths_and_lines() {
        let results = "src/lib.rs:42:fn main() {}\nsrc/main.rs:10:use lib;\n";
        let out = reduce_search_results(results, 300);
        assert!(out.compacted.contains("src/lib.rs:42"));
        assert!(out.compacted.contains("fn main"));
    }

    #[test]
    fn shell_reducer_extracts_command_and_status() {
        let log = "$ cargo build\nCompiling crate v0.1\nFinished dev profile\nexit: 0\n";
        let out = reduce_shell_log(log, 200);
        assert!(out.compacted.contains("cargo build"));
        assert!(out.compacted.contains("exit=0"));
    }
}
