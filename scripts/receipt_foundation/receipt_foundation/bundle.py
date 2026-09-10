"""Create-only private bundle publication, never native trust admission.

Linux-only no-replace directory rename is intentional. Unsupported systems fail
closed. Every public bundle query revalidates the manifest, exact files, SQL
schema, logical contents, build status, and current implementation binding.
"""
from __future__ import annotations

import contextlib
import ctypes
import errno
import hashlib
import json
import os
import pathlib
import re
import sqlite3
import stat
import sys
import tempfile
from typing import Any, Iterator

from .common import FoundationError, Limits, canonical_json, digest, identifier, write_json
from .contracts import validate_projection
from .projection import APPLICATION_ID, SCHEMA_VERSION, SQL, build, code_digest, logical_manifest, summarize
from .query import query
from .serialization import plain, strict_loads

FILES = ('projection.sqlite', 'build.json', 'contracts.json', 'query-proof.json')
MAX_JSON = 16 * 1024 * 1024
BUNDLE_SCHEMA = 'ReceiptObservationBundleV1'


def _private_directory(path: pathlib.Path) -> int:
    """Descriptor walk rejects symlink ancestors; the leaf is operator-private."""
    if sys.platform != 'linux' or not hasattr(os, 'O_NOFOLLOW'):
        raise FoundationError('BUNDLE_PLATFORM_UNSUPPORTED')
    if not path.is_absolute() or '..' in path.parts:
        raise FoundationError('BUNDLE_PATH_NOT_ABSOLUTE_OR_SAFE')
    fd = os.open('/', os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for name in path.parts[1:]:
            child = os.open(name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            os.close(fd)
            fd = child
        st = os.fstat(fd)
        if st.st_uid != os.geteuid() or stat.S_IMODE(st.st_mode) & 0o077:
            raise FoundationError('BUNDLE_DIRECTORY_NOT_PRIVATE')
        return fd
    except BaseException:
        os.close(fd)
        raise


def _safe_basename(name: str) -> None:
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]{0,127}', name):
        raise FoundationError('BUNDLE_NAME_INVALID')


def _publish_noreplace(parent_fd: int, staging: str, target: str) -> None:
    """No fallback to overwrite-capable rename; identical-filesystem fd anchor."""
    if sys.platform != 'linux':
        raise FoundationError('BUNDLE_PLATFORM_UNSUPPORTED')
    libc = ctypes.CDLL(None, use_errno=True)
    try:
        fn = libc.renameat2
    except AttributeError as exc:
        raise FoundationError('ATOMIC_DIRECTORY_RENAME_UNAVAILABLE') from exc
    fn.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint]
    fn.restype = ctypes.c_int
    if fn(parent_fd, os.fsencode(staging), parent_fd, os.fsencode(target), 1):
        error = ctypes.get_errno()
        if error in (errno.EEXIST, errno.ENOTEMPTY):
            raise FoundationError('BUNDLE_ALREADY_EXISTS')
        if error in (errno.ENOSYS, errno.EINVAL, errno.EOPNOTSUPP):
            raise FoundationError('ATOMIC_DIRECTORY_RENAME_UNAVAILABLE')
        raise FoundationError('ATOMIC_DIRECTORY_PUBLICATION_FAILURE')


def _file_facts(fd: int, max_bytes: int) -> dict[str, Any]:
    st = os.fstat(fd)
    if not stat.S_ISREG(st.st_mode) or st.st_nlink != 1:
        raise FoundationError('BUNDLE_FILE_NOT_SINGLE_REGULAR')
    if st.st_uid != os.geteuid() or stat.S_IMODE(st.st_mode) & 0o077:
        raise FoundationError('BUNDLE_FILE_NOT_PRIVATE')
    if st.st_size > max_bytes:
        raise FoundationError('BUNDLE_FILE_SIZE_LIMIT')
    os.lseek(fd, 0, os.SEEK_SET)
    h = hashlib.sha256()
    size = 0
    while part := os.read(fd, 65536):
        size += len(part)
        if size > max_bytes:
            raise FoundationError('BUNDLE_FILE_SIZE_LIMIT')
        h.update(part)
    after = os.fstat(fd)
    if (size, after.st_size, after.st_mtime_ns, after.st_ctime_ns) != (st.st_size, st.st_size, st.st_mtime_ns, st.st_ctime_ns):
        raise FoundationError('BUNDLE_CHANGED_DURING_READ')
    os.lseek(fd, 0, os.SEEK_SET)
    return {'bytes': size, 'sha256': h.hexdigest()}


