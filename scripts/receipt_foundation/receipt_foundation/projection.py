"""Disposable SQLite observation projection. Source bytes are never stored here."""
from __future__ import annotations
import collections, dataclasses, datetime as dt, hashlib, json, os, pathlib, re, resource, sqlite3, sys, tempfile, time
from typing import Any
from .archive import Archive, Member, path_problem
from .common import VERSION, RULES_VERSION, SCHEMA_VERSION, FoundationError, Limits, canonical_json, digest, identifier, open_source, write_json
from .serialization import parse, plain, strict_loads, Number, ParsedValue, structural_digest
from .normalize import normalize, COMMON_FIELDS, state
from .privacy import scan, sensitivity, scalar_safe

APPLICATION_ID=0x52435054
SQL='''
PRAGMA foreign_keys=ON;
CREATE TABLE metadata(key TEXT PRIMARY KEY, value TEXT NOT NULL) WITHOUT ROWID;
CREATE TABLE collections(archive_sha256 TEXT PRIMARY KEY CHECK(length(archive_sha256)=64), byte_length INTEGER NOT NULL CHECK(byte_length>=0)) WITHOUT ROWID;
CREATE TABLE blobs(sha256 TEXT PRIMARY KEY CHECK(length(sha256)=64), byte_length INTEGER NOT NULL CHECK(byte_length>=0)) WITHOUT ROWID;
CREATE TABLE occurrences(
 occurrence_id TEXT PRIMARY KEY, archive_sha256 TEXT NOT NULL REFERENCES collections(archive_sha256),
 ordinal INTEGER NOT NULL, header_offset INTEGER NOT NULL, data_offset INTEGER NOT NULL,
 archive_member TEXT, path_key TEXT, path_sha256 TEXT NOT NULL,
 member_type TEXT NOT NULL, byte_length INTEGER NOT NULL, header_sha256 TEXT NOT NULL,
 blob_sha256 TEXT REFERENCES blobs(sha256), disposition TEXT NOT NULL, extension TEXT NOT NULL,
 mode INTEGER NOT NULL, tar_mtime_raw TEXT NOT NULL, candidate INTEGER NOT NULL DEFAULT 0 CHECK(candidate IN (0,1)),
 source_path TEXT, collector_class TEXT, storage_root TEXT, filesystem_mtime_asserted TEXT,
 manifest_entry_index INTEGER, manifest_occurrence_id TEXT REFERENCES occurrences(occurrence_id),
 UNIQUE(archive_sha256,ordinal), UNIQUE(archive_sha256,header_offset)
) WITHOUT ROWID;
CREATE INDEX occurrences_blob ON occurrences(blob_sha256);
CREATE INDEX occurrences_source_path ON occurrences(source_path);
CREATE INDEX occurrences_path ON occurrences(path_key);
CREATE TABLE archive_controls(archive_sha256 TEXT NOT NULL REFERENCES collections(archive_sha256), header_offset INTEGER NOT NULL,
 kind TEXT NOT NULL, header_sha256 TEXT NOT NULL, payload_sha256 TEXT NOT NULL, payload_bytes INTEGER NOT NULL,
 PRIMARY KEY(archive_sha256,header_offset)) WITHOUT ROWID;
CREATE TABLE parses(occurrence_id TEXT PRIMARY KEY REFERENCES occurrences(occurrence_id), declared_format TEXT NOT NULL,
 observed_format TEXT NOT NULL, status TEXT NOT NULL, value_count INTEGER NOT NULL, recovery_mode TEXT NOT NULL) WITHOUT ROWID;
CREATE TABLE records(record_id TEXT PRIMARY KEY, occurrence_id TEXT NOT NULL REFERENCES occurrences(occurrence_id),
 value_index INTEGER NOT NULL, pointer TEXT NOT NULL, byte_start INTEGER NOT NULL, byte_end INTEGER NOT NULL CHECK(byte_end>=byte_start),
 structural_sha256 TEXT NOT NULL, projection_kind TEXT NOT NULL, category TEXT NOT NULL, context TEXT NOT NULL,
 native_schema TEXT, native_version_state TEXT NOT NULL, native_version_json TEXT, classification_json TEXT NOT NULL,
 sensitivity TEXT NOT NULL, export_eligibility TEXT NOT NULL CHECK(export_eligibility='denied'),
 UNIQUE(occurrence_id,value_index,pointer)) WITHOUT ROWID;
CREATE INDEX records_family ON records(native_schema,category);
CREATE INDEX records_structure ON records(structural_sha256);
CREATE TABLE fields(record_id TEXT NOT NULL REFERENCES records(record_id), field_group TEXT NOT NULL, name TEXT NOT NULL,
 state TEXT NOT NULL, value_json TEXT, source_pointer TEXT, rule TEXT NOT NULL, role TEXT, format_status TEXT,
 PRIMARY KEY(record_id,field_group,name)) WITHOUT ROWID;
CREATE INDEX fields_lookup ON fields(name,value_json,record_id);
CREATE TABLE edges(edge_id TEXT PRIMARY KEY, source_record_id TEXT NOT NULL REFERENCES records(record_id),
 kind TEXT NOT NULL, target_type TEXT NOT NULL, target_value TEXT NOT NULL,
 target_record_id TEXT REFERENCES records(record_id), resolution TEXT NOT NULL, source_pointer TEXT NOT NULL,
 rule TEXT NOT NULL, assertion_state TEXT NOT NULL) WITHOUT ROWID;
CREATE INDEX edges_target ON edges(target_record_id,kind);
CREATE TABLE artifact_refs(artifact_ref_id TEXT PRIMARY KEY, source_record_id TEXT NOT NULL REFERENCES records(record_id),
 role TEXT NOT NULL, source_pointer TEXT NOT NULL, path_asserted TEXT, sha256_asserted TEXT,
 matched_blob_sha256 TEXT REFERENCES blobs(sha256), resolution TEXT NOT NULL, rule TEXT NOT NULL) WITHOUT ROWID;
CREATE TABLE anomalies(anomaly_id TEXT PRIMARY KEY, occurrence_id TEXT REFERENCES occurrences(occurrence_id),
 record_id TEXT REFERENCES records(record_id), stage TEXT NOT NULL, code TEXT NOT NULL,
 severity TEXT NOT NULL, byte_start INTEGER, byte_end INTEGER, source_pointer TEXT,
 details_json TEXT NOT NULL, recoverable INTEGER NOT NULL CHECK(recoverable IN(0,1))) WITHOUT ROWID;
CREATE INDEX anomalies_code ON anomalies(code,severity);
CREATE TABLE schema_observations(record_id TEXT PRIMARY KEY REFERENCES records(record_id), schema_id TEXT, title TEXT,
 draft_uri TEXT, structural_sha256 TEXT NOT NULL, native_definition_validation TEXT NOT NULL) WITHOUT ROWID;
CREATE TABLE signature_observations(record_id TEXT NOT NULL REFERENCES records(record_id), source_pointer TEXT NOT NULL,
 field_name TEXT NOT NULL, source_value_type TEXT NOT NULL, verification TEXT NOT NULL CHECK(verification='not_performed'),
 PRIMARY KEY(record_id,source_pointer)) WITHOUT ROWID;
'''
LOGICAL_TABLES=('metadata','collections','blobs','occurrences','archive_controls','parses','records','fields','edges','artifact_refs','anomalies','schema_observations','signature_observations')


