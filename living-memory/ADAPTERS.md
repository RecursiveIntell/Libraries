# ADAPTERS.md
# ProjectAdapter: Rust/Cargo (v1)

## ProjectAdapter trait

```rust
pub trait ProjectAdapter: Send + Sync {
    fn detect(workspace: &Path) -> bool where Self: Sized;
    fn name(&self) -> &str;
    fn check_commands(&self, config: &ForgeConfig) -> Vec<CheckCommand>;
    fn parse_check_output(&self, cmd: &CheckCommand, stdout: &str, stderr: &str, exit_code: i32) -> ParsedCheckOutput;
}

pub struct CheckCommand {
    pub kind:    CheckKind,
    pub program: String,
    pub args:    Vec<String>,
    pub env:     Vec<(String, String)>,
}

pub enum CheckKind { Fmt, Clippy, Test }
```

---

## CargoAdapter

### Detection
```rust
fn detect(workspace: &Path) -> bool {
    workspace.join("Cargo.toml").exists()
}
```

### Commands (configurable; defaults below)

**Fmt check:**
```
cargo fmt --all -- --check
```
- Run before Clippy (formatting errors pollute clippy output)
- Parse: lines containing `Diff in <file>` → file with formatting issues

**Clippy lint:**
```
cargo clippy --all-targets --all-features --message-format=json -- -D warnings
```
- `--message-format=json` outputs machine-readable diagnostics on stderr
- Parse: JSON stream of `CompilerMessage` objects; extract `{ code.code, spans[0].file_name, spans[0].line_start }`

**Tests:**
```
cargo test --all --all-features
```
- Try with `-- --format=json` first; if it fails (older toolchain), retry without
- Parse text output for `test <name> ... FAILED` and `FAILED` summary lines

### Output parsing detail

**Clippy JSON format** (relevant fields per message):
```json
{
  "reason": "compiler-message",
  "message": {
    "code": { "code": "clippy::needless_return" },
    "level": "warning" | "error",
    "message": "...",
    "spans": [{ "file_name": "src/lib.rs", "line_start": 42, "column_start": 5 }]
  }
}
```
Ignore messages where `reason != "compiler-message"` or `message.code == null`.

**Test JSON format** (when available):
```json
{ "type": "test", "event": "failed", "name": "module::test_name" }
```

### CheckResult
```rust
pub struct CheckResult {
    pub fmt_pass:    bool,
    pub clippy_pass: bool,
    pub test_pass:   bool,
    pub fmt_output:    ParsedCheckOutput,
    pub clippy_output: ParsedCheckOutput,
    pub test_output:   ParsedCheckOutput,
    pub total_duration_ms: u64,
}

impl CheckResult {
    pub fn all_pass(&self) -> bool {
        self.fmt_pass && self.clippy_pass && self.test_pass
    }
}
```

### Forbidden path pre-check
If `config.allow_test_modifications == false` (default):
- Validate patch before running any commands.
- If patch touches forbidden paths → reject with `ValidationResult`; do NOT run commands.
- This saves time and prevents spurious CEA attributions from invalid patches.

---

## Future adapters (not in v1)
- `NodeAdapter` (npm test, eslint)
- `PythonAdapter` (pytest, ruff)
- `GoAdapter` (go test, go vet)

The `ProjectAdapter` trait is designed to accommodate these without changes to Runtime or Lab.