def _load_json(fd: int) -> dict:
    os.lseek(fd, 0, os.SEEK_SET)
    parts = []
    size = 0
    while part := os.read(fd, 65536):
        size += len(part)
        if size > MAX_JSON:
            raise FoundationError('BUNDLE_JSON_SIZE_LIMIT')
        parts.append(part)
    # Exact numeric lexemes stay lexemes in operational observations too.
    obj = plain(strict_loads(b''.join(parts)))
    if not isinstance(obj, dict):
        raise FoundationError('BUNDLE_JSON_OBJECT_REQUIRED')
    return obj


def _sql_schema(db: sqlite3.Connection) -> list:
    return [tuple(r) for r in db.execute("SELECT type,name,tbl_name,sql FROM sqlite_schema WHERE sql IS NOT NULL ORDER BY type,name")]


def query_proof(db: sqlite3.Connection) -> dict:
    """Data-dependent witnesses; absent source evidence is not fabricated."""
    cases: list[dict] = []
    def check(kind: str, value: str | None = None, available: bool = True) -> None:
        if not available:
            cases.append({'kind': kind, 'status': 'NO_SOURCE_WITNESS', 'reason': 'absent_in_this_corpus'})
            return
        result = query(db, kind, value, limit=2)
        cases.append({'kind': kind, 'status': 'PASS' if result['rows'] else 'FAIL',
                      'rows': len(result['rows']), 'witness_value_sha256': digest(value.encode()) if value else None})
    first = db.execute('SELECT r.record_id,r.occurrence_id,o.blob_sha256,r.native_schema,r.category FROM records r JOIN occurrences o USING(occurrence_id) ORDER BY r.record_id LIMIT 1').fetchone()
    check('records', available=first is not None)
    for kind, column in [('digest', 'blob_sha256'), ('occurrences', 'blob_sha256'), ('occurrence', 'occurrence_id'), ('provenance', 'record_id')]:
        check(kind, first[column] if first else None, first is not None)
    check('family', (first['native_schema'] or first['category']) if first else None, first is not None)
    for kind, field in [('revision', 'revision'), ('run', 'run_id'), ('trace', 'trace_id'), ('session', 'session_id'), ('episode', 'episode_id'), ('task', 'task_id'), ('goal', 'goal_id')]:
        row = db.execute("SELECT value_json FROM fields WHERE field_group='field' AND name=? AND state='source_asserted' AND json_type(value_json)='text' ORDER BY record_id LIMIT 1", (field,)).fetchone()
        check(kind, json.loads(row[0]) if row else None, row is not None)
    for kind, sql in [('verification', "SELECT 1 FROM edges WHERE kind='VERIFIES' LIMIT 1"), ('artifacts', 'SELECT 1 FROM artifact_refs LIMIT 1'), ('quarantine', 'SELECT 1 FROM anomalies LIMIT 1'), ('duplicates', 'SELECT 1 FROM occurrences WHERE blob_sha256 IS NOT NULL GROUP BY blob_sha256 HAVING count(*)>1 LIMIT 1')]:
        check(kind, available=db.execute(sql).fetchone() is not None)
    return {'schema': 'ReceiptBundleQueryProofV1', 'status': 'FAIL' if any(c['status']=='FAIL' for c in cases) else 'PASS',
            'checks': cases, 'native_semantic_verification': 'not_performed'}


