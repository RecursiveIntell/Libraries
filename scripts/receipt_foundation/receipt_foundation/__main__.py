"""python -m receipt_foundation --help"""
from __future__ import annotations
import argparse, dataclasses, json, os, pathlib, sqlite3, sys
from .common import FoundationError, write_json
from .projection import build, connect_readonly, logical_manifest, summarize
from .query import KINDS, query, source_bytes
from .contracts import envelope, schemas, validate_projection, load_request
from .bundle import build_bundle, verify_bundle, bundle_query
from .doctor import doctor


def parser() -> argparse.ArgumentParser:
    p=argparse.ArgumentParser(description='Private, rebuildable receipt evidence observation. Never modifies source evidence.')
    sub=p.add_subparsers(dest='command',required=True)
    for name in ('build','rebuild-check'):
        q=sub.add_parser(name)
        q.add_argument('archive',type=pathlib.Path)
        q.add_argument('--recover-json-streams',action='store_true',help='Explicit historical recovery; conformance issues remain visible.')
        q.add_argument('--compression',choices=('zstd','tar'),default='zstd')
        q.add_argument('--expect-sha256')
        q.add_argument('--without-collection-manifest',action='store_true',help='Explicit generic fixture/foreign archive mode; no collector manifest claims.')
        if name=='build':
            q.add_argument('--output',type=pathlib.Path,required=True)
            q.add_argument('--receipt',type=pathlib.Path,required=True)
        else:q.add_argument('--work-dir',type=pathlib.Path,required=True,help='Must not already exist; only newly created derived files are removed.')
    q=sub.add_parser('doctor');q.add_argument('archive',type=pathlib.Path);q.add_argument('--output-parent',type=pathlib.Path,required=True);q.add_argument('--expect-sha256',required=True);q.add_argument('--repo',type=pathlib.Path);q.add_argument('--expect-head')
    q=sub.add_parser('bundle-build');q.add_argument('archive',type=pathlib.Path);q.add_argument('--destination',type=pathlib.Path,required=True);q.add_argument('--expect-sha256',required=True);q.add_argument('--recover-json-streams',action='store_true');q.add_argument('--compression',choices=('zstd','tar'),default='zstd')
    for name in ('bundle-verify','bundle-query'):
        q=sub.add_parser(name);q.add_argument('bundle',type=pathlib.Path);q.add_argument('--expect-sha256',required=True)
        if name=='bundle-query':
            q.add_argument('--kind',choices=KINDS,required=True);q.add_argument('--value');q.add_argument('--limit',type=int,default=20);q.add_argument('--offset',type=int,default=0);q.add_argument('--private',action='store_true')
    q=sub.add_parser('validate-contracts');q.add_argument('database',type=pathlib.Path)
    q=sub.add_parser('envelope');q.add_argument('database',type=pathlib.Path);q.add_argument('--record',required=True)
    q=sub.add_parser('schemas');q.add_argument('--output-dir',type=pathlib.Path,required=True)
    q=sub.add_parser('build-request');q.add_argument('archive',type=pathlib.Path);q.add_argument('--request',type=pathlib.Path,required=True);q.add_argument('--output',type=pathlib.Path,required=True);q.add_argument('--receipt',type=pathlib.Path,required=True)
    q=sub.add_parser('summary');q.add_argument('database',type=pathlib.Path)
    q=sub.add_parser('logical-manifest');q.add_argument('database',type=pathlib.Path)
    q=sub.add_parser('query');q.add_argument('database',type=pathlib.Path);q.add_argument('--kind',choices=KINDS,required=True)
    q.add_argument('--value');q.add_argument('--limit',type=int,default=20);q.add_argument('--offset',type=int,default=0);q.add_argument('--private',action='store_true')
    q=sub.add_parser('source');q.add_argument('database',type=pathlib.Path);q.add_argument('--archive',type=pathlib.Path,required=True)
    q.add_argument('--record',required=True);q.add_argument('--output',type=pathlib.Path,required=True)
    q.add_argument('--ack-private-content',action='store_true');q.add_argument('--compression',choices=('zstd','tar'),default='zstd')
    return p


