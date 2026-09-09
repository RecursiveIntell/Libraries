"""Rebuild private inventory/query witnesses from a projection; does not mutate it."""
from __future__ import annotations
import argparse, collections, json, pathlib
from receipt_foundation.common import digest, write_json
from receipt_foundation.projection import connect_readonly, summarize, logical_manifest
from receipt_foundation.query import query, source_bytes

def audit(database:pathlib.Path, output:pathlib.Path, archive:pathlib.Path|None=None) -> dict:
    output.mkdir(mode=0o700,parents=False,exist_ok=False)
    with connect_readonly(database) as db:
        summary=summarize(db)
        members=[]
        for row in db.execute('SELECT o.*,p.declared_format,p.observed_format,p.status AS parse_status,p.value_count,p.recovery_mode FROM occurrences o LEFT JOIN parses p USING(occurrence_id) ORDER BY o.ordinal'):
            m=dict(row)
            m['classifications']=[json.loads(r[0]) for r in db.execute('SELECT DISTINCT classification_json FROM records WHERE occurrence_id=? ORDER BY classification_json',(m['occurrence_id'],))]
            m['issues']=[dict(r) for r in db.execute('SELECT code,stage,severity,byte_start,byte_end,recoverable FROM anomalies WHERE occurrence_id=? ORDER BY anomaly_id',(m['occurrence_id'],))]
            m['parse_attempt']='not_applicable_directory' if m['member_type']=='directory' else 'not_applicable_or_rejected' if m['parse_status'] is None else 'performed'
            members.append(m)
        inventory={'schema':'ReceiptCorpusInventoryV1','derived_only':True,'sensitivity':'private_internal_unreviewed','logical_manifest':logical_manifest(db),'summary':summary,'members':members,'archive_controls':[dict(r) for r in db.execute('SELECT * FROM archive_controls ORDER BY header_offset')]}
        write_json(output/'CORPUS_INVENTORY.json',inventory)
        write_json(output/'CORPUS_SUMMARY.json',summary)
        schemas=[dict(r) for r in db.execute('SELECT native_schema,native_version_state,native_version_json,count(*) AS records FROM records GROUP BY native_schema,native_version_state,native_version_json ORDER BY native_schema,native_version_state,native_version_json')]
        conflicts=[dict(r) for r in db.execute('SELECT schema_id,count(DISTINCT structural_sha256) AS definitions,count(*) AS occurrences FROM schema_observations WHERE schema_id IS NOT NULL GROUP BY schema_id HAVING count(DISTINCT structural_sha256)>1 ORDER BY schema_id')]
        schema_docs=[dict(r) for r in db.execute('SELECT s.*,r.occurrence_id,o.blob_sha256 FROM schema_observations s JOIN records r USING(record_id) JOIN occurrences o USING(occurrence_id) ORDER BY s.record_id')]
        write_json(output/'SCHEMA_GENEALOGY.json',{'schema':'ReceiptSchemaGenealogyV1','observed_family_versions':schemas,'schema_id_conflicts':conflicts,'schema_documents':schema_docs,'semantic_equivalence_not_claimed':True})
        anomalies=[dict(r) for r in db.execute('SELECT * FROM anomalies ORDER BY anomaly_id')]
        write_json(output/'CORPUS_ANOMALIES.json',{'schema':'ReceiptCorpusAnomaliesV1','count':len(anomalies),'rows':anomalies,'not_automatically_repaired':True})
        exemplar=dict(db.execute("SELECT r.record_id,r.occurrence_id,o.blob_sha256 FROM records r JOIN occurrences o USING(occurrence_id) WHERE r.category='profile_panel_receipt' ORDER BY record_id LIMIT 1").fetchone())
        vals={'digest':exemplar['blob_sha256'],'occurrences':exemplar['blob_sha256'],'occurrence':exemplar['occurrence_id'],'provenance':exemplar['record_id'],'family':'AresProfilePanelReceiptV2','quarantine':'EMPTY_DOCUMENT'}
        for kind,name in [('run','run_id'),('revision','revision'),('trace','trace_id'),('session','session_id')]:
            vals[kind]=json.loads(db.execute("SELECT value_json FROM fields WHERE name=? AND state='source_asserted' ORDER BY record_id LIMIT 1",(name,)).fetchone()[0])
        result=[]
        for kind in ('records','digest','occurrence','occurrences','family','revision','run','trace','session','verification','artifacts','quarantine','duplicates','provenance','schemas','times','verification-coverage'):
            got=query(db,kind,vals.get(kind),limit=3)
            result.append({'kind':kind,'value':vals.get(kind),'rows_returned':len(got['rows']),'next_offset':got['next_offset'],'status':'PASS' if got['rows'] else 'FAIL','rows':got['rows']})
        child=db.execute("SELECT p.source_record_id FROM edges p JOIN edges v ON v.target_record_id=p.target_record_id AND v.kind='VERIFIES' WHERE p.kind='PART_OF' ORDER BY p.source_record_id LIMIT 1").fetchone()[0]
        child_result=query(db,'verification',child,limit=3)
        result.append({'kind':'verification-via-structural-parent','value':child,'rows_returned':len(child_result['rows']),'rows':child_result['rows'],'status':'PASS' if child_result['rows'] else 'FAIL','target_scope':'panel verification applies to containing panel; not independent per-execution certification'})
        proof={'schema':'ReceiptQueryProofV1','status':'PASS' if all(r['status']=='PASS' for r in result) else 'FAIL','checks':result}
        write_json(output/'QUERY_PROOF.json',proof)
        write_json(output/'GRAPH_COVERAGE.json',{'schema':'ReceiptGraphCoverageV1','edges':[dict(r) for r in db.execute('SELECT kind,resolution,count(*) AS count FROM edges GROUP BY kind,resolution ORDER BY kind,resolution')],'foreign_key_errors':[list(r) for r in db.execute('PRAGMA foreign_key_check')],'scope':'No native scope inferred; external parents unresolved explicitly.'})
    if archive:
        raw_path=output/'retrieved-private-source.json'
        retrieval=source_bytes(database,archive,exemplar['record_id'],raw_path,acknowledge_private=True)
        retrieval['written_file_deleted_after_byte_witness']=True
        raw_path.unlink()  # Only this script's newly created private retrieval copy; never source evidence.
        write_json(output/'SOURCE_RETRIEVAL_PROOF.json',retrieval)
    return {'schema':'ReceiptAuditArtifactsV1','query_status':proof['status'],'member_count':len(members),'output_files':len(list(output.iterdir()))}

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('database',type=pathlib.Path);p.add_argument('--output-dir',type=pathlib.Path,required=True);p.add_argument('--archive',type=pathlib.Path)
    a=p.parse_args();print(json.dumps(audit(a.database,a.output_dir,a.archive),sort_keys=True))