@contextlib.contextmanager
def verified_bundle(path: pathlib.Path, *, expected_archive_sha256: str, expected_code_sha256: str | None = None) -> Iterator[tuple[sqlite3.Connection, dict]]:
    """Yield the same descriptor-pinned DB that was validated, not a reopened path."""
    if not re.fullmatch('[0-9a-f]{64}', expected_archive_sha256):
        raise FoundationError('EXPECTED_ARCHIVE_DIGEST_INVALID')
    expected_code = expected_code_sha256 or code_digest()
    directory_fd = _private_directory(path)
    descriptors: dict[str, int] = {}
    db = None
    try:
        if set(os.listdir(directory_fd)) != set(FILES) | {'bundle.json'}:
            raise FoundationError('BUNDLE_MEMBER_SET_MISMATCH')
        facts = {}
        for name in (*FILES, 'bundle.json'):
            fd = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=directory_fd)
            descriptors[name] = fd
            facts[name] = _file_facts(fd, Limits().projection_bytes if name=='projection.sqlite' else MAX_JSON)
        manifest = _load_json(descriptors['bundle.json'])
        required = {'schema', 'bundle_id', 'archive_sha256', 'implementation_sha256', 'logical_sha256', 'files', 'trust', 'publication_protocol'}
        if set(manifest) != required or manifest['schema'] != BUNDLE_SCHEMA:
            raise FoundationError('BUNDLE_MANIFEST_CONTRACT_INVALID')
        material = {k:v for k,v in manifest.items() if k!='bundle_id'}
        if manifest['bundle_id'] != identifier('bundle', material):
            raise FoundationError('BUNDLE_ID_MISMATCH')
        if manifest['files'] != {name:facts[name] for name in FILES}:
            raise FoundationError('BUNDLE_FILE_DIGEST_MISMATCH')
        if manifest['archive_sha256'] != expected_archive_sha256:
            raise FoundationError('BUNDLE_ARCHIVE_BINDING_MISMATCH')
        if manifest['implementation_sha256'] != expected_code:
            raise FoundationError('BUNDLE_IMPLEMENTATION_BINDING_MISMATCH')
        if manifest['trust'] != 'observation_only_no_native_admission' or manifest['publication_protocol'] != 'linux-renameat2-noreplace-fsync-v1':
            raise FoundationError('BUNDLE_TRUST_CONTRACT_INVALID')
        receipt = _load_json(descriptors['build.json'])
        contract = _load_json(descriptors['contracts.json'])
        proof = _load_json(descriptors['query-proof.json'])
        if receipt.get('schema') != 'ReceiptProjectionBuildV1' or receipt.get('state') != 'built':
            raise FoundationError('BUNDLE_BUILD_NOT_SUCCESSFUL')
        if receipt.get('collection_integrity', {}).get('status') != 'PASS':
            raise FoundationError('BUNDLE_COLLECTION_NOT_VERIFIED')
        if contract.get('status') != 'PASS' or proof.get('status') != 'PASS':
            raise FoundationError('BUNDLE_VALIDATION_NOT_SUCCESSFUL')
        db = sqlite3.connect(f'file:/proc/self/fd/{descriptors["projection.sqlite"]}?mode=ro&immutable=1', uri=True)
        db.row_factory = sqlite3.Row
        db.execute('PRAGMA query_only=ON')
        db.execute('PRAGMA trusted_schema=OFF')
        if db.execute('PRAGMA application_id').fetchone()[0]!=APPLICATION_ID or db.execute('PRAGMA user_version').fetchone()[0]!=SCHEMA_VERSION:
            raise FoundationError('PROJECTION_VERSION_MISMATCH')
        with contextlib.closing(sqlite3.connect(':memory:')) as expected:
            expected.executescript(SQL)
            if _sql_schema(db) != _sql_schema(expected):
                raise FoundationError('BUNDLE_SQL_SCHEMA_MISMATCH')
        if db.execute('PRAGMA integrity_check').fetchone()[0]!='ok' or list(db.execute('PRAGMA foreign_key_check')):
            raise FoundationError('BUNDLE_DATABASE_INTEGRITY_FAILURE')
        md={r[0]:json.loads(r[1]) for r in db.execute('SELECT key,value FROM metadata')}
        if md.get('implementation_sha256')!=expected_code or receipt.get('implementation_sha256')!=expected_code:
            raise FoundationError('BUNDLE_IMPLEMENTATION_BINDING_MISMATCH')
        if [r[0] for r in db.execute('SELECT archive_sha256 FROM collections')] != [expected_archive_sha256] or receipt.get('archive_sha256')!=expected_archive_sha256:
            raise FoundationError('BUNDLE_ARCHIVE_BINDING_MISMATCH')
        logical=logical_manifest(db)
        if logical != receipt.get('logical_manifest') or logical['logical_sha256'] != manifest['logical_sha256']:
            raise FoundationError('BUNDLE_LOGICAL_BINDING_MISMATCH')
        summary=summarize(db)
        if summary != receipt.get('summary') or summary['unaccounted_members'] or summary['export_allowed_records']:
            raise FoundationError('BUNDLE_SUMMARY_OR_PRIVACY_MISMATCH')
        if query_proof(db) != proof:
            raise FoundationError('BUNDLE_QUERY_PROOF_MISMATCH')
        report={'schema':'ReceiptBundleVerificationV1','status':'PASS','bundle_id':manifest['bundle_id'],
                'archive_sha256':expected_archive_sha256,'implementation_sha256':expected_code,
                'logical_sha256':logical['logical_sha256'],'native_verification':'not_performed',
                'signature_verification':'not_performed','export_eligibility':'denied',
                'private_directory_and_files':True,'source_archive_rehashed_by_this_command':False}
        yield db,report
        # Detect changes during consumption as well as before it. This is not a
        # defense against an adversary controlling the same OS account.
        for name,fd in descriptors.items():
            if _file_facts(fd, Limits().projection_bytes if name=='projection.sqlite' else MAX_JSON)!=facts[name]:
                raise FoundationError('BUNDLE_CHANGED_DURING_READ')
    finally:
        if db is not None:db.close()
        for fd in descriptors.values():os.close(fd)
        os.close(directory_fd)


