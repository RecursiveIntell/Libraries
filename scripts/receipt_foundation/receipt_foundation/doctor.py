"""Read-only operator-host preflight. No downloads, installs, git writes or services."""
from __future__ import annotations
import hashlib
import importlib.metadata
import os
import pathlib
import platform
import re
import shutil
import sqlite3
import subprocess
import sys
from .bundle import _private_directory
from .common import FoundationError, Limits, VERSION, open_source
from .projection import code_digest


def doctor(archive: pathlib.Path, output_parent: pathlib.Path, *, expected_archive_sha256: str,
           repo: pathlib.Path | None=None, expected_head: str | None=None) -> dict:
    checks=[]
    def record(name, ok, detail):checks.append({'name':name,'status':'PASS' if ok else 'BLOCKED','detail':detail})
    record('linux_platform',sys.platform=='linux',platform.system())
    record('python_version',sys.version_info[:2]>=(3,11),platform.python_version())
    record('sqlite_version',sqlite3.sqlite_version_info>=(3,38,0),sqlite3.sqlite_version)
    record('expected_archive_digest',bool(re.fullmatch('[0-9a-f]{64}',expected_archive_sha256)),'operator-supplied SHA-256 required')
    try:
        h=hashlib.sha256();size=0
        with open_source(archive) as f:
            while part:=f.read(65536):
                size+=len(part)
                if size>Limits().archive_bytes:raise FoundationError('COMPRESSED_SIZE_LIMIT')
                h.update(part)
        record('archive_digest',h.hexdigest()==expected_archive_sha256,{'sha256':h.hexdigest(),'bytes':size})
    except (FoundationError,OSError) as e:record('archive_digest',False,e.code if isinstance(e,FoundationError) else 'SOURCE_READ_FAILURE')
    try:
        fd=_private_directory(output_parent)
        os.close(fd)
        free=shutil.disk_usage(output_parent).free
        record('private_output_parent',True,'owned directory with no group/other access; no symlink ancestors')
        record('free_space',free>=2*Limits().projection_bytes,{'bytes_available':free,'minimum_bytes':2*Limits().projection_bytes})
    except (FoundationError,OSError) as e:record('private_output_parent',False,e.code if isinstance(e,FoundationError) else 'OUTPUT_PARENT_FAILURE')
    try:
        z=subprocess.run(['zstd','--version'],capture_output=True,timeout=5,check=False)
        record('zstd_executable',z.returncode==0,{'exit_code':z.returncode,'version':z.stdout.decode('ascii','replace').strip()[:256]})
    except (OSError,subprocess.SubprocessError):record('zstd_executable',False,'ZSTD_UNAVAILABLE')
    dependencies={}
    for name in ('jsonschema','jsonschema-specifications','attrs','referencing','rpds-py'):
        try:dependencies[name]=importlib.metadata.version(name)
        except importlib.metadata.PackageNotFoundError:dependencies[name]=None
    record('schema_validation_dependencies',all(dependencies.values()),dependencies)
    if (repo is None)!=(expected_head is None):raise FoundationError('REPO_AND_EXPECTED_HEAD_REQUIRED_TOGETHER')
    repository_state={'state':'not_requested'}
    if repo is not None:
        if not re.fullmatch('[0-9a-f]{40}',expected_head or ''):raise FoundationError('EXPECTED_REPO_HEAD_INVALID')
        def git(*args):
            r=subprocess.run(['git','--no-optional-locks','-c','core.fsmonitor=false','-C',str(repo),*args],capture_output=True,timeout=10,check=False)
            if r.returncode:raise FoundationError('GIT_PREFLIGHT_FAILURE')
            return r.stdout.decode('utf-8','strict').strip()
        try:
            head=git('rev-parse','HEAD');branch=git('branch','--show-current');status=git('status','--porcelain=v1','--untracked-files=normal')
            repository_state={'head':head,'branch':branch,'dirty':bool(status),'status_sha256':hashlib.sha256(status.encode()).hexdigest(),
                              'instructions_acknowledged':False,'owner_approval':'operator_review_required'}
            record('repository_head',head==expected_head,head)
            record('repository_clean',not status,{'dirty':bool(status),'status_sha256':repository_state['status_sha256']})
        except (FoundationError,OSError,UnicodeError,subprocess.SubprocessError):record('repository_preflight',False,'GIT_PREFLIGHT_FAILURE')
    return {'schema':'ReceiptFoundationHostPreflightV1','status':'BLOCKED' if any(c['status']=='BLOCKED' for c in checks) else 'PASS',
            'checks':checks,'implementation_version':VERSION,'implementation_sha256':code_digest(),'repository_state':repository_state,
            'local_writes_performed':False,'owner_approval_established':False,'runtime_identity_established':False,
            'atomic_rename_filesystem_support':'validated_only_by_bundle_publication_not_this_read_only_probe'}
