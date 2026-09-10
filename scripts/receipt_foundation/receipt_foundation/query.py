"""Bounded read-only queries and explicit source-byte retrieval."""
from __future__ import annotations
import json, pathlib, sqlite3
from .archive import Archive
from .common import FoundationError, canonical_json, digest, private_write
from .projection import connect_readonly, logical_manifest, summarize

KINDS=('records','digest','occurrence','occurrences','family','revision','run','trace','session','episode','task','goal','verification','artifacts','quarantine','duplicates','provenance','schemas','times','verification-coverage')


def query(db:sqlite3.Connection, kind:str, value:str|None=None, *, limit:int=20, offset:int=0, private:bool=False) -> dict:
    if kind not in KINDS:raise FoundationError('QUERY_KIND_UNSUPPORTED')
    if not 1<=limit<=1000 or not 0<=offset<=1000000:raise FoundationError('QUERY_BOUND_INVALID')
    record_cols='r.record_id,r.occurrence_id,r.projection_kind,r.category,r.native_schema,r.sensitivity,r.export_eligibility'
    if private:record_cols+=',o.archive_member,o.source_path'
    records=f'SELECT {record_cols} FROM records r JOIN occurrences o USING(occurrence_id)'
    args=[];sql=''
    if kind=='records':sql=records+' ORDER BY r.record_id'
    elif kind in {'digest','occurrence','occurrences'}:
        if value is None:raise FoundationError('QUERY_VALUE_REQUIRED')
        cols='occurrence_id,blob_sha256,member_type,byte_length,disposition,candidate'
        if private:cols+=',archive_member,source_path,archive_sha256,header_offset'
        column='occurrence_id' if kind=='occurrence' else 'blob_sha256'
        sql=f'SELECT {cols} FROM occurrences WHERE {column}=? ORDER BY occurrence_id';args=[value if kind=='occurrence' else value.removeprefix('sha256:')]
    elif kind=='family':
        if value is None:raise FoundationError('QUERY_VALUE_REQUIRED')
        sql=records+' WHERE r.native_schema=? OR r.category=? ORDER BY r.record_id';args=[value,value]
    elif kind in {'revision','run','trace','session','episode','task','goal'}:
        if value is None:raise FoundationError('QUERY_VALUE_REQUIRED')
        field={'revision':'revision','run':'run_id','trace':'trace_id','session':'session_id','episode':'episode_id','task':'task_id','goal':'goal_id'}[kind]
        sql=records+" JOIN fields f USING(record_id) WHERE f.field_group='field' AND f.name=? AND f.state='source_asserted' AND f.value_json=? ORDER BY r.record_id";args=[field,canonical_json(value)]
    elif kind=='verification':
        sql='SELECT e.edge_id,e.source_record_id AS verification_record_id,e.target_record_id AS target_record_id,e.resolution,e.assertion_state FROM edges e WHERE e.kind=?';args=['VERIFIES']
        if value:
            sql+=" AND (e.target_record_id=? OR e.source_record_id=? OR EXISTS (SELECT 1 FROM edges p WHERE p.kind='PART_OF' AND p.source_record_id=? AND p.target_record_id=e.target_record_id))"
            args += [value,value,value]
        sql+=' ORDER BY e.edge_id'
    elif kind=='artifacts':
        cols='a.artifact_ref_id,a.source_record_id,a.role,a.sha256_asserted,a.matched_blob_sha256,a.resolution'
        if private:cols+=',a.path_asserted'
        sql=f'SELECT {cols} FROM artifact_refs a'
        if value:
            sql+=" JOIN fields f ON f.record_id=a.source_record_id WHERE f.name='run_id' AND f.state='source_asserted' AND f.value_json=?";args=[canonical_json(value)]
        sql+=' ORDER BY a.artifact_ref_id'
    elif kind=='quarantine':
        sql='SELECT anomaly_id,occurrence_id,record_id,stage,code,severity,byte_start,byte_end,recoverable FROM anomalies'
        if value:sql+=' WHERE code=?';args=[value]
        sql+=' ORDER BY anomaly_id'
    elif kind=='duplicates':
        sql='SELECT blob_sha256,count(*) AS occurrences,sum(candidate) AS candidate_occurrences FROM occurrences WHERE blob_sha256 IS NOT NULL GROUP BY blob_sha256 HAVING count(*)>1 ORDER BY occurrences DESC,blob_sha256'
    elif kind=='provenance':
        if value is None:raise FoundationError('QUERY_VALUE_REQUIRED')
        cols='r.record_id,r.occurrence_id,r.value_index,r.pointer,r.byte_start,r.byte_end,o.blob_sha256,o.archive_sha256,o.header_offset,o.data_offset,o.byte_length,o.path_sha256,o.manifest_occurrence_id,o.manifest_entry_index'
        if private:cols+=',o.archive_member,o.source_path,o.filesystem_mtime_asserted,o.tar_mtime_raw'
        sql=f'SELECT {cols} FROM records r JOIN occurrences o USING(occurrence_id) WHERE r.record_id=?';args=[value]
    elif kind=='schemas':
        sql="SELECT native_schema,native_version_state,native_version_json,count(*) AS records FROM records WHERE projection_kind='source_json_value' GROUP BY native_schema,native_version_state,native_version_json ORDER BY records DESC,native_schema,native_version_json"
    elif kind=='times':
        sql="SELECT name,state,role,format_status,count(*) AS records FROM fields WHERE field_group='time' GROUP BY name,state,role,format_status ORDER BY name,state,format_status"
    elif kind=='verification-coverage':
        sql="SELECT state,value_json,count(*) AS records FROM fields WHERE field_group='field' AND name='verification_state' GROUP BY state,value_json ORDER BY records DESC,value_json"
    rows=[dict(r) for r in db.execute(sql+' LIMIT ? OFFSET ?',args+[limit+1,offset])]
    more=len(rows)>limit
    return {'schema':'ReceiptObservationQueryV1','kind':kind,'rows':rows[:limit],'next_offset':offset+limit if more else None,
            'private_metadata_included':private,'truth_status':'source_assertions_and_derived_observations_only','raw_content_included':False}


