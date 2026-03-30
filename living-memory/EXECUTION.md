# EXECUTION.md
# Execution Backends

## ExecutionBackend trait

```rust
#[async_trait]
pub trait ExecutionBackend: Send + Sync {
    fn kind(&self) -> ExecutionBackendKind;
    async fn prepare_workspace(&self, fixture: &Path) -> Result<Workspace>;
    async fn apply_patch(&self, ws: &Workspace, patch: &StructuredPatch) -> Result<(PatchedWorkspace, LineAttributionMap)>;
    async fn render_diff(&self, original: &Workspace, patched: &PatchedWorkspace) -> Result<String>;
    async fn run_checks(&self, patched: &PatchedWorkspace, adapter: &dyn ProjectAdapter) -> Result<CheckResult>;
    async fn collect_logs(&self, result: &CheckResult) -> Result<LogBundle>;
}

pub enum ExecutionBackendKind { Host, Container }
```

---

## HostBackend

- Runs commands via `std::process::Command` with output capture.
- Workspace: temp directory created with `tempfile::TempDir`; fixture files copied in.
- Environment sanitization:
  - Inherit `PATH`, `HOME`, `RUSTUP_HOME`, `CARGO_HOME` from parent.
  - Strip any variables matching `SECRET`, `TOKEN`, `KEY`, `PASSWORD`, `ANTHROPIC`, `OPENAI`.
  - Set `CARGO_TERM_COLOR=never`, `RUST_BACKTRACE=0`.
- Timeouts: configurable per command; default 120 seconds. Kills process + children on timeout.
- Log capture: stdout and stderr captured to `LogBundle`; not streamed to stdout.

---

## ContainerBackend

### Runtime autodetect
Probe in order:
1. `docker version` → exit 0 → use Docker
2. `podman version` → exit 0 → use Podman
3. `nerdctl version` → exit 0 → use nerdctl
4. None found → `ContainerBackend` unavailable → auto-select falls through to `HostBackend`

Store detected runtime in `ContainerBackend` struct. Log which runtime was selected.

### Container lifecycle
```
prepare_workspace(fixture):
  copy fixture to host temp dir
  record temp dir path as workspace.host_path

apply_patch(ws, patch):
  apply patch to ws.host_path (same as HostBackend apply)
  return PatchedWorkspace { host_path: ws.host_path, ... }

run_checks(patched, adapter):
  for each command in adapter.check_commands():
    run:
      <runtime> run --rm \
        -v <patched.host_path>:/workspace:rw \
        -w /workspace \
        [--network=none if sealed] \
        [--memory=2g] \
        [--cpus=2.0] \
        <rust_image> \
        sh -c "<command>"
    capture stdout/stderr
    record exit code
  return CheckResult
```

### Container runtime command mapping

| Runtime  | Network flag    | Volume flag       | Remove flag |
|----------|-----------------|-------------------|-------------|
| docker   | `--network=none`| `-v src:dst`      | `--rm`      |
| podman   | `--network=none`| `-v src:dst`      | `--rm`      |
| nerdctl  | `--net=none`    | `-v src:dst`      | `--rm`      |

### Sealed mode enforcement
If `config.mode == sealed_local`:
- MUST include network isolation flag (see table above).
- If the runtime does not support it → return `Err(ForgeError::SealedModeUnsupported { runtime })`.
- Never silently proceed without network isolation in sealed mode.

---

## Output capture
All stdout and stderr per command stored in `LogBundle`:

```rust
pub struct LogBundle {
    pub fmt_stdout:     String,
    pub fmt_stderr:     String,
    pub clippy_stdout:  String,
    pub clippy_stderr:  String,
    pub test_stdout:    String,
    pub test_stderr:    String,
    pub timings:        CommandTimings,
}
```

`logs_ref` in `eval_runs`:
- Preferred: write `LogBundle` as JSON to `<forge_data_dir>/logs/<eval_id>.json`; store path.
- Fallback (small logs): `inline:<base64(json)>` — only if log < 64KB total.

---

## ParsedCheckOutput (for CEA)

After each check command completes, parse output into structured form:

```rust
pub struct ParsedCheckOutput {
    pub check_kind:  CheckKind,
    pub exit_code:   i32,
    pub effects:     Vec<LocatedEffect>,  // positioned effects for CEA attribution
    pub raw_stdout:  String,
    pub raw_stderr:  String,
}

pub struct LocatedEffect {
    pub file:    Option<PathBuf>,   // repo-relative if parseable
    pub line:    Option<u32>,
    pub col:     Option<u32>,
    pub message: String,
    pub sig:     EffectSignature,   // pre-computed for CEA
}
```

Parsing per check kind:
- `fmt`: parse list of changed files from stdout (lines like `Diff in src/lib.rs:`)
- `clippy`: run with `--message-format=json` on stderr; parse JSON diagnostic objects
- `test`: run with `-- --format=json` if supported; fallback parse text output for `FAILED`
