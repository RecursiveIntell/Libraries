#!/usr/bin/env python3
"""Archive/restore exact generated Git blobs. Integrity is not remote authentication.

Python 3.14+ only. This helper never removes source/index entries. The archive
contains regular files only, and verification restores into a fresh external
root before reporting success. Original checkout timestamps/UIDs are not Git
source identity; bytes, Git modes and object IDs are.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import lzma
import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile

SCHEMA = 'LibrariesGeneratedQuarantineV1'
PREFIX = '_salvage_from_libraries2/Libraries2/target_schema_gen'
REPOSITORY = 'https://github.com/RecursiveIntell/Libraries.git'
MODES = {'100644': 0o644, '100755': 0o755}
ROOT = Path(__file__).resolve().parents[1]


class QuarantineError(ValueError):
    """A failed integrity/scope check; no source removal is authorized."""


def require(condition, code):
    if not condition:
        raise QuarantineError(code)


def hex_value(value, length):
    return isinstance(value, str) and re.fullmatch(r'[0-9a-f]{' + str(length) + '}', value) is not None


def safe_path(value):
    require(isinstance(value, str) and value.startswith(PREFIX + '/'), 'PATH_OUTSIDE_PREFIX')
    require(not any(c in value for c in ('\\', ':', '\x00', '\n', '\r')), 'UNSAFE_PATH')
    require(all(part not in ('', '.', '..') for part in value.split('/')), 'UNSAFE_PATH')
    return value


def git(repo, *args):
    # No lazy network fetch or system/global helper is needed for cached objects.
    env = {'PATH': os.environ.get('PATH', os.defpath), 'HOME': os.environ.get('HOME', ''),
           'LANG': 'C.UTF-8', 'LC_ALL': 'C.UTF-8', 'GIT_NO_LAZY_FETCH': '1',
           'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': os.devnull}
    return subprocess.check_output(['git', '-C', str(repo), *args], env=env, stderr=subprocess.PIPE)


def inventory(repo, commit):
    require(hex_value(commit, 40), 'SOURCE_COMMIT_FORMAT')
    require(git(repo, 'rev-parse', '--show-object-format').strip() == b'sha1', 'UNSUPPORTED_GIT_OBJECT_FORMAT')
    require(git(repo, 'cat-file', '-t', commit).strip() == b'commit', 'SOURCE_OBJECT_NOT_COMMIT')
    tree = git(repo, 'rev-parse', commit + '^{tree}').decode().strip()
    rows = []
    for raw in git(repo, 'ls-tree', '-r', '-z', '-l', commit, '--', PREFIX).split(b'\x00'):
        if not raw:
            continue
        meta, name = raw.split(b'\t', 1)
        mode, kind, oid, size = meta.decode('ascii').split()
        path = safe_path(name.decode('utf-8', 'strict'))
        require(kind == 'blob' and mode in MODES, 'UNSUPPORTED_SOURCE_ENTRY')
        rows.append({'path': path, 'mode': mode, 'git_oid': oid, 'bytes': int(size)})
    require(bool(rows), 'EMPTY_SOURCE_TREE')
    return tree, sorted(rows, key=lambda row: row['path'])


def sha256_file(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def blob_oid(data):
    return hashlib.sha1(b'blob ' + str(len(data)).encode('ascii') + b'\x00' + data).hexdigest()


def no_duplicates(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'DUPLICATE_JSON_KEY')
        result[key] = value
    return result


def validate(repo, document, expected_source=None):
    require(isinstance(document, dict) and set(document) == {'schema', 'prefix', 'source', 'archive', 'entry_count', 'total_bytes', 'entries'}, 'MANIFEST_FIELDS')
    require(document['schema'] == SCHEMA and document['prefix'] == PREFIX, 'MANIFEST_SCOPE')
    source = document['source']
    require(isinstance(source, dict) and set(source) == {'repository', 'commit', 'tree', 'object_format'}, 'SOURCE_FIELDS')
    require(source['repository'] == REPOSITORY and source['object_format'] == 'sha1', 'SOURCE_DESCRIPTION')
    if expected_source is not None:
        require(hex_value(expected_source, 40), 'EXPECTED_SOURCE_FORMAT')
        require(source['commit'] == expected_source, 'SOURCE_COMMIT_MISMATCH')
    require(hex_value(source['tree'], 40), 'SOURCE_TREE_FORMAT')
    tree, actual = inventory(repo, source['commit'])
    require(tree == source['tree'], 'SOURCE_TREE_MISMATCH')
    archive = document['archive']
    require(isinstance(archive, dict) and set(archive) == {'file', 'sha256', 'format'}, 'ARCHIVE_FIELDS')
    require(isinstance(archive['file'], str) and re.fullmatch(r'[A-Za-z0-9._-]+\.tar\.xz', archive['file']) is not None, 'ARCHIVE_FILENAME')
    require(hex_value(archive['sha256'], 64) and archive['format'] == 'tar.xz', 'ARCHIVE_DESCRIPTION')
    entries = document['entries']
    require(isinstance(entries, list) and bool(entries), 'ENTRIES_REQUIRED')
    require(type(document['entry_count']) is int and document['entry_count'] == len(entries), 'ENTRY_COUNT')
    require(type(document['total_bytes']) is int, 'TOTAL_BYTES_TYPE')
    names = set()
    for entry in entries:
        require(isinstance(entry, dict) and set(entry) == {'path', 'mode', 'git_oid', 'bytes', 'sha256'}, 'ENTRY_FIELDS')
        path = safe_path(entry['path'])
        require(path not in names, 'DUPLICATE_MANIFEST_PATH')
        names.add(path)
        require(isinstance(entry['mode'], str) and entry['mode'] in MODES and hex_value(entry['git_oid'], 40), 'ENTRY_IDENTITY')
        require(type(entry['bytes']) is int and entry['bytes'] >= 0 and hex_value(entry['sha256'], 64), 'ENTRY_DIGEST')
    declared = sorted([{key: row[key] for key in ('path', 'mode', 'git_oid', 'bytes')} for row in entries], key=lambda row: row['path'])
    require([row['path'] for row in entries] == sorted(names), 'UNSORTED_MANIFEST_PATHS')
    require(declared == actual, 'SOURCE_INVENTORY_MISMATCH')
    require(document['total_bytes'] == sum(row['bytes'] for row in entries), 'TOTAL_BYTES')
    return {row['path']: row for row in entries}


def restore_check(repo, document, archive, destination, expected_source):
    entries = validate(repo, document, expected_source)
    require(sha256_file(archive) == document['archive']['sha256'], 'ARCHIVE_DIGEST_MISMATCH')
    destination = Path(destination)
    require(not os.path.lexists(destination), 'DESTINATION_EXISTS')
    require(not destination.resolve().is_relative_to(Path(repo).resolve()), 'RESTORE_MUST_BE_EXTERNAL')
    destination.mkdir(mode=0o700)
    seen = set()
    with tarfile.open(archive, mode='r:xz') as stream:
        for member in stream:
            name = safe_path(member.name)
            require(member.isfile() and not member.issparse(), 'NONREGULAR_ARCHIVE_MEMBER')
            # PAX long-path metadata is needed by this historical tree. No
            # link, sparse, size or other extension may override the carrier.
            require(set(member.pax_headers) <= {'path'}, 'UNSUPPORTED_TAR_EXTENSION')
            require(not member.pax_headers or member.pax_headers['path'] == name, 'TAR_PATH_OVERRIDE')
            require(name in entries and name not in seen, 'EXTRA_OR_DUPLICATE_ARCHIVE_MEMBER')
            entry = entries[name]
            require(member.size == entry['bytes'] and member.mode == MODES[entry['mode']], 'ARCHIVE_MEMBER_METADATA')
            body = stream.extractfile(member)
            if body is None:
                raise QuarantineError('MISSING_ARCHIVE_BODY')
            with body:
                data = body.read(entry['bytes'] + 1)
            require(len(data) == entry['bytes'], 'CONTENT_SIZE_MISMATCH')
            require(hashlib.sha256(data).hexdigest() == entry['sha256'], 'CONTENT_SHA256_MISMATCH')
            require(blob_oid(data) == entry['git_oid'], 'CONTENT_GIT_OID_MISMATCH')
            path = destination / name
            path.parent.mkdir(parents=True, exist_ok=True)
            with path.open('xb') as target:
                target.write(data)
            path.chmod(MODES[entry['mode']])
            seen.add(name)
    require(seen == set(entries), 'MISSING_ARCHIVE_MEMBER')
    restored = {p.relative_to(destination).as_posix() for p in destination.rglob('*') if p.is_file()}
    require(restored == seen, 'RESTORE_INVENTORY_MISMATCH')
    for name, entry in entries.items():
        path = destination / name
        require(not path.is_symlink() and sha256_file(path) == entry['sha256'], 'RESTORE_DIGEST_MISMATCH')
        require(path.stat().st_mode & 0o7777 == MODES[entry['mode']], 'RESTORE_MODE_MISMATCH')
    return {'state': 'verified', 'source': document['source']['commit'], 'entries': len(seen),
            'total_bytes': document['total_bytes'], 'git_inventory_checked': True,
            'restore_readback': True, 'expected_source_checked': expected_source is not None,
            'scope': 'integrity against the provided local Git snapshot, not remote authentication'}


def verify(repo, manifest, *, archive_override=None, extract_to=None, expected_source=None):
    manifest = Path(manifest)
    require(manifest.stat().st_size <= 16 * 1024 * 1024, 'MANIFEST_TOO_LARGE')
    document = json.loads(manifest.read_text(encoding='utf-8'), object_pairs_hook=no_duplicates)
    validate(repo, document, expected_source)
    archive = Path(archive_override) if archive_override is not None else manifest.parent / document['archive']['file']
    if extract_to is not None:
        return restore_check(repo, document, archive, extract_to, expected_source)
    # Versioned manifests may live in source; scratch must remain external.
    with tempfile.TemporaryDirectory(prefix='quarantine-readback-') as scratch:
        return restore_check(repo, document, archive, Path(scratch) / 'restored', expected_source)


def create(repo, commit, manifest):
    manifest = Path(manifest)
    require(not manifest.resolve().is_relative_to(Path(repo).resolve()), 'ARCHIVE_MUST_BE_EXTERNAL')
    require(not os.path.lexists(manifest), 'MANIFEST_EXISTS')
    require(not git(repo, 'status', '--porcelain', '--untracked-files=all', '--', PREFIX).strip(), 'SOURCE_WORKTREE_DIRTY')
    tree, rows = inventory(repo, commit)
    archive = manifest.parent / ('target_schema_gen-' + commit + '.tar.xz')
    manifest.parent.mkdir(parents=True, exist_ok=True)
    epoch = int(git(repo, 'show', '-s', '--format=%ct', commit))
    require(not os.path.lexists(archive), 'ARCHIVE_EXISTS')
    # Publish no final-named artifact until the complete archive has passed an
    # independent restoration. Hard-link publication is atomic and no-clobber;
    # the manifest is linked last as the completion marker. A crash between
    # links can leave only an orphan archive, never a successful manifest.
    with tempfile.TemporaryDirectory(prefix='.quarantine-incomplete-', dir=manifest.parent) as scratch:
        scratch = Path(scratch)
        pending_archive = scratch / archive.name
        entries = []
        with pending_archive.open('xb') as raw:
            with tarfile.open(fileobj=raw, mode='w:xz', format=tarfile.PAX_FORMAT) as stream:
                for row in rows:
                    data = git(repo, 'cat-file', 'blob', row['git_oid'])
                    require(len(data) == row['bytes'] and blob_oid(data) == row['git_oid'], 'SOURCE_BLOB_MISMATCH')
                    member = tarfile.TarInfo(row['path'])
                    member.size = len(data)
                    member.mode = MODES[row['mode']]
                    member.mtime = epoch
                    stream.addfile(member, io.BytesIO(data))
                    entries.append({**row, 'sha256': hashlib.sha256(data).hexdigest()})
            raw.flush()
            os.fsync(raw.fileno())
        document = {'schema': SCHEMA, 'prefix': PREFIX,
                    'source': {'repository': REPOSITORY, 'commit': commit, 'tree': tree, 'object_format': 'sha1'},
                    'archive': {'file': archive.name, 'sha256': sha256_file(pending_archive), 'format': 'tar.xz'},
                    'entry_count': len(entries), 'total_bytes': sum(row['bytes'] for row in entries), 'entries': entries}
        result = restore_check(repo, document, pending_archive, scratch / 'restored', commit)
        pending_manifest = scratch / 'manifest.json'
        with pending_manifest.open('x', encoding='utf-8') as output:
            json.dump(document, output, indent=2, sort_keys=True)
            output.write('\n')
            output.flush()
            os.fsync(output.fileno())
        os.link(pending_archive, archive)
        try:
            os.link(pending_manifest, manifest)
        except OSError:
            archive.unlink()  # only the link created by this invocation
            raise
    return result


def main():
    require(sys.version_info >= (3, 14), 'PYTHON_314_REQUIRED')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', required=True, type=Path)
    parser.add_argument('--repo', type=Path, default=ROOT)
    parser.add_argument('--source', help='exact full source commit; required for creation, optional pinned expectation when verifying')
    parser.add_argument('--create', action='store_true')
    parser.add_argument('--archive', type=Path, help='archive location override for a moved external archive')
    parser.add_argument('--extract-to', type=Path, help='keep restoration in this fresh external directory')
    args = parser.parse_args()
    try:
        if args.create:
            require(args.source is not None and args.archive is None and args.extract_to is None, 'CREATE_ARGUMENTS')
            result = create(args.repo, args.source, args.manifest)
        else:
            result = verify(args.repo, args.manifest, archive_override=args.archive, extract_to=args.extract_to, expected_source=args.source)
        print(json.dumps(result, sort_keys=True))
        return 0
    except (QuarantineError, OSError, subprocess.CalledProcessError, tarfile.TarError, lzma.LZMAError, UnicodeError, json.JSONDecodeError) as error:
        print(json.dumps({'state': 'failed', 'detail': str(error)}), file=sys.stderr)
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