def source_bytes(db_path:pathlib.Path, archive_path:pathlib.Path, record_id:str, output:pathlib.Path, *, acknowledge_private:bool=False, compression:str='zstd') -> dict:
    if not acknowledge_private:raise FoundationError('PRIVATE_CONTENT_ACK_REQUIRED')
    with connect_readonly(db_path) as db:
        row=db.execute('SELECT r.*,o.header_offset,o.archive_sha256,o.blob_sha256 FROM records r JOIN occurrences o USING(occurrence_id) WHERE r.record_id=?',(record_id,)).fetchone()
        if row is None:raise FoundationError('RECORD_NOT_FOUND')
        expected=dict(row)
    found=None;arch=Archive(archive_path,compression=compression)
    for m in arch.members():
        if arch.sha256!=expected['archive_sha256']:raise FoundationError('ARCHIVE_DIGEST_EXPECTATION_MISMATCH')
        if m.header_offset==expected['header_offset']:
            if m.blob_sha256!=expected['blob_sha256'] or m.data is None:raise FoundationError('SOURCE_BLOB_MISMATCH')
            a,b=expected['byte_start'],expected['byte_end']
            if not 0<=a<=b<=len(m.data):raise FoundationError('PROVENANCE_BYTE_RANGE_INVALID')
            found=m.data[a:b]
    if found is None:raise FoundationError('SOURCE_MEMBER_NOT_FOUND')
    private_write(output,found)
    return {'schema':'ReceiptSourceRetrievalV1','record_id':record_id,'archive_sha256':arch.sha256,
            'blob_sha256':expected['blob_sha256'],'byte_start':expected['byte_start'],'byte_end':expected['byte_end'],
            'json_pointer_within_source_value':expected['pointer'],'returned_bytes':len(found),'returned_sha256':digest(found),
            'representation':'exact_original_json_value_bytes_not_reconstructed','sensitivity':'private_unreviewed','source_modified':False}
