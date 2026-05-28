//! Repository, patch, command, sandbox, and packet artifacts.
//!
//! Tool outputs are local operator receipts; repository contents and package truth remain separately verified.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoReadReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub tool_id: String,
    pub sandbox_root: String,
    pub requested_path: String,
    pub resolved_path: String,
    pub bytes: u64,
    pub content_digest: DisplayDigestV1,
    pub allowed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub read_at: DateTime<Utc>,
}

impl RepoReadReportV1 {
    pub fn allowed(
        sandbox_root: impl Into<String>,
        requested_path: impl Into<String>,
        resolved_path: impl Into<String>,
        bytes: u64,
        content: &str,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("repo-read"),
            kind: ArtifactKindV1::RepoRead,
            tool_id: "aidens:repo-read:1".into(),
            sandbox_root: sandbox_root.into(),
            requested_path: requested_path.into(),
            resolved_path: resolved_path.into(),
            bytes,
            content_digest: DisplayDigestV1::for_text(content),
            allowed: true,
            reason_codes: vec!["sandbox-read-allowed".into()],
            read_at: Utc::now(),
        }
    }

    pub fn denied(
        sandbox_root: impl Into<String>,
        requested_path: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        let reason = reason.into();
        Self {
            receipt_id: display_only_unstable_id("repo-read"),
            kind: ArtifactKindV1::RepoRead,
            tool_id: "aidens:repo-read:1".into(),
            sandbox_root: sandbox_root.into(),
            requested_path: requested_path.into(),
            resolved_path: String::new(),
            bytes: 0,
            content_digest: DisplayDigestV1::for_text(""),
            allowed: false,
            reason_codes: vec![reason],
            read_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoListEntryV1 {
    pub path: String,
    pub entry_kind: String,
    pub bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoListReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub tool_id: String,
    pub sandbox_root: String,
    pub requested_path: String,
    pub entries: Vec<RepoListEntryV1>,
    pub listing_digest: DisplayDigestV1,
    #[serde(default)]
    pub total_entries: usize,
    #[serde(default)]
    pub returned_entries: usize,
    #[serde(default)]
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_listing_digest: Option<DisplayDigestV1>,
    pub allowed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub listed_at: DateTime<Utc>,
}

impl RepoListReportV1 {
    pub fn allowed(
        sandbox_root: impl Into<String>,
        requested_path: impl Into<String>,
        entries: Vec<RepoListEntryV1>,
    ) -> Self {
        Self::allowed_with_full_listing(
            sandbox_root,
            requested_path,
            entries.clone(),
            entries.len(),
            DisplayDigestV1::for_json_value(
                &serde_json::to_value(&entries).unwrap_or(serde_json::Value::Null),
            ),
        )
    }

    pub fn allowed_with_full_listing(
        sandbox_root: impl Into<String>,
        requested_path: impl Into<String>,
        entries: Vec<RepoListEntryV1>,
        total_entries: usize,
        full_listing_digest: DisplayDigestV1,
    ) -> Self {
        let listing = serde_json::to_value(&entries).unwrap_or(serde_json::Value::Null);
        let returned_entries = entries.len();
        let truncated = returned_entries < total_entries;
        Self {
            receipt_id: display_only_unstable_id("repo-list"),
            kind: ArtifactKindV1::RepoList,
            tool_id: "aidens:repo-list:1".into(),
            sandbox_root: sandbox_root.into(),
            requested_path: requested_path.into(),
            entries,
            listing_digest: DisplayDigestV1::for_json_value(&listing),
            total_entries,
            returned_entries,
            truncated,
            full_listing_digest: Some(full_listing_digest),
            allowed: true,
            reason_codes: if truncated {
                vec![
                    "sandbox-list-allowed".into(),
                    "repo-list-truncated-with-full-digest".into(),
                ]
            } else {
                vec!["sandbox-list-allowed".into()]
            },
            listed_at: Utc::now(),
        }
    }

    pub fn denied(
        sandbox_root: impl Into<String>,
        requested_path: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("repo-list"),
            kind: ArtifactKindV1::RepoList,
            tool_id: "aidens:repo-list:1".into(),
            sandbox_root: sandbox_root.into(),
            requested_path: requested_path.into(),
            entries: Vec::new(),
            listing_digest: DisplayDigestV1::for_json_value(&serde_json::json!([])),
            total_entries: 0,
            returned_entries: 0,
            truncated: false,
            full_listing_digest: Some(DisplayDigestV1::for_json_value(&serde_json::json!([]))),
            allowed: false,
            reason_codes: vec![reason.into()],
            listed_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PatchProposalV1 {
    pub proposal_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub summary: String,
    pub unified_diff: String,
    pub touched_paths: Vec<String>,
    pub diff_digest: DisplayDigestV1,
    pub mutates_files: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub proposed_at: DateTime<Utc>,
}

impl PatchProposalV1 {
    pub fn new(
        summary: impl Into<String>,
        unified_diff: impl Into<String>,
        touched_paths: Vec<String>,
    ) -> Self {
        let unified_diff = unified_diff.into();
        Self {
            proposal_id: display_only_unstable_id("patch-proposal"),
            kind: ArtifactKindV1::PatchProposal,
            summary: summary.into(),
            diff_digest: DisplayDigestV1::for_text(&unified_diff),
            unified_diff,
            touched_paths: sorted_unique_strings(touched_paths),
            mutates_files: false,
            reason_codes: vec!["proposal-only-no-file-mutation".into()],
            proposed_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PatchApplyReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub tool_id: String,
    pub sandbox_root: String,
    pub proposal_id: Option<ArtifactId>,
    pub permit_grant_id: Option<ArtifactId>,
    pub permit_use_receipt_id: Option<ArtifactId>,
    pub touched_paths: Vec<String>,
    pub input_digest: DisplayDigestV1,
    pub before_digests: BTreeMap<String, DisplayDigestV1>,
    pub after_digests: BTreeMap<String, DisplayDigestV1>,
    pub applied: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_files: Vec<String>,
    #[serde(default)]
    pub dry_run_checked: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub semantic_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rollback_advice: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub applied_at: DateTime<Utc>,
}

impl PatchApplyReportV1 {
    pub fn new(
        sandbox_root: impl Into<String>,
        input: &serde_json::Value,
        touched_paths: Vec<String>,
        before_digests: BTreeMap<String, DisplayDigestV1>,
        after_digests: BTreeMap<String, DisplayDigestV1>,
        permit_grant_id: Option<ArtifactId>,
        permit_use_receipt_id: Option<ArtifactId>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("patch-apply"),
            kind: ArtifactKindV1::PatchApply,
            tool_id: "aidens:patch-apply:1".into(),
            sandbox_root: sandbox_root.into(),
            proposal_id: None,
            permit_grant_id,
            permit_use_receipt_id,
            touched_paths: sorted_unique_strings(touched_paths.clone()),
            input_digest: DisplayDigestV1::for_json_value(input),
            before_digests,
            after_digests,
            applied: true,
            changed_files: sorted_unique_strings(touched_paths),
            dry_run_checked: true,
            semantic_status: "exact_check".into(),
            failure_kind: None,
            rollback_advice: vec![
                "Use before_digests to verify the pre-apply state before manual rollback.".into(),
                "Reconstruct prior file contents from version control or the original sandbox fixture.".into(),
            ],
            reason_codes: vec!["patch-applied-with-explicit-permit".into()],
            applied_at: Utc::now(),
        }
    }

    pub fn checked(
        sandbox_root: impl Into<String>,
        input: &serde_json::Value,
        touched_paths: Vec<String>,
        before_digests: BTreeMap<String, DisplayDigestV1>,
        after_digests: BTreeMap<String, DisplayDigestV1>,
        permit_grant_id: Option<ArtifactId>,
        permit_use_receipt_id: Option<ArtifactId>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("patch-apply"),
            kind: ArtifactKindV1::PatchApply,
            tool_id: "aidens:patch-apply:1".into(),
            sandbox_root: sandbox_root.into(),
            proposal_id: None,
            permit_grant_id,
            permit_use_receipt_id,
            touched_paths: sorted_unique_strings(touched_paths.clone()),
            input_digest: DisplayDigestV1::for_json_value(input),
            before_digests,
            after_digests,
            applied: false,
            changed_files: sorted_unique_strings(touched_paths),
            dry_run_checked: true,
            semantic_status: "exact_check".into(),
            failure_kind: None,
            rollback_advice: vec![
                "No rollback required; check_only mode did not mutate files.".into(),
            ],
            reason_codes: vec!["patch-validated-without-application".into()],
            applied_at: Utc::now(),
        }
    }

    pub fn denied(
        sandbox_root: impl Into<String>,
        input: &serde_json::Value,
        reason: impl Into<String>,
    ) -> Self {
        Self::denied_with_details(
            sandbox_root,
            input,
            reason,
            "invalid-patch",
            Vec::new(),
            None,
            None,
        )
    }

    pub fn denied_with_details(
        sandbox_root: impl Into<String>,
        input: &serde_json::Value,
        reason: impl Into<String>,
        failure_kind: impl Into<String>,
        touched_paths: Vec<String>,
        permit_grant_id: Option<ArtifactId>,
        permit_use_receipt_id: Option<ArtifactId>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("patch-apply"),
            kind: ArtifactKindV1::PatchApply,
            tool_id: "aidens:patch-apply:1".into(),
            sandbox_root: sandbox_root.into(),
            proposal_id: None,
            permit_grant_id,
            permit_use_receipt_id,
            touched_paths: sorted_unique_strings(touched_paths.clone()),
            input_digest: DisplayDigestV1::for_json_value(input),
            before_digests: BTreeMap::new(),
            after_digests: BTreeMap::new(),
            applied: false,
            changed_files: sorted_unique_strings(touched_paths),
            dry_run_checked: true,
            semantic_status: "failed_exact_check".into(),
            failure_kind: Some(failure_kind.into()),
            rollback_advice: vec![
                "No files were written by this failed-closed patch attempt.".into(),
                "Regenerate a single-file unified diff with unique removal context before retrying.".into(),
            ],
            reason_codes: vec![reason.into()],
            applied_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CommandRunReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub tool_id: String,
    pub sandbox_root: String,
    pub command: Vec<String>,
    pub permit_grant_id: Option<ArtifactId>,
    pub permit_use_receipt_id: Option<ArtifactId>,
    pub allowed_by_policy: bool,
    pub exit_code: Option<i32>,
    pub stdout_digest: DisplayDigestV1,
    pub stderr_digest: DisplayDigestV1,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub timed_out: bool,
    pub succeeded: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

impl CommandRunReportV1 {
    pub fn completed(
        sandbox_root: impl Into<String>,
        command: Vec<String>,
        permit_grant_id: Option<ArtifactId>,
        permit_use_receipt_id: Option<ArtifactId>,
        exit_code: Option<i32>,
        stdout: &str,
        stderr: &str,
    ) -> Self {
        let succeeded = exit_code == Some(0);
        Self {
            receipt_id: display_only_unstable_id("command-run"),
            kind: ArtifactKindV1::CommandRun,
            tool_id: "aidens:run-checks:1".into(),
            sandbox_root: sandbox_root.into(),
            command,
            permit_grant_id,
            permit_use_receipt_id,
            allowed_by_policy: true,
            exit_code,
            stdout_digest: DisplayDigestV1::for_text(stdout),
            stderr_digest: DisplayDigestV1::for_text(stderr),
            stdout_bytes: stdout.len(),
            stderr_bytes: stderr.len(),
            timed_out: false,
            succeeded,
            reason_codes: if succeeded {
                vec!["allowed-check-command-succeeded".into()]
            } else {
                vec!["allowed-check-command-failed".into()]
            },
            started_at: Utc::now(),
            completed_at: Utc::now(),
        }
    }

    pub fn blocked(
        sandbox_root: impl Into<String>,
        command: Vec<String>,
        reason: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            receipt_id: display_only_unstable_id("command-run"),
            kind: ArtifactKindV1::CommandRun,
            tool_id: "aidens:run-checks:1".into(),
            sandbox_root: sandbox_root.into(),
            command,
            permit_grant_id: None,
            permit_use_receipt_id: None,
            allowed_by_policy: false,
            exit_code: None,
            stdout_digest: DisplayDigestV1::for_text(""),
            stderr_digest: DisplayDigestV1::for_text(""),
            stdout_bytes: 0,
            stderr_bytes: 0,
            timed_out: false,
            succeeded: false,
            reason_codes: vec![reason.into()],
            started_at: now,
            completed_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SandboxCapabilityTruthV1 {
    pub truth_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub sandbox_root: String,
    pub denied_prefixes: Vec<String>,
    pub symlink_policy: String,
    pub env_allowlist: Vec<String>,
    pub network_policy: String,
    pub process_timeout_millis: u64,
    pub write_requires_permit: bool,
    pub shell_requires_permit: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub checked_at: DateTime<Utc>,
}

impl SandboxCapabilityTruthV1 {
    pub fn coding_default(sandbox_root: impl Into<String>) -> Self {
        Self {
            truth_id: display_only_unstable_id("sandbox-truth"),
            kind: ArtifactKindV1::SandboxTruth,
            sandbox_root: sandbox_root.into(),
            denied_prefixes: vec![
                ".ssh".into(),
                ".gnupg".into(),
                ".cargo".into(),
                ".recall".into(),
                ".password-store".into(),
            ],
            symlink_policy: "canonicalize-and-require-within-root".into(),
            env_allowlist: vec!["PATH".into(), "CARGO_HOME".into(), "RUSTUP_HOME".into()],
            network_policy: "disabled".into(),
            process_timeout_millis: 120_000,
            write_requires_permit: true,
            shell_requires_permit: true,
            reason_codes: vec![
                "no-shell-file-write-or-network-by-default".into(),
                "side-effects-require-scoped-permit".into(),
            ],
            checked_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CodexPacketV1 {
    pub packet_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub current_pass: String,
    pub next_pass: String,
    pub source_basis: String,
    pub issue: String,
    pub source_map: Vec<String>,
    pub changed_files: Vec<String>,
    pub commands_run: Vec<CommandRunReportV1>,
    pub receipt_ids: Vec<ArtifactId>,
    pub blockers: Vec<String>,
    pub notes: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexPacketInputV1 {
    pub current_pass: String,
    pub next_pass: String,
    pub issue: String,
    pub source_map: Vec<String>,
    pub changed_files: Vec<String>,
    pub commands_run: Vec<CommandRunReportV1>,
    pub receipt_ids: Vec<ArtifactId>,
    pub blockers: Vec<String>,
    pub notes: Vec<String>,
}

impl CodexPacketV1 {
    pub fn new(input: CodexPacketInputV1) -> Self {
        Self {
            packet_id: display_only_unstable_id("codex-packet"),
            kind: ArtifactKindV1::CodexPacket,
            current_pass: input.current_pass,
            next_pass: input.next_pass,
            source_basis: "SOURCE_BASIS.md".into(),
            issue: input.issue,
            source_map: sorted_unique_strings(input.source_map),
            changed_files: sorted_unique_strings(input.changed_files),
            commands_run: input.commands_run,
            receipt_ids: input.receipt_ids,
            blockers: input.blockers,
            notes: input.notes,
            generated_at: Utc::now(),
        }
    }

    pub fn has_resume_context(&self) -> bool {
        !self.current_pass.trim().is_empty()
            && !self.next_pass.trim().is_empty()
            && !self.source_basis.trim().is_empty()
            && !self.issue.trim().is_empty()
            && !self.source_map.is_empty()
    }
}