def code_digest() -> str:
    root=pathlib.Path(__file__).parent
    h=hashlib.sha256()
    for p in sorted(root.glob('*.py')):
        h.update(p.name.encode()+b'\0');h.update(p.read_bytes());h.update(b'\0')
    return h.hexdigest()


class _ReadonlyConnection:
    def __init__(self, path: pathlib.Path):
        # Open and validate immediately so invalid projections fail at construction.
        with open_source(path): pass
        try:
            self.db=sqlite3.connect(path.resolve().as_uri()+'?mode=ro',uri=True)
            self.db.execute('PRAGMA query_only=ON');self.db.execute('PRAGMA trusted_schema=OFF')
            if self.db.execute('PRAGMA application_id').fetchone()[0]!=APPLICATION_ID or self.db.execute('PRAGMA user_version').fetchone()[0]!=SCHEMA_VERSION:
                self.db.close();raise FoundationError('PROJECTION_VERSION_MISMATCH')
            self.db.row_factory=sqlite3.Row
        except sqlite3.Error as e:
            try:self.db.close()
            except AttributeError:pass
            raise FoundationError('PROJECTION_OPEN_FAILURE') from e
    def __enter__(self) -> sqlite3.Connection:return self.db
    def __exit__(self, exc_type, exc, traceback) -> None:self.db.close()


def connect_readonly(path: pathlib.Path) -> _ReadonlyConnection:
    return _ReadonlyConnection(path)


def logical_manifest(db: sqlite3.Connection) -> dict:
    tables={}
    for table in LOGICAL_TABLES:
        columns=[r[1] for r in db.execute(f'PRAGMA table_info({table})')]
        order=','.join('"'+x+'"' for x in columns)
        h=hashlib.sha256();count=0
        for row in db.execute(f'SELECT * FROM {table} ORDER BY {order}'):
            b=canonical_json(list(row)).encode();h.update(len(b).to_bytes(8,'big'));h.update(b);count+=1
        tables[table]={'row_count':count,'sha256':h.hexdigest(),'columns':columns}
    return {'schema':'ReceiptProjectionLogicalManifestV1','algorithm':'ordered-relational-json-v1','tables':tables,
            'logical_sha256':digest(canonical_json(tables).encode())}


def field_value(db: sqlite3.Connection, record_id: str, name: str) -> Any:
    r=db.execute('SELECT value_json FROM fields WHERE record_id=? AND field_group=? AND name=? AND state=?',(record_id,'field',name,'source_asserted')).fetchone()
    return json.loads(r[0]) if r and r[0] is not None else None


def insert_exact(db: sqlite3.Connection, table: str, values: tuple) -> None:
    """Idempotence admits only byte-for-byte equal row material, never collisions."""
    keys={'blobs':'sha256','anomalies':'anomaly_id','edges':'edge_id','artifact_refs':'artifact_ref_id'}
    if table not in keys:raise FoundationError('INTERNAL_TABLE_NOT_ALLOWED')
    old=db.execute(f'SELECT * FROM {table} WHERE {keys[table]}=?',(values[0],)).fetchone()
    if old is not None:
        if tuple(old)!=values:raise FoundationError('PROJECTION_IDENTITY_COLLISION')
        return
    marks=','.join('?' for _ in values)
    db.execute(f'INSERT INTO {table} VALUES({marks})',values)


