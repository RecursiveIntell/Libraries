# P24 evidence and reporting requirements

## Required evidence artifacts

Each phase must emit:

- `AiDENs/handoffs/p24/PHASE_XX_REPORT.md`
- command transcript with cwd, command, exit code, duration, stdout/stderr summary
- changed-file list
- test list and status
- artifact list with SHA-256
- degraded/partial/scaffold/deferred support changes
- unresolved risk list

The final pass must emit:

- `AiDENs/handoffs/p24/P24_FINAL_AUDIT_REPORT.md`
- `AiDENs/handoffs/p24/P24_KNOWN_LIMITATIONS.md`
- `AiDENs/P24_STATUS_EVIDENCE_MANIFEST.json`
- `AiDENs/docs/p24/P24_CANONICAL_SEAM_MAP.md`
- `AiDENs/docs/p24/P24_CONTRACT_SURFACE_REPORT.md`
- package report/manifest/findings/excluded/codex-archive sidecars
- at least one run-bundle V2 artifact and replay receipt
- coding-agent lane evidence if promoted
- memory/runtime seam evidence if promoted
- daemon-safe lane evidence if promoted

## Command transcript minimum fields

```json
{
  "command_id": "string",
  "phase": "P24-XX",
  "cwd": "string",
  "argv": ["string"],
  "started_utc": "string",
  "ended_utc": "string",
  "duration_ms": 0,
  "exit_code": 0,
  "stdout_path": "string",
  "stderr_path": "string",
  "status": "pass|fail|timeout|skipped",
  "reason": "string"
}
```

## Run-bundle evidence minimum fields

`AiDENsRunBundleV2` must include:

- schema name and version
- support tier
- run id and content digest
- canonical `ExecutionContextV1` or backpointer
- `TraceCtx`, `AttemptId`, `TrialId`
- replay link and replay parent if any
- provider route and dispatch outcome
- tool route and receipt references
- queue hops if any
- budget/deadline/cancellation/degradation
- event log digest and normalized replay digest
- environment fingerprint
- final outcome and failure taxonomy

## Failure taxonomy

Use these failure classes:

- `unsupported_surface`
- `scaffold_surface`
- `missing_permit`
- `provider_unavailable`
- `tool_denied`
- `budget_exhausted`
- `deadline_exceeded`
- `parser_ambiguity`
- `repair_required`
- `verification_required`
- `canonical_owner_missing`
- `digest_mismatch`
- `backpointer_missing`
- `non_replayable`
- `timeout`

## Honesty rule

If a thing cannot be proven by a command, fixture, schema, or artifact, it may be mentioned only as deferred, partial, scaffold, or future work. No prose-only promotion.