def verify_bundle(path: pathlib.Path, *, expected_archive_sha256: str) -> dict:
    with verified_bundle(path, expected_archive_sha256=expected_archive_sha256) as (_, report):
        return report


def bundle_query(path: pathlib.Path, kind: str, value: str | None, *, expected_archive_sha256: str,
                 limit: int=20, offset: int=0, private: bool=False) -> dict:
    with verified_bundle(path, expected_archive_sha256=expected_archive_sha256) as (db, report):
        result=query(db,kind,value,limit=limit,offset=offset,private=private)
        result['bundle_verification']=report
        return result


def build_bundle(archive: pathlib.Path, destination: pathlib.Path, *, expected_archive_sha256: str,
                 recover_streams: bool=False, compression: str='zstd') -> dict:
    """The final name never exposes a database without all required witnesses."""
    _safe_basename(destination.name)
    if not re.fullmatch('[0-9a-f]{64}',expected_archive_sha256):
        raise FoundationError('EXPECTED_ARCHIVE_DIGEST_INVALID')
    parent_fd=_private_directory(destination.parent)
    staging=None
    published=False
    try:
        if destination.name in os.listdir(parent_fd):raise FoundationError('BUNDLE_ALREADY_EXISTS')
        # /proc fd access pins parent identity across path renames.
        anchored=pathlib.Path(f'/proc/self/fd/{parent_fd}')
        staging=pathlib.Path(tempfile.mkdtemp(prefix='.receipt-bundle-staging-',dir=anchored))
        receipt=build(archive,staging/'projection.sqlite',recover_streams=recover_streams,compression=compression,
                      expected_archive_sha256=expected_archive_sha256,require_collection_manifest=True)
        if receipt['state']!='built' or receipt['collection_integrity']['status']!='PASS':
            raise FoundationError('BUNDLE_COLLECTION_NOT_VERIFIED')
        with contextlib.closing(sqlite3.connect(f'file:{staging}/projection.sqlite?mode=ro',uri=True)) as db:
            db.row_factory=sqlite3.Row
            contract=validate_projection(db)
            proof=query_proof(db)
        if contract['status']!='PASS' or proof['status']!='PASS':raise FoundationError('BUNDLE_VALIDATION_NOT_SUCCESSFUL')
        write_json(staging/'build.json',receipt)
        write_json(staging/'contracts.json',contract)
        write_json(staging/'query-proof.json',proof)
        facts={}
        for name in FILES:
            fd=os.open(staging/name,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
            try:
                facts[name]=_file_facts(fd,Limits().projection_bytes if name=='projection.sqlite' else MAX_JSON)
                os.fsync(fd)
            finally:os.close(fd)
        manifest={'schema':BUNDLE_SCHEMA,'archive_sha256':receipt['archive_sha256'],
                  'implementation_sha256':receipt['implementation_sha256'],'logical_sha256':receipt['logical_manifest']['logical_sha256'],
                  'files':facts,'trust':'observation_only_no_native_admission','publication_protocol':'linux-renameat2-noreplace-fsync-v1'}
        manifest['bundle_id']=identifier('bundle',manifest)
        write_json(staging/'bundle.json',manifest)
        stage_fd=os.open(staging,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
        try:os.fsync(stage_fd)
        finally:os.close(stage_fd)
        if code_digest()!=receipt['implementation_sha256']:raise FoundationError('IMPLEMENTATION_CHANGED_DURING_BUILD')
        _publish_noreplace(parent_fd,staging.name,destination.name)
        published=True
        try:os.fsync(parent_fd)
        except OSError as exc:raise FoundationError('BUNDLE_PUBLICATION_DURABILITY_UNKNOWN') from exc
        return {'schema':'ReceiptBundlePublicationV1','status':'PASS','bundle_id':manifest['bundle_id'],
                'archive_sha256':manifest['archive_sha256'],'implementation_sha256':manifest['implementation_sha256'],
                'logical_sha256':manifest['logical_sha256'],'directory_fsync':'passed','native_admission':False,'deployment':False}
    except BaseException as exc:
        if staging is not None and not published:
            # Retain only our own hidden staging tree as quarantined diagnostics;
            # do not recursively erase user data, race winners, or old evidence.
            try:write_json(staging/'FAILURE.json',{'schema':'ReceiptBundleFailureV1','status':'FAIL',
                 'reason':exc.code if isinstance(exc,FoundationError) else 'LOCAL_EXECUTION_FAILURE','adoption_allowed':False})
            except OSError:pass
            except FoundationError:pass
        raise
    finally:
        os.close(parent_fd)
