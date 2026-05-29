# Reference z.py change plan

This is not a blind patch. Codex must inspect current `z.py` and adapt exactly.

## Minimal direct hunks

```diff
@@
 ALLOWED_TEXT_EXTENSIONS = {
+    ".pyi",
@@
 ALLOWED_BASENAMES = {
+    "py.typed",
```

## Add context log policy

Add constants near `LOG_EXTENSIONS` or archive policy constants:

```python
CONTEXT_LOG_BASENAMES = {"commands_run.log", "validation.log", "stdout.log", "stderr.log"}
CONTEXT_LOG_PACKAGE_ROLES = {"next-codex-context", "codex-run-full", "audit-full"}
```

Add function:

```python
def is_context_receipt_log(rel: str, package_role: str) -> bool:
    parts = Path(rel).parts
    return (
        package_role in CONTEXT_LOG_PACKAGE_ROLES
        and len(parts) >= 2
        and parts[0] == ".codex-runs"
        and Path(rel).name in CONTEXT_LOG_BASENAMES
    )
```

Modify `.log` handling in `include_decision` before `log-disabled`.

## Add root package archive

Mirror the existing root markdown and codex archive result pattern. Preferred dataclass name:

```python
@dataclass
class RootPackageArchiveResult:
    enabled: bool
    dry_run: bool
    verify_only: bool
    archive_only: bool
    archive_root: str
    archive_dir: str
    manifest_path: str | None
    inspected_count: int
    protected_count: int
    candidate_count: int
    planned_count: int
    moved_count: int
    skipped_existing_count: int
    collision_count: int
    manifest_written: bool
    candidate_paths: list[str]
    protected_paths: list[str]
    moved: list[dict[str, Any]]
    planned: list[dict[str, Any]]
    skipped_existing: list[dict[str, Any]]
    collisions: list[dict[str, Any]]
    errors: list[str]
```

Implement:

- `classify_root_package_artifact(path, reserved_output_paths)`
- `iter_root_package_archive_candidates(...)`
- `archive_root_package_artifacts(...)`
- `root_package_archive_summary(...)`

Call before `collect_files`.
```

