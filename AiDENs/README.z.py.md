# z.py README

`z.py` is the AiDENs source/context archive certifier. It builds a ZIP archive
and companion audit files for Codex handoffs, release context packages, and
research-heavy Rust workspaces.

The script is intentionally single-file and Python-stdlib only. It is designed
to make archive output inspectable before it leaves the machine: included files
are hashed, excluded files are explained, required project surfaces are checked,
and common self-containment failures are reported.

## Quick Start

From the AiDENs repository root:

```bash
python3 z.py --root . --profile aidens --mode next-codex-context
```

For a strict dry run that emits reports without writing the ZIP:

```bash
python3 z.py --root . --profile aidens --mode next-codex-context --dry-run
```

For a full Codex-run handoff package:

```bash
python3 z.py --root . --profile aidens --mode codex-run-full
```

For the parent Libraries workspace:

```bash
python3 z.py --root .. --profile libraries --mode next-codex-context
```

## Outputs

Unless overridden, `z.py` writes these files beside the archive:

- `<archive>.zip`
- `<archive>.manifest.json`
- `<archive>.report.md`
- `<archive>.excluded.json`
- `<archive>.findings.json`
- `<archive>.codex-archive.json` when Codex-run archival normalization runs

The generated Markdown report is the fastest human review surface. The manifest
and findings JSON are the machine-readable audit surfaces.

## Modes

Supported modes are:

- `source-clean`
- `release-context`
- `next-codex-context`
- `codex-context` legacy alias for `next-codex-context`
- `codex-run-full`
- `full-context` legacy alias for `codex-run-full`
- `research-context`
- `audit-full`

Use `next-codex-context` for ordinary Codex handoff packages. Use
`codex-run-full` only when the active run context itself should be packaged. Use
`audit-full` when archived Codex-run history is intentionally part of the audit
payload.

## Profiles

Supported profiles are:

- `auto`
- `aidens`
- `libraries`
- `recall`
- `recall-coding`
- `generic-rust`
- `generic`
- `research`

`auto` infers the profile from the root. For reproducible handoffs, prefer
passing the profile explicitly.

## Validation Behavior

`z.py` runs in strict mode by default. If validation errors are found, it exits
with code `2` and does not write the ZIP. Sidecar reports are still written so
the failure can be inspected.

Use `--no-strict` only when producing a diagnostic package with known issues:

```bash
python3 z.py --root . --profile aidens --mode next-codex-context --no-strict
```

Exit codes:

- `0`: archive written or dry run completed
- `2`: validation failed under strict mode
- `1`: unexpected operational failure

## Codex-Run Archive Hygiene

By default, `z.py` normalizes stale active Codex-run artifacts into
`docs/codex-runs/archive/` before packaging. The current run defaults to `P23`
and can be changed with:

```bash
python3 z.py --root . --codex-current-run P24
```

Useful archive-hygiene commands:

```bash
python3 z.py --root . --archive-only
python3 z.py --root . --verify-codex-archive-hygiene
python3 z.py --root . --include-codex-archive --mode audit-full
```

`--no-archive-codex-runs` is diagnostic only. In strict packaging, stale active
Codex-run artifacts remain validation errors.

## Inclusion Controls

Common toggles:

- `--include-external-path-deps` or `--no-include-external-path-deps`
- `--include-generated-schemas` or `--exclude-generated-schemas`
- `--include-codex-artifacts` or `--exclude-codex-artifacts`
- `--include-doc-binaries` or `--exclude-doc-binaries`
- `--include-images` or `--exclude-images`
- `--include-logs` or `--exclude-logs`
- `--follow-symlinks`
- `--max-file-size-mb <number>`

Secret-like filenames are excluded by default. Do not make
`--allow-secret-like-names` the default; use it only for an explicit diagnostic
run after reviewing the file list.

## Reference Checks

Enabled by default:

- Rust `include_str!` and `include_bytes!` reference checks
- Cargo path dependency self-containment checks
- shell script references to `.sh` and `.py` files
- high-risk secret-pattern content scanning

Each check has a matching `--no-check-*` flag for diagnostic runs, but normal
handoff packages should leave the checks enabled.

## Determinism

ZIP entry timestamps are deterministic by default. Use `--preserve-mtime` only
when timestamp preservation is more important than byte-for-byte reproducibility.

Compression defaults to level `9` and can be changed with:

```bash
python3 z.py --root . --compresslevel 6
```

## Review Checklist

Before treating a package as a handoff artifact:

1. Read `<archive>.report.md`.
2. Confirm validation findings are empty or explicitly accepted.
3. Review `<archive>.excluded.json` for unexpected omissions.
4. Confirm external Cargo path dependencies are included or intentionally
   downgraded.
5. Confirm Codex-run archive hygiene is clean for the intended current run.

Use `python3 z.py --help` for the authoritative CLI option list.