def main(argv:list[str]|None=None) -> int:
    args=parser().parse_args(argv)
    try:
        if args.command in {'doctor','bundle-build','bundle-verify','bundle-query'}:
            if args.command=='doctor':r=doctor(args.archive,args.output_parent,expected_archive_sha256=args.expect_sha256,repo=args.repo,expected_head=args.expect_head)
            elif args.command=='bundle-build':r=build_bundle(args.archive,args.destination,expected_archive_sha256=args.expect_sha256,recover_streams=args.recover_json_streams,compression=args.compression)
            elif args.command=='bundle-verify':r=verify_bundle(args.bundle,expected_archive_sha256=args.expect_sha256)
            else:r=bundle_query(args.bundle,args.kind,args.value,expected_archive_sha256=args.expect_sha256,limit=args.limit,offset=args.offset,private=args.private)
            print(json.dumps(r,sort_keys=True,indent=2,ensure_ascii=True));return 3 if r.get('status') in {'FAIL','BLOCKED'} else 0
        if args.command=='schemas':
            args.output_dir.mkdir(mode=0o700,exist_ok=False)
            for name,obj in schemas().items():write_json(args.output_dir/name,obj)
            print(json.dumps({'schema_files':len(schemas())}));return 0
        if args.command=='build-request':
            opts=load_request(args.request)
            if args.receipt.exists() or args.receipt.is_symlink():raise FoundationError('RECEIPT_OUTPUT_ALREADY_EXISTS')
            r=build(args.archive,args.output,**opts);write_json(args.receipt,r)
            print(json.dumps({'state':r['state'],'logical_sha256':r['logical_manifest']['logical_sha256']}));return 3 if r['collection_integrity']['status']=='FAIL' else 0
        if args.command in {'build','rebuild-check'}:
            opts=dict(recover_streams=args.recover_json_streams,compression=args.compression,
                      require_collection_manifest=not args.without_collection_manifest,expected_archive_sha256=args.expect_sha256)
            if args.command=='build':
                if args.receipt.exists() or args.receipt.is_symlink():raise FoundationError('RECEIPT_OUTPUT_ALREADY_EXISTS')
                r=build(args.archive,args.output,**opts)
                write_json(args.receipt,r)
                print(json.dumps({'state':r['state'],'archive_sha256':r['archive_sha256'],
                                  'logical_sha256':r['logical_manifest']['logical_sha256'],'summary':r['summary']},sort_keys=True,indent=2))
                return 3 if r['collection_integrity']['status']=='FAIL' else 0
            if args.work_dir.exists() or args.work_dir.is_symlink():raise FoundationError('VALIDATION_DIRECTORY_ALREADY_EXISTS')
            args.work_dir.mkdir(mode=0o700)
            out=args.work_dir/'projection.sqlite'
            first=build(args.archive,out,**opts);write_json(args.work_dir/'build-a.json',first)
            # Delete only the derived output created by this command, never arbitrary state.
            out.unlink()
            second=build(args.archive,out,**opts);write_json(args.work_dir/'build-b.json',second)
            same=first['logical_manifest']==second['logical_manifest']
            r={'schema':'ReceiptRebuildValidationV1','status':'PASS' if same else 'FAIL','projection_deleted_between_builds':True,
               'archive_sha256':first['archive_sha256'],'first_logical_sha256':first['logical_manifest']['logical_sha256'],
               'second_logical_sha256':second['logical_manifest']['logical_sha256'],'table_manifests_equal':same,
               'first_runtime_seconds':first['runtime_seconds'],'second_runtime_seconds':second['runtime_seconds'],
               'full_corpus_integrity':second['collection_integrity']['status']}
            write_json(args.work_dir/'determinism.json',r);print(json.dumps(r,sort_keys=True,indent=2))
            return 0 if same and second['collection_integrity']['status']!='FAIL' else 3
        if args.command=='source':
            r=source_bytes(args.database,args.archive,args.record,args.output,acknowledge_private=args.ack_private_content,compression=args.compression)
        else:
            with connect_readonly(args.database) as db:
                r=validate_projection(db) if args.command=='validate-contracts' else envelope(db,args.record) if args.command=='envelope' else summarize(db) if args.command=='summary' else logical_manifest(db) if args.command=='logical-manifest' else query(db,args.kind,args.value,limit=args.limit,offset=args.offset,private=args.private)
        print(json.dumps(r,sort_keys=True,indent=2,ensure_ascii=True));return 3 if r.get('status')=='FAIL' else 0
    except (FoundationError,sqlite3.Error,OSError) as exc:
        err=exc.as_dict() if isinstance(exc,FoundationError) else {'code':'SQLITE_FAILURE' if isinstance(exc,sqlite3.Error) else 'FILESYSTEM_FAILURE','offset':None}
        failure={'schema':'ReceiptFoundationFailureV1','state':'failed','error':err,'raw_error_message_suppressed':True}
        if isinstance(exc,FoundationError) and exc.context:failure.update(exc.context)
        # Keep failure reporting separate from trusted evidence. No source excerpts.
        if getattr(args,'receipt',None) and not args.receipt.exists() and not args.receipt.is_symlink():
            try:write_json(args.receipt,failure)
            except FoundationError as report_error:
                failure['failure_receipt_write_error']=report_error.as_dict()
        print(json.dumps(failure,sort_keys=True),file=sys.stderr);return 2

if __name__=='__main__':raise SystemExit(main())
