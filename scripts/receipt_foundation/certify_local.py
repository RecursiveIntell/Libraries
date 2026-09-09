"""Receipt-bearing local certification of a newly created observation directory.

Never installs packages, modifies a repository, queries a live store, runs a
producer, fetches missing artifacts, publishes evidence, or activates a service.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
import pathlib
import subprocess
import sys
import time

from receipt_foundation.bundle import verify_bundle, FILES
from receipt_foundation.common import FoundationError, write_json
from receipt_foundation.projection import code_digest


def certify(archive:pathlib.Path, work:pathlib.Path, expected:str) -> dict:
    if not archive.is_absolute() or not work.is_absolute():raise FoundationError('ABSOLUTE_PATHS_REQUIRED')
    # Must be new; no --force, update-in-place, hidden retry or fallback modes.
    work.mkdir(mode=0o700,parents=False,exist_ok=False)
    commands=[]
    implementation=code_digest()
    receipt={'schema':'ReceiptLocalCertificationV1','status':'FAIL','implementation_sha256':implementation,
             'archive_sha256':expected,'commands':commands,'source_code_modified':False,
             'runtime_services_modified':False,'native_verification':'not_performed','native_adoption_gate':'BLOCKED',
             'source_script_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()}
    try:
        root=pathlib.Path(__file__).resolve().parent
        def run(label:str,args:list[str],seed:str='2718',timeout:int=240):
            stdout=work/(label+'.stdout.json');stderr=work/(label+'.stderr.log')
            env=dict(os.environ);env['PYTHONHASHSEED']=seed
            start=time.monotonic();rc=None
            # Private logs are created before subprocess execution; never echo source bodies.
            outfd=os.open(stdout,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
            errfd=os.open(stderr,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
            try:
                with os.fdopen(outfd,'wb') as out,os.fdopen(errfd,'wb') as err:
                    p=subprocess.run([sys.executable,*args],cwd=root,env=env,stdout=out,stderr=err,timeout=timeout,check=False)
                    rc=p.returncode
            finally:
                commands.append({'label':label,'argv':[sys.executable,*args],'exit_code':rc,'runtime_seconds':round(time.monotonic()-start,6),
                    'pythonhashseed':seed,'timeout_seconds':timeout,'stdout':stdout.name,'stderr':stderr.name,
                    'stdout_sha256':hashlib.sha256(stdout.read_bytes()).hexdigest(),'stderr_sha256':hashlib.sha256(stderr.read_bytes()).hexdigest()})
            if rc!=0:raise FoundationError('CERTIFICATION_COMMAND_FAILED')
        module=['-m','receipt_foundation']
        run('01-doctor',[*module,'doctor',str(archive),'--output-parent',str(work),'--expect-sha256',expected])
        run('02-tests',['-m','unittest','discover','-s','tests','-v'])
        first=work/'run-a';second=work/'observation-bundle'
        run('03-build-a',[*module,'bundle-build',str(archive),'--destination',str(first),'--recover-json-streams','--expect-sha256',expected],seed='3141')
        a=verify_bundle(first,expected_archive_sha256=expected)
        write_json(work/'BUILD_A_VERIFICATION.json',a)
        # Retain new operational witnesses before deleting only our freshly made projection.
        for name in ('build.json','bundle.json','contracts.json','query-proof.json'):
            write_json(work/('RUN_A_'+name),json.loads((first/name).read_text()))
        for name in (*FILES,'bundle.json'):(first/name).unlink()
        first.rmdir()
        receipt['derived_bundle_deleted_between_builds']=True
        run('04-build-b',[*module,'bundle-build',str(archive),'--destination',str(second),'--recover-json-streams','--expect-sha256',expected],seed='9265')
        run('05-verify-b',[*module,'bundle-verify',str(second),'--expect-sha256',expected])
        b=verify_bundle(second,expected_archive_sha256=expected)
        am=json.loads((work/'RUN_A_build.json').read_text())['logical_manifest']
        bm=json.loads((second/'build.json').read_text())['logical_manifest']
        receipt['determinism']={'status':'PASS' if am==bm else 'FAIL','first_logical_sha256':am['logical_sha256'],
                                'second_logical_sha256':bm['logical_sha256'],'all_table_manifests_equal':am==bm,
                                'independent_python_processes':True,'different_hash_seeds':True}
        if am!=bm:raise FoundationError('DETERMINISM_MISMATCH')
        run('06-contracts',[*module,'validate-contracts',str(second/'projection.sqlite')])
        run('07-corpus-audit',['audit_corpus.py',str(second/'projection.sqlite'),'--output-dir',str(work/'corpus-audit'),'--archive',str(archive)])
        summary=json.loads((second/'build.json').read_text())['summary']
        if sum(summary['member_dispositions'].values())!=summary['total_members'] or summary['unaccounted_members']:
            raise FoundationError('CORPUS_ACCOUNTING_MISMATCH')
        if code_digest()!=implementation:raise FoundationError('IMPLEMENTATION_CHANGED_DURING_CERTIFICATION')
        receipt.update(status='PASS',bundle_verification=b,summary=summary,foundation_verdict='FOUNDATION_NOT_READY',
                       verdict_reason='Offline certification passed. Operator-host ownership, native validation and adoption remain separate gates.')
    except (FoundationError,OSError,subprocess.SubprocessError,ValueError) as exc:
        receipt['error']={'code':exc.code if isinstance(exc,FoundationError) else 'LOCAL_CERTIFICATION_FAILURE','raw_error_suppressed':True}
    finally:
        write_json(work/'LOCAL_CERTIFICATION.json',receipt)
    return receipt


def main()->int:
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('archive',type=pathlib.Path);p.add_argument('--work-dir',type=pathlib.Path,required=True);p.add_argument('--expect-sha256',required=True)
    args=p.parse_args()
    try:
        r=certify(args.archive,args.work_dir,args.expect_sha256)
        print(json.dumps({'schema':r['schema'],'status':r['status'],'receipt':str(args.work_dir/'LOCAL_CERTIFICATION.json'),'error':r.get('error')},sort_keys=True))
        return 0 if r['status']=='PASS' else 3
    except (FoundationError,OSError) as exc:
        print(json.dumps({'status':'FAIL','error':exc.code if isinstance(exc,FoundationError) else 'WORK_DIRECTORY_CREATE_FAILURE'}),file=sys.stderr)
        return 2

if __name__=='__main__':raise SystemExit(main())