class Builder:
    def __init__(self, db: sqlite3.Connection, *, recover_streams: bool, limits: Limits):
        self.db,self.recover_streams,self.limits=db,recover_streams,limits
        self.meta_sources:dict[str,tuple[str,bytes]]={}
        self.archive_digest=''
        self.record_count=0

    def anomaly(self, occurrence: str | None, stage: str, code: str, *, record: str | None=None,
                severity: str='error', start: int | None=None, end: int | None=None,
                pointer: str | None=None, details: dict | None=None, recoverable: bool=True) -> None:
        safe=details or {}
        aid=identifier('anomaly',occurrence,record,stage,code,start,end,pointer,safe)
        insert_exact(self.db,'anomalies',
                        (aid,occurrence,record,stage,code,severity,start,end,pointer,canonical_json(safe),int(recoverable)))

    def edge(self, record: str, kind: str, target_type: str, target: str, pointer: str,
             rule: str, *, resolution: str='unresolved', target_record: str | None=None, assertion: str='source_asserted') -> None:
        if not scalar_safe(target):
            row=self.db.execute('SELECT occurrence_id FROM records WHERE record_id=?',(record,)).fetchone()
            self.anomaly(row[0],'privacy','REFERENCE_EXCLUDED_SENSITIVE',record=record,pointer=pointer);return
        eid=identifier('edge',record,kind,target_type,target,pointer,rule)
        insert_exact(self.db,'edges',
                        (eid,record,kind,target_type,target,target_record,resolution,pointer,rule,assertion))

    def artifact(self, record: str, role: str, pointer: str, path: Any, sha: Any, *, relation: str='REFERENCES') -> None:
        safe_path=path if isinstance(path,str) and scalar_safe(path) else None
        raw_sha=sha if isinstance(sha,str) else None
        # Explicit SHA-256 keys, not generic hash guesses. Native spelling stays in raw evidence.
        sha=raw_sha.removeprefix('sha256:') if raw_sha else None
        if sha is not None and not re.fullmatch('[0-9a-f]{64}',sha):sha=None
        if safe_path is None and sha is None:return
        aid=identifier('artifact-reference',record,role,pointer,safe_path,sha)
        insert_exact(self.db,'artifact_refs',
                        (aid,record,role,pointer,safe_path,sha,None,'not_resolved','explicit-artifact-reference-v1'))
        self.edge(record,relation,'artifact_reference',aid,pointer,'explicit-artifact-reference-v1',resolution='reference_record_present')

    def record(self, occurrence: str, path: str, pv: ParsedValue, sens: str,
               *, pointer: str='', kind: str='source_json_value', parent: str | None=None,
               parent_object: dict | None=None) -> str:
        self.record_count+=1
        if self.record_count>self.limits.projection_records:raise FoundationError('PROJECTION_RECORD_COUNT_LIMIT')
        o=pv.value;env,errs=normalize(o,path,pointer)
        rid=identifier('record',occurrence,pv.index,pointer)
        cl=env['classification']
        if kind=='profile_result_projection':
            cl={'category':'profile_execution_projection','context':cl['context'],'method':'exact-structural-parent',
                'confidence':'high','rule':'ares-profile-results-v1','evidence':pointer,'native_schema':None,'native_schema_pointer':None}
            env['classification']=cl
            # Explicit structural membership in a source panel, with parent pointers.
            for target,source_key in [('run_id','run_id'),('revision','runtime_revision'),('workspace','workspace'),('runtime','runtime')]:
                if env['fields'][target]['state']=='unknown' and parent_object is not None and source_key in parent_object:
                    env['fields'][target]=state(parent_object[source_key],'/'+source_key,'panel-parent-field-v1')
        ver=env['native_version']
        self.db.execute('INSERT INTO records VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)',
            (rid,occurrence,pv.index,pointer,pv.start,pv.end,pv.structural_sha256,kind,cl['category'],cl['context'],
             cl['native_schema'],ver['state'],canonical_json(ver['value']) if ver['state']=='source_asserted' else None,
             canonical_json(cl),sens,'denied'))
        for group,key in [('field','fields'),('time','times')]:
            for name,ob in env[key].items():
                if ob['state']=='unknown':continue
                self.db.execute('INSERT INTO fields VALUES(?,?,?,?,?,?,?,?,?)',
                    (rid,group,name,ob['state'],canonical_json(ob['value']) if ob['value'] is not None else None,
                     ob['source_pointer'],ob['rule'],ob.get('role'),ob.get('format_status')))
        if ver['state']!='unknown':
            self.db.execute('INSERT INTO fields VALUES(?,?,?,?,?,?,?,?,?)',
                (rid,'schema','native_version',ver['state'],canonical_json(ver['value']) if ver['value'] is not None else None,ver['source_pointer'],ver['rule'],None,None))
        for err in errs:
            self.anomaly(occurrence,'normalization',err['code'],record=rid,pointer=err.get('pointer'),
                         details={k:v for k,v in err.items() if k not in {'code','pointer'}})
        self.edge(rid,'DERIVED_FROM','occurrence',occurrence,pointer,'raw-evidence-source-v1',resolution='exact',assertion='derived')
        if parent:self.edge(rid,'PART_OF','record',parent,pointer,'structural-containment-v1',target_record=parent,resolution='exact',assertion='derived')
        for field,kind_ref in [('run_id','BELONGS_TO_RUN'),('trace_id','BELONGS_TO_TRACE'),('session_id','BELONGS_TO_SESSION'),('episode_id','BELONGS_TO_EPISODE')]:
            ob=env['fields'][field]
            if ob['state']=='source_asserted' and isinstance(ob['value'],str):
                self.edge(rid,kind_ref,'native_'+field,ob['value'],ob['source_pointer'],'direct-identity-reference-v1',resolution='native_parent_not_observed')
        if isinstance(o,dict):
            if cl['category']=='schema':
                def scalar(k):return o[k] if isinstance(o.get(k),str) and scalar_safe(o[k]) else None
                self.db.execute('INSERT INTO schema_observations VALUES(?,?,?,?,?,?)',
                    (rid,scalar('$id'),scalar('title'),scalar('$schema'),pv.structural_sha256,'not_performed'))
            if cl['native_schema'] in {'AresProfilePanelVerificationV1','AresProfilePanelVerificationV2'}:
                if isinstance(o.get('receipt'),str):self.edge(rid,'VERIFIES','source_path',o['receipt'],pointer+'/receipt','ares-verification-target-v1')
                else:self.anomaly(occurrence,'graph','VERIFICATION_TARGET_MISSING',record=rid,pointer=pointer+'/receipt')
            if kind=='profile_result_projection':
                for stream in ('stdout','stderr'):
                    self.artifact(rid,'output_stream',pointer+'/'+stream+'_sha256',o.get(stream+'_path'),o.get(stream+'_sha256'),relation='PRODUCED')
                self.artifact(rid,'input_manifest',pointer+'/input_manifest_sha256',o.get('input_manifest_path'),o.get('input_manifest_sha256'),relation='CONSUMED')
            for key in ('artifacts','audit_artifacts'):
                if isinstance(o.get(key),list):
                    for i,x in enumerate(o[key]):
                        if isinstance(x,dict):self.artifact(rid,'artifact_reference',pointer+f'/{key}/{i}',x.get('path'),x.get('sha256'))
            if isinstance(o.get('artifact'),dict):self.artifact(rid,'artifact_reference',pointer+'/artifact',o['artifact'].get('path'),o['artifact'].get('sha256'))
            for key in ('supersedes','superseded_by','invalidates','rollback_ref','parent_receipt','policy_ref','permit_ref'):
                value=o.get(key)
                if isinstance(value,str):
                    self.edge(rid,'REFERENCES','unscoped_native_reference',value,pointer+'/'+key,'unscoped-reference-v1',resolution='scope_unknown')
                    if key in {'supersedes','superseded_by','invalidates'}:
                        self.anomaly(occurrence,'graph','GOVERNANCE_REFERENCE_SCOPE_UNKNOWN',record=rid,pointer=pointer+'/'+key,severity='warning')
            for key in ('signature','signatures','attestation','attestation_envelope_id'):
                if key in o and o[key] not in (None,[],{},''):
                    self.db.execute('INSERT INTO signature_observations VALUES(?,?,?,?,?)',(rid,pointer+'/'+key,key,type(o[key]).__name__,'not_performed'))
            if cl['category']=='profile_panel_receipt' and isinstance(o.get('results'),list):
                for i,x in enumerate(o['results']):
                    if isinstance(x,dict):
                        child=ParsedValue(pv.index,pv.start,pv.end,x,structural_digest(x))
                        self.record(occurrence,path,child,sens,pointer=f'/results/{i}',kind='profile_result_projection',parent=rid,parent_object=o)
                    else:self.anomaly(occurrence,'normalization','PROFILE_RESULT_NOT_OBJECT',record=rid,pointer=f'/results/{i}')
        return rid

    def member(self, m: Member, archive_sha: str) -> None:
        self.archive_digest=archive_sha
        occ=identifier('occurrence',archive_sha,m.header_offset,m.path)
        if m.blob_sha256 is not None:
            old=self.db.execute('SELECT byte_length FROM blobs WHERE sha256=?',(m.blob_sha256,)).fetchone()
            if old and old[0]!=m.size:raise FoundationError('BLOB_IDENTITY_COLLISION')
            insert_exact(self.db,'blobs',(m.blob_sha256,m.size))
        # Case folding is a portability warning, not a reason to erase occurrences.
        safety=[c for c in m.issues if c!='PATH_CASEFOLD_COLLISION']
        disposition='safely_rejected' if safety else 'classified_non_data' if m.member_type=='directory' else 'unparsed'
        archive_path=m.path if scalar_safe(m.path) else None
        key=m.path_key if archive_path is not None else None
        self.db.execute('INSERT INTO occurrences VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)',
            (occ,archive_sha,m.ordinal,m.header_offset,m.data_offset,archive_path,key,digest(m.path.encode()),m.member_type,m.size,
             m.header_sha256,m.blob_sha256,disposition,pathlib.PurePosixPath(m.path).suffix.lower(),m.mode,m.mtime_header,0,None,None,None,None,None,None))
        for code in m.issues:self.anomaly(occ,'archive',code,severity='warning' if code=='PATH_CASEFOLD_COLLISION' else 'error')
        if archive_path is None:self.anomaly(occ,'privacy','ARCHIVE_PATH_EXCLUDED_SENSITIVE')
        if m.data is None or safety:return
        findings=scan(m.data);sens=sensitivity(m.data,findings)
        for x in findings:self.anomaly(occ,'privacy','SENSITIVE_CANDIDATE',severity='warning',start=x['byte_start'],end=x['byte_end'],details={'kind':x['kind'],'certainty':'candidate','rule':x['rule']})
        parsed=parse(m.data,m.path,limits=self.limits,recover_streams=self.recover_streams)
        self.db.execute('INSERT INTO parses VALUES(?,?,?,?,?,?)',(occ,parsed.declared,parsed.observed,parsed.status,len(parsed.values),parsed.recovery_mode))
        disposition='quarantined' if parsed.status in {'quarantined','partial_quarantined'} else 'unsupported' if parsed.status=='unsupported' else 'classified_non_data' if parsed.status=='classified_non_data' else 'parsed'
        self.db.execute('UPDATE occurrences SET disposition=? WHERE occurrence_id=?',(disposition,occ))
        for err in parsed.issues:self.anomaly(occ,'parse',err['code'],severity=err['severity'],start=err['byte_start'],end=err['byte_end'])
        for pv in parsed.values:self.record(occ,m.path,pv,sens)
        if m.path_key in {'RECEIPT_COLLECTION_MANIFEST.json','RECEIPT_COLLECTION_CHECKSUMS.sha256','metadata/receipt-location-index.json'}:
            self.meta_sources[m.path_key]=(occ,m.data)

    def collection_integrity(self, required: bool) -> dict:
        report={'schema':'ReceiptCollectionIntegrityV1','manifest_present':False,'checksums_present':False,'index_present':False,'manifest_entries':None,
                'manifest_digest_mismatches':0,'manifest_size_mismatches':0,'manifest_missing_members':0,'manifest_duplicate_entries':0,
                'checksum_entries':0,'checksum_mismatches':0,'index_binding_matches':None,'index_inventory_matches':None,'errors':[]}
        manifest=self.meta_sources.get('RECEIPT_COLLECTION_MANIFEST.json')
        if not manifest:
            if required:
                report['errors'].append('REQUIRED_MANIFEST_MISSING');self.anomaly(None,'integrity','REQUIRED_MANIFEST_MISSING')
            report['status']='FAIL' if required else 'NOT_APPLICABLE';return report
        report['manifest_present']=True;mid,raw=manifest
        try:
            obj=plain(strict_loads(raw,self.limits))
            if not isinstance(obj,dict) or obj.get('schema')!='AresReceiptCollectionManifestV1' or not isinstance(obj.get('entries'),list):raise FoundationError('MANIFEST_CONTRACT_UNSUPPORTED')
            entries=obj['entries'];report['manifest_entries']=len(entries)
            if obj.get('entry_count')!=len(entries):report['errors'].append('MANIFEST_ENTRY_COUNT_MISMATCH')
            seen=set();source_paths=set();total=0
            for i,e in enumerate(entries):
                ptr=f'/entries/{i}'
                if not isinstance(e,dict) or not isinstance(e.get('archive_member'),str) or not isinstance(e.get('sha256'),str) or not re.fullmatch('[0-9a-f]{64}',e['sha256']) or type(e.get('bytes')) is not int or e['bytes']<0:
                    self.anomaly(mid,'integrity','MANIFEST_ENTRY_INVALID',pointer=ptr);report['errors'].append('MANIFEST_ENTRY_INVALID');continue
                key,problems=path_problem(e['archive_member'],self.limits)
                if problems:
                    self.anomaly(mid,'integrity','MANIFEST_MEMBER_PATH_UNSAFE',pointer=ptr);report['errors'].append('MANIFEST_MEMBER_PATH_UNSAFE');continue
                if key in seen:
                    report['manifest_duplicate_entries']+=1;self.anomaly(mid,'integrity','MANIFEST_DUPLICATE_MEMBER',pointer=ptr);continue
                seen.add(key);total+=e['bytes']
                hits=self.db.execute('SELECT occurrence_id,blob_sha256,byte_length FROM occurrences WHERE path_key=? AND member_type=?',(key,'file')).fetchall()
                if len(hits)!=1:
                    report['manifest_missing_members']+=1;self.anomaly(mid,'integrity','MANIFEST_MEMBER_NOT_UNIQUE_OR_MISSING',pointer=ptr);continue
                oid,sha,size=hits[0]
                if sha!=e['sha256']:report['manifest_digest_mismatches']+=1;self.anomaly(oid,'integrity','MANIFEST_DIGEST_MISMATCH',pointer=ptr)
                if size!=e['bytes']:report['manifest_size_mismatches']+=1;self.anomaly(oid,'integrity','MANIFEST_SIZE_MISMATCH',pointer=ptr)
                def safe(k):return e[k] if isinstance(e.get(k),str) and scalar_safe(e[k]) else None
                source=safe('source_path')
                if source is not None and source in source_paths:self.anomaly(mid,'integrity','MANIFEST_SOURCE_PATH_REPEATED',pointer=ptr)
                if source is not None:source_paths.add(source)
                self.db.execute('UPDATE occurrences SET candidate=1,source_path=?,collector_class=?,storage_root=?,filesystem_mtime_asserted=?,manifest_entry_index=?,manifest_occurrence_id=? WHERE occurrence_id=?',
                    (source,safe('evidence_class'),safe('storage_root'),safe('mtime_utc'),i,mid,oid))
            if total!=obj.get('total_source_bytes'):report['errors'].append('MANIFEST_TOTAL_BYTES_MISMATCH')
            # This collector declares all receipt-prefix files as entries.
            unlisted=self.db.execute("SELECT occurrence_id FROM occurrences WHERE member_type='file' AND path_key LIKE 'receipts/%' AND candidate=0").fetchall()
            for (oid,) in unlisted:self.anomaly(oid,'integrity','UNLISTED_RECEIPT_MEMBER')
            report['unlisted_receipt_files']=len(unlisted)
            self.db.execute('INSERT INTO metadata VALUES(?,?)',('collection_time_source_assertion',canonical_json({'value':obj.get('created_utc'),'source_occurrence':mid,'source_pointer':'/created_utc','role':'collection_time_not_event_time'})))
            index=self.meta_sources.get('metadata/receipt-location-index.json')
            if index:
                report['index_present']=True;iid,iraw=index
                expected=obj.get('selection_index',{}).get('sha256')
                report['index_binding_matches']=expected==digest(iraw)
                if not report['index_binding_matches']:self.anomaly(iid,'integrity','INDEX_DIGEST_MISMATCH')
                idx=plain(strict_loads(iraw,self.limits))
                if not isinstance(idx,dict) or idx.get('schema')!='AresReceiptLocationIndexV1':raise FoundationError('INDEX_CONTRACT_UNSUPPORTED')
                declared_count=idx.get('counts',{}).get('all_candidates')
                report['index_inventory_matches']=declared_count==len(entries)
                # Full index-vs-manifest path sets; count equality alone is insufficient.
                artifacts=idx.get('artifacts',{})
                idx_paths=[]
                if isinstance(artifacts,dict):
                    for group in artifacts.values():
                        if isinstance(group,list):
                            for row in group:
                                if isinstance(row,dict) and isinstance(row.get('path'),str):idx_paths.append(row['path'])
                                elif isinstance(row,str):idx_paths.append(row)
                report['index_path_entries']=len(idx_paths)
                report['index_path_set_matches']=set(idx_paths)==source_paths and len(idx_paths)==len(entries)
                if not report['index_path_set_matches']:self.anomaly(iid,'integrity','INDEX_PATH_SET_MISMATCH')
            elif required:report['errors'].append('REQUIRED_INDEX_MISSING')
        except FoundationError as e:
            report['errors'].append(e.code);self.anomaly(mid,'integrity',e.code)
        checksum=self.meta_sources.get('RECEIPT_COLLECTION_CHECKSUMS.sha256')
        if checksum:
            report['checksums_present']=True;cid,craw=checksum;seen=set();start=0
            for line in craw.splitlines(keepends=True):
                stripped=line.rstrip(b'\r\n');match=re.fullmatch(rb'([0-9a-f]{64}) [ *](.+)',stripped)
                if match is None:
                    self.anomaly(cid,'integrity','CHECKSUM_LINE_INVALID',start=start,end=start+len(line));report['errors'].append('CHECKSUM_LINE_INVALID');start+=len(line);continue
                try:name=match[2].decode('utf8','strict')
                except UnicodeDecodeError:
                    self.anomaly(cid,'integrity','CHECKSUM_PATH_UTF8_INVALID',start=start);report['errors'].append('CHECKSUM_PATH_UTF8_INVALID');start+=len(line);continue
                key,problems=path_problem(name,self.limits)
                if problems or key in seen:
                    self.anomaly(cid,'integrity','CHECKSUM_PATH_INVALID_OR_DUPLICATE',start=start);report['errors'].append('CHECKSUM_PATH_INVALID_OR_DUPLICATE');start+=len(line);continue
                seen.add(key);report['checksum_entries']+=1
                rows=self.db.execute('SELECT blob_sha256 FROM occurrences WHERE path_key=? AND member_type=?',(key,'file')).fetchall()
                if len(rows)!=1 or rows[0][0]!=match[1].decode():
                    report['checksum_mismatches']+=1;self.anomaly(cid,'integrity','CHECKSUM_DIGEST_MISMATCH',start=start,end=start+len(line))
                start+=len(line)
            candidate_paths={r[0] for r in self.db.execute('SELECT path_key FROM occurrences WHERE candidate=1')}
            report['checksum_candidate_coverage']=len(candidate_paths & seen)
            if candidate_paths-seen:report['errors'].append('CHECKSUM_CANDIDATE_COVERAGE_GAP')
        elif required:report['errors'].append('REQUIRED_CHECKSUM_FILE_MISSING')
        for code in set(report['errors']):self.anomaly(mid,'integrity',code)
        error_count=self.db.execute("SELECT count(*) FROM anomalies WHERE stage='integrity' AND severity='error'").fetchone()[0]
        report['status']='FAIL' if error_count else 'PASS';return report

    def resolve_graph(self) -> dict:
        db=self.db
        for eid,rid,path,pointer in db.execute("SELECT edge_id,source_record_id,target_value,source_pointer FROM edges WHERE kind='VERIFIES' AND target_type='source_path'").fetchall():
            targets=db.execute("SELECT r.record_id FROM records r JOIN occurrences o USING(occurrence_id) WHERE o.source_path=? AND r.projection_kind='source_json_value' AND r.category='profile_panel_receipt'",(path,)).fetchall()
            if len(targets)==1:
                db.execute('UPDATE edges SET target_record_id=?,resolution=? WHERE edge_id=?',(targets[0][0],'exact_source_path_source_assertion',eid))
            else:
                db.execute('UPDATE edges SET resolution=? WHERE edge_id=?',('target_not_in_collection' if not targets else 'ambiguous_target',eid))
                oid=db.execute('SELECT occurrence_id FROM records WHERE record_id=?',(rid,)).fetchone()[0]
                self.anomaly(oid,'graph','VERIFICATION_TARGET_NOT_UNIQUE_OR_MISSING',record=rid,pointer=pointer,details={'target_count':len(targets)})
        for aid,rid,sha in db.execute('SELECT artifact_ref_id,source_record_id,sha256_asserted FROM artifact_refs').fetchall():
            exists=sha is not None and db.execute('SELECT 1 FROM blobs WHERE sha256=?',(sha,)).fetchone() is not None
            db.execute('UPDATE artifact_refs SET matched_blob_sha256=?,resolution=? WHERE artifact_ref_id=?',(sha if exists else None,'exact_digest_match' if exists else 'referenced_bytes_not_in_collection',aid))
            if not exists:
                oid=db.execute('SELECT occurrence_id FROM records WHERE record_id=?',(rid,)).fetchone()[0]
                self.anomaly(oid,'graph','ARTIFACT_BYTES_NOT_IN_COLLECTION',record=rid,severity='warning',details={'artifact_ref_id':aid})
        # Native IDs are compared for anomalies but never used for global joins.
        native=collections.defaultdict(list)
        for rid,value,schema,struct,oid in db.execute("SELECT f.record_id,f.value_json,r.native_schema,r.structural_sha256,r.occurrence_id FROM fields f JOIN records r USING(record_id) WHERE f.name='native_id' AND f.state='source_asserted'"):
            native[value].append((rid,schema,struct,oid))
        for values in native.values():
            schemas={v[1] for v in values}
            code='NATIVE_ID_SCHEMA_REUSE' if len(schemas)>1 else 'NATIVE_ID_CONTENT_CONFLICT' if len({v[2] for v in values})>1 else None
            if code:
                for rid,_,_,oid in values:self.anomaly(oid,'graph',code,record=rid,severity='warning',details={'occurrences':len(values)})
        # Strong resolved record edges are typed. Structural containment must be acyclic.
        adjacency=collections.defaultdict(list)
        for a,b in db.execute("SELECT source_record_id,target_record_id FROM edges WHERE kind='PART_OF' AND target_record_id IS NOT NULL"):
            adjacency[a].append(b)
        visited=set();active=set()
        def visit(node):
            if node in active:raise FoundationError('STRUCTURAL_PROVENANCE_CYCLE')
            if node in visited:return
            active.add(node)
            for parent in adjacency[node]:visit(parent)
            active.remove(node);visited.add(node)
        for node in list(adjacency):visit(node)
        fk=list(db.execute('PRAGMA foreign_key_check'))
        if fk:raise FoundationError('PROJECTION_DANGLING_FOREIGN_KEY')
        bad=db.execute('SELECT count(*) FROM records r JOIN occurrences o USING(occurrence_id) WHERE r.byte_start<0 OR r.byte_end>o.byte_length OR o.blob_sha256 IS NULL').fetchone()[0]
        if bad:raise FoundationError('PROVENANCE_BYTE_RANGE_INVALID')
        return {'schema':'ReceiptGraphValidationV1','foreign_key_errors':len(fk),'invalid_byte_ranges':bad,'structural_cycles':0,
                'native_scope_policy':'no_global_native_id_joins','verification_targets':dict(db.execute("SELECT resolution,count(*) FROM edges WHERE kind='VERIFIES' GROUP BY resolution")),
                'artifact_resolution':dict(db.execute('SELECT resolution,count(*) FROM artifact_refs GROUP BY resolution')),
                'supersession_cycle_validation':'blocked_for_unscoped_references_not_silently_repaired'}


