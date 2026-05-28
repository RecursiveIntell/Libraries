# P22 `z.py` Implementation Notes

These are implementation notes, not a mandate to copy verbatim.

## Suggested data structures

```python
@dataclass(frozen=True)
class CodexArchiveCandidate:
    original_path: str
    run_id: str
    reason: str
    sha256: str
    bytes: int
    mtime_utc: str

@dataclass
class CodexArchiveResult:
    enabled: bool
    dry_run: bool
    moved: list[dict]
    skipped_existing: list[dict]
    collisions: list[dict]
    unclassified: list[dict]
    active_stale_after: list[str]
    manifest_paths: list[str]
```

## Suggested build order inside `build(args)`

```python
root = resolve_root(args)
policy = make_policy(args, resolved_profile)
if args.archive_codex_runs:
    codex_archive_result = normalize_codex_run_artifacts(root, args, dry_run=args.dry_run)
    if args.archive_only:
        write archive sidecars and return BuildResult-like summary
    if args.strict and codex_archive_result.active_stale_after:
        add error finding and do not zip
included = collect_files(...)
# collect_files must exclude docs/codex-runs/archive unless include_codex_archive/audit-full
```

## Secret scanner refinement

Add a predicate similar to:

```python
def is_nonliteral_rust_secret_field_copy(line: str) -> bool:
    return bool(re.search(r"\b(api[_-]?key|token|secret)\s*:\s*[A-Za-z_][\w.]*\.(api[_-]?key|token|secret)\.(clone|to_owned)\(\)\s*,?", line))
```

Use this only to suppress warnings for Rust field-copy patterns. Do not suppress string literals, env files, shell assignments, or config values.
