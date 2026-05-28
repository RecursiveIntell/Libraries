#!/usr/bin/env python3

from __future__ import annotations

import argparse
from datetime import datetime
from pathlib import Path
import zipfile


EXCLUDED_DIR_NAMES = {
    ".git",
    ".claude",
    "target",
    "node_modules",
    "dist",
    ".next",
    "coverage",
    ".cache",
    ".tmp-rust",
    "library-source-zips",
    "source-zips-2026-03-10",
    "zips",
    "rendered",
    "ARCHIVE",
    "schemas.generated",
    "__pycache__",
}

EXCLUDED_DIR_PREFIXES = ("tmp", "tmp-")

EXCLUDED_FILE_EXTENSIONS = {
    ".zip",
    ".tar",
    ".gz",
    ".tgz",
    ".7z",
    ".rar",
    ".bz2",
    ".xz",
    ".dylib",
    ".so",
    ".dll",
    ".exe",
    ".pdf",
    ".docx",
}

ALLOWED_EXTENSIONS = {
    ".rs",
    ".toml",
    ".lock",
    ".md",
    ".txt",
    ".json",
    ".yml",
    ".yaml",
    ".ts",
    ".tsx",
    ".js",
    ".jsx",
    ".css",
    ".html",
    ".sql",
    ".sh",
    ".py",
    ".proto",
}

ALLOWED_BASENAMES = {
    ".gitignore",
    "LICENSE",
    "Makefile",
}


def should_skip_dir(path: Path) -> bool:
    name = path.name
    return name in EXCLUDED_DIR_NAMES or name.startswith(EXCLUDED_DIR_PREFIXES)


def should_include_file(path: Path, output_path: Path) -> bool:
    if path == output_path:
        return False

    if any(should_skip_dir(parent) for parent in path.parents):
        return False

    if path.suffix.lower() in EXCLUDED_FILE_EXTENSIONS:
        return False

    if path.name in ALLOWED_BASENAMES:
        return True

    return path.suffix.lower() in ALLOWED_EXTENSIONS


def iter_source_files(root: Path, output_path: Path) -> list[Path]:
    files: list[Path] = []
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        if should_include_file(path, output_path):
            files.append(path)
    files.sort()
    return files


def build_archive(root: Path, output_path: Path) -> tuple[int, int]:
    files = iter_source_files(root, output_path)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    with zipfile.ZipFile(output_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        for path in files:
            zf.write(path, path.relative_to(root))

    return len(files), sum(path.stat().st_size for path in files)


def default_output_name() -> str:
    return f"libraries-source-clean-{datetime.now().strftime('%Y%m%d')}.zip"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Create a source-only zip archive for the Libraries workspace."
    )
    parser.add_argument(
        "-o",
        "--output",
        default=default_output_name(),
        help="Output zip path. Defaults to libraries-source-clean-YYYYMMDD.zip",
    )
    parser.add_argument(
        "--root",
        default=".",
        help="Workspace root to archive. Defaults to current directory.",
    )
    args = parser.parse_args()

    root = Path(args.root).resolve()
    output_path = Path(args.output)
    if not output_path.is_absolute():
        output_path = root / output_path
    output_path = output_path.resolve()

    file_count, total_bytes = build_archive(root, output_path)
    print(f"wrote {output_path}")
    print(f"files={file_count} bytes={total_bytes}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