def build(archive_path: pathlib.Path, output: pathlib.Path, *, recover_streams: bool=False,
          compression: str='zstd', limits: Limits=Limits(), require_collection_manifest: bool=True,
          expected_archive_sha256: str | None=None) -> dict:
    """Create-only, transactional staging; a failed build never appears at output."""
    implementation_digest=code_digest()
    start=time.monotonic();imported_at=dt.datetime.now(dt.timezone.utc).isoformat()
    if output.exists() or output.is_symlink():raise FoundationError('OUTPUT_ALREADY_EXISTS')
    if not output.parent.is_dir():raise FoundationError('OUTPUT_PARENT_MISSING')
    fd,tmp=tempfile.mkstemp(prefix='.receipt-staging-',suffix='.sqlite',dir=output.parent)
    os.close(fd);os.chmod(tmp,0o600);temp=pathlib.Path(tmp)
    db=sqlite3.connect(temp);db.row_factory=sqlite3.Row
    arch=Archive(archive_path,limits,compression=compression)
    try:
        page_size=db.execute('PRAGMA page_size').fetchone()[0]
        if limits.projection_bytes<page_size:raise FoundationError('PROJECTION_SIZE_LIMIT')
        db.execute(f'PRAGMA max_page_count={limits.projection_bytes//page_size}')
        db.executescript(SQL);db.execute(f'PRAGMA application_id={APPLICATION_ID}');db.execute(f'PRAGMA user_version={SCHEMA_VERSION}')
        db.execute('PRAGMA synchronous=FULL');db.execute('PRAGMA journal_mode=DELETE');db.execute('PRAGMA temp_store=FILE')
        b=Builder(db,recover_streams=recover_streams,limits=limits)
        config={'recover_streams':recover_streams,'compression':compression,'limits':dataclasses.asdict(limits),'require_collection_manifest':require_collection_manifest}
        for k,v in {'projection_kind':'rebuildable_observation_only','implementation_version':VERSION,'implementation_sha256':implementation_digest,
                    'classification_rules':RULES_VERSION,'schema_version':SCHEMA_VERSION,'config':config,'structural_digest_algorithm':'json-lexeme-tree-v1','export_policy':'deny-all-v1'}.items():
            db.execute('INSERT INTO metadata VALUES(?,?)',(k,canonical_json(v)))
        initialized=False
        for member in arch.members():
            if not initialized:
                if expected_archive_sha256 and arch.sha256!=expected_archive_sha256:raise FoundationError('ARCHIVE_DIGEST_EXPECTATION_MISMATCH')
                db.execute('INSERT INTO collections VALUES(?,?)',(arch.sha256,arch.bytes));initialized=True
            b.member(member,arch.sha256)
        if not initialized:
            if expected_archive_sha256 and arch.sha256!=expected_archive_sha256:raise FoundationError('ARCHIVE_DIGEST_EXPECTATION_MISMATCH')
            db.execute('INSERT INTO collections VALUES(?,?)',(arch.sha256,arch.bytes))
        for row in arch.reader.controls:
            db.execute('INSERT INTO archive_controls VALUES(?,?,?,?,?,?)',(arch.sha256,row['header_offset'],row['kind'],row['header_sha256'],row['payload_sha256'],row['payload_bytes']))
        integrity=b.collection_integrity(require_collection_manifest)
        graph=b.resolve_graph()
        db.commit()
        if db.execute('PRAGMA integrity_check').fetchone()[0]!='ok':raise FoundationError('SQLITE_INTEGRITY_FAILURE')
        manifest=logical_manifest(db)
        summary=summarize(db)
        if code_digest()!=implementation_digest:raise FoundationError('IMPLEMENTATION_CHANGED_DURING_BUILD')
        db.close()
        # Atomic publish without replace. Concurrent writers cannot overwrite a winner.
        try:os.link(temp,output)
        except FileExistsError as e:raise FoundationError('OUTPUT_ALREADY_EXISTS') from e
        except OSError as e:raise FoundationError('ATOMIC_PUBLICATION_FAILURE') from e
        temp.unlink()
        return {'schema':'ReceiptProjectionBuildV1','state':'built_with_integrity_failures' if integrity['status']=='FAIL' else 'built',
                'archive_sha256':arch.sha256,'archive_bytes':arch.bytes,'source_preservation':'rehash_matched_after_read',
                'implementation_sha256':implementation_digest,'imported_at':imported_at,'runtime_seconds':round(time.monotonic()-start,6),
                'peak_rss_kib':resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,'projection_bytes':output.stat().st_size,
                'archive_safety':arch.receipt(),'collection_integrity':integrity,'graph_validation':graph,
                'logical_manifest':manifest,'summary':summary,'ci_status':'not_run'}
    except BaseException as exc:
        original=exc
        if isinstance(exc,sqlite3.Error):
            exc=FoundationError('PROJECTION_SIZE_LIMIT' if getattr(exc,'sqlite_errorname','')=='SQLITE_FULL' else 'SQLITE_WRITE_FAILURE')
        if isinstance(exc,FoundationError):
            partial=arch.receipt()
            partial['source_modified']='not_certified_on_failed_read'
            partial['accounting_state']='partial_archive_rejected_no_projection_published'
            exc.context={'archive_safety_partial':partial}
        db.close()
        if temp.exists():temp.unlink()
        journal=pathlib.Path(str(temp)+'-journal')
        if journal.exists():journal.unlink()
        if exc is not original:raise exc from original
        raise


def summarize(db: sqlite3.Connection) -> dict:
    def one(sql):return db.execute(sql).fetchone()[0]
    def count(sql):return dict(db.execute(sql))
    candidates=one('SELECT count(*) FROM occurrences WHERE candidate=1')
    unique=one('SELECT count(DISTINCT blob_sha256) FROM occurrences WHERE candidate=1')
    summary={'total_members':one('SELECT count(*) FROM occurrences'),'regular_files':one("SELECT count(*) FROM occurrences WHERE member_type='file'"),
             'directories':one("SELECT count(*) FROM occurrences WHERE member_type='directory'"),'physical_control_headers':one('SELECT count(*) FROM archive_controls'),
             'candidate_receipts':candidates,'candidate_raw_bytes':one('SELECT coalesce(sum(byte_length),0) FROM occurrences WHERE candidate=1'),
             'candidate_unique_blobs':unique,'candidate_excess_duplicate_occurrences':candidates-unique,
             'all_unique_blobs':one('SELECT count(*) FROM blobs'),'records':one('SELECT count(*) FROM records'),
             'top_level_values':one("SELECT count(*) FROM records WHERE projection_kind='source_json_value'"),
             'member_dispositions':count('SELECT disposition,count(*) FROM occurrences GROUP BY disposition'),
             'candidate_dispositions':count('SELECT disposition,count(*) FROM occurrences WHERE candidate=1 GROUP BY disposition'),
             'candidate_extensions':count('SELECT extension,count(*) FROM occurrences WHERE candidate=1 GROUP BY extension'),
             'candidate_serialization':count('SELECT p.observed_format,count(*) FROM parses p JOIN occurrences o USING(occurrence_id) WHERE o.candidate=1 GROUP BY p.observed_format'),
             'collector_asserted_classes':count('SELECT collector_class,count(*) FROM occurrences WHERE candidate=1 GROUP BY collector_class'),
             'record_classification':count('SELECT category,count(*) FROM records GROUP BY category'),
             'anomalies':count('SELECT code,count(*) FROM anomalies GROUP BY code'),
             'record_sensitivity':count('SELECT sensitivity,count(*) FROM records GROUP BY sensitivity'),
             'export_allowed_records':one("SELECT count(*) FROM records WHERE export_eligibility!='denied'"),
             'schema_document_occurrences':one('SELECT count(*) FROM schema_observations'),
             'schema_distinct_structures':one('SELECT count(DISTINCT structural_sha256) FROM schema_observations'),
             'explicit_native_version_records':one("SELECT count(*) FROM records WHERE native_version_state='source_asserted'"),
             'native_schema_present_records':one('SELECT count(*) FROM records WHERE native_schema IS NOT NULL'),
             'time_field_coverage':count("SELECT name,count(*) FROM fields WHERE field_group='time' GROUP BY name"),
             'field_coverage':count("SELECT name,count(*) FROM fields WHERE field_group='field' AND state='source_asserted' GROUP BY name"),
             'attestation_envelope_reference_occurrences':one("SELECT count(*) FROM signature_observations WHERE field_name='attestation_envelope_id'"),
             'signature_material_field_candidates':one("SELECT count(*) FROM signature_observations WHERE field_name IN ('signature','signatures','attestation')"),
             'cryptographically_verified_by_importer':0,
             'unaccounted_members':one("SELECT count(*) FROM occurrences WHERE disposition='unparsed'")}
    # Reproduce the supplied canonical path cohort without assuming collector trust.
    prefix='receipts/.ares/profile-collaboration/receipts/'
    cohort=f"substr(o.path_key,1,{len(prefix)})='{prefix}' COLLATE BINARY"
    base=f'FROM records r JOIN occurrences o USING(occurrence_id) WHERE {cohort}'
    summary['profile_cohort']={'definition':'exact case-sensitive archive prefix receipts/.ares/profile-collaboration/receipts/',
        'panels':one(f"SELECT count(*) {base} AND r.category='profile_panel_receipt'"),
        'verification_receipts':one(f"SELECT count(*) {base} AND r.category='verification_receipt'"),
        'executions':one(f"SELECT count(*) {base} AND r.projection_kind='profile_result_projection'"),
        'runtime_revisions':one(f"SELECT count(DISTINCT f.value_json) FROM fields f JOIN records r USING(record_id) JOIN occurrences o USING(occurrence_id) WHERE {cohort} AND r.category='profile_panel_receipt' AND f.name='revision' AND f.state='source_asserted'"),
        'workspaces':one(f"SELECT count(DISTINCT f.value_json) FROM fields f JOIN records r USING(record_id) JOIN occurrences o USING(occurrence_id) WHERE {cohort} AND r.category='profile_panel_receipt' AND f.name='workspace' AND f.state='source_asserted'"),
        'execution_outcomes':{json.loads(k):v for k,v in db.execute(f"SELECT f.value_json,count(*) FROM fields f JOIN records r USING(record_id) JOIN occurrences o USING(occurrence_id) WHERE {cohort} AND r.projection_kind='profile_result_projection' AND f.name='outcome' AND f.state='source_asserted' GROUP BY f.value_json")},
        'verification_states':{json.loads(k):v for k,v in db.execute(f"SELECT f.value_json,count(*) FROM fields f JOIN records r USING(record_id) JOIN occurrences o USING(occurrence_id) WHERE {cohort} AND r.category='verification_receipt' AND f.name='verification_state' AND f.state='source_asserted' GROUP BY f.value_json")},
        'controller_verified_source_true':one(f"SELECT count(*) FROM fields f JOIN records r USING(record_id) JOIN occurrences o USING(occurrence_id) WHERE {cohort} AND r.category='verification_receipt' AND f.name='controller_verified' AND f.value_json='true'")}
    return summary
