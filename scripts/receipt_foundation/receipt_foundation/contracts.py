"""Single-source observation-wire schemas. Never native receipt/authority schemas.

JSON Schemas are generated from this module; generated schema files must not be
edited independently. Unknown historical values remain tagged observations.
"""
from __future__ import annotations
import json, pathlib
from .common import FoundationError, Limits, canonical_json, identifier, open_source, write_json
from .normalize import COMMON_FIELDS, TIME_KEYS
from .serialization import plain, strict_loads

HASH={'type':'string','pattern':'^[0-9a-f]{64}$'}
STRING={'type':'string'}
NULL_STRING={'type':['string','null']}
INT={'type':'integer','minimum':0}
NULL_HASH={'anyOf':[HASH,{'type':'null'}]}
OBS_STATES=['unknown','source_null','source_asserted','redacted','unsupported_type','ambiguous']

def closed(properties:dict,required:list|None=None)->dict:
    return {'type':'object','properties':properties,'required':list(properties) if required is None else required,'additionalProperties':False}

def schema(name:str,properties:dict)->dict:
    return {'$schema':'http://json-schema.org/draft-07/schema#','$id':'urn:recursiveintell:receipt-observation:'+name+(':v2' if name.endswith('V2') else ':v1'),
            'title':name,**closed(properties)}

def observation(*,time:bool=False)->dict:
    props={'state':{'enum':OBS_STATES},'value':{'anyOf':[{'type':['string','integer','boolean','null']},closed({'numeric_lexeme':STRING})]},
           'source_pointer':NULL_STRING,'rule':STRING}
    if time:props.update(role=STRING,format_status=STRING)
    out=closed(props)
    out['allOf']=[{'if':{'properties':{'state':{'enum':['unknown','source_null','redacted','unsupported_type','ambiguous']}}},'then':{'properties':{'value':{'type':'null'}}}},
                  {'if':{'properties':{'state':{'const':'unknown'}}},'then':{'properties':{'source_pointer':{'type':'null'},'rule':{'const':'absent-v1'}}}},
                  {'if':{'properties':{'state':{'const':'source_asserted'}}},'then':{'properties':{'source_pointer':{'type':'string'},'value':{'not':{'type':'null'}}}}}]
    return out

def schemas()->dict[str,dict]:
    provenance=closed({'archive_sha256':HASH,'occurrence_id':HASH,'blob_sha256':HASH,'header_offset':INT,
                       'byte_start':INT,'byte_end':INT,'value_index':INT,'json_pointer':STRING})
    classification=closed({'category':STRING,'context':STRING,'method':STRING,'confidence':{'enum':['high','medium','unknown']},
                           'rule':STRING,'evidence':NULL_STRING,'native_schema':NULL_STRING,'native_schema_pointer':NULL_STRING})
    result={
      'receipt-blob-v1.schema.json':schema('ReceiptObservationBlobV1',{'schema':{'const':'ReceiptObservationBlobV1'},'digest_algorithm':{'const':'sha256'},'sha256':HASH,'byte_length':INT}),
      'receipt-occurrence-v1.schema.json':schema('ReceiptObservationOccurrenceV1',{'schema':{'const':'ReceiptObservationOccurrenceV1'},'occurrence_id':HASH,
          'archive_sha256':HASH,'ordinal':INT,'header_offset':INT,'data_offset':INT,'archive_member':NULL_STRING,'path_key':NULL_STRING,'path_sha256':HASH,
          'member_type':{'enum':['file','directory','symlink','hardlink','character_device','block_device','fifo','unsupported']},'byte_length':INT,'header_sha256':HASH,'blob_sha256':NULL_HASH,
          'disposition':{'enum':['parsed','quarantined','unsupported','classified_non_data','safely_rejected','access_failure']},'extension':STRING,'mode':INT,'tar_mtime_raw':STRING,
          'candidate':{'type':'integer','enum':[0,1]},'source_path':NULL_STRING,'collector_class':NULL_STRING,'storage_root':NULL_STRING,'filesystem_mtime_asserted':NULL_STRING,
          'manifest_entry_index':{'type':['integer','null'],'minimum':0},'manifest_occurrence_id':NULL_HASH}),
      'receipt-envelope-v2.schema.json':schema('ReceiptObservationEnvelopeV2',{'schema':{'const':'ReceiptObservationEnvelopeV2'},'record_id':HASH,'provenance':provenance,
          'projection_kind':{'enum':['source_json_value','profile_result_projection']},'structural_sha256':HASH,'structural_algorithm':{'const':'json-lexeme-tree-v1'},
          'classification':classification,'native_version':observation(),'fields':closed({name:observation() for name in COMMON_FIELDS}),
          'times':closed({name:observation(time=True) for name in TIME_KEYS},required=[]),
          'sensitivity':{'enum':['public-safe','internal','sensitive','secret/credential','unknown']},'export_eligibility':{'const':'denied'},
          'semantic_validation':{'const':'not_performed'}}),
      'receipt-edge-v2.schema.json':schema('ReceiptObservationEdgeV2',{'schema':{'const':'ReceiptObservationEdgeV2'},'edge_id':HASH,'source_record_id':HASH,
          'kind':{'enum':['DERIVED_FROM','PART_OF','BELONGS_TO_RUN','BELONGS_TO_TRACE','BELONGS_TO_SESSION','BELONGS_TO_EPISODE','VERIFIES','PRODUCED','CONSUMED','REFERENCES']},
          'target_type':STRING,'target_value':STRING,'target_record_id':NULL_HASH,'resolution':STRING,'source_pointer':STRING,'rule':STRING,
          'assertion_state':{'enum':['derived','source_asserted']}}),
      'quarantine-record-v1.schema.json':schema('ReceiptObservationAnomalyV1',{'schema':{'const':'ReceiptObservationAnomalyV1'},'anomaly_id':HASH,
          'occurrence_id':NULL_HASH,'record_id':NULL_HASH,'stage':STRING,'code':STRING,'severity':{'enum':['error','warning']},
          'byte_start':{'type':['integer','null'],'minimum':0},'byte_end':{'type':['integer','null'],'minimum':0},'source_pointer':NULL_STRING,
          'details_json':STRING,'recoverable':{'type':'integer','enum':[0,1]}}),
      'ingestion-request-v1.schema.json':schema('ReceiptObservationRequestV1',{'schema':{'const':'ReceiptObservationRequestV1'},
          'archive_sha256':HASH,'compression':{'enum':['zstd','tar']},'parser_mode':{'enum':['strict-v1','explicit-json-streams-v1']},
          'collection_contract':{'enum':['AresReceiptCollectionManifestV1','generic-archive-v1']},
          'source_handle':{'type':'string','pattern':'^[A-Za-z0-9_.-]{1,128}$'},'authority_effect':{'const':'none'},'network_fetch':{'const':False}})
    }
    return result

def envelope(db,record_id:str)->dict:
    r=db.execute('SELECT r.*,o.archive_sha256,o.blob_sha256,o.header_offset FROM records r JOIN occurrences o USING(occurrence_id) WHERE record_id=?',(record_id,)).fetchone()
    if r is None:raise FoundationError('RECORD_NOT_FOUND')
    out={'schema':'ReceiptObservationEnvelopeV2','record_id':record_id,
      'provenance':{k:r[k] for k in ('archive_sha256','occurrence_id','blob_sha256','header_offset','byte_start','byte_end','value_index')},
      'projection_kind':r['projection_kind'],'structural_sha256':r['structural_sha256'],'structural_algorithm':'json-lexeme-tree-v1',
      'classification':json.loads(r['classification_json']),'native_version':{'state':'unknown','value':None,'source_pointer':None,'rule':'absent-v1'},
      'fields':{name:{'state':'unknown','value':None,'source_pointer':None,'rule':'absent-v1'} for name in COMMON_FIELDS},'times':{},
      'sensitivity':r['sensitivity'],'export_eligibility':r['export_eligibility'],'semantic_validation':'not_performed'}
    out['provenance']['json_pointer']=r['pointer']
    for f in db.execute('SELECT * FROM fields WHERE record_id=?',(record_id,)):
        ob={'state':f['state'],'value':json.loads(f['value_json']) if f['value_json'] is not None else None,'source_pointer':f['source_pointer'],'rule':f['rule']}
        if f['field_group']=='time':ob.update(role=f['role'],format_status=f['format_status']);out['times'][f['name']]=ob
        elif f['field_group']=='field':out['fields'][f['name']]=ob
        elif f['field_group']=='schema':out['native_version']=ob
    return out

def contract_rows(db):
    for table,name,tag in [('blobs','receipt-blob-v1.schema.json','ReceiptObservationBlobV1'),('occurrences','receipt-occurrence-v1.schema.json','ReceiptObservationOccurrenceV1'),
                           ('edges','receipt-edge-v2.schema.json','ReceiptObservationEdgeV2'),('anomalies','quarantine-record-v1.schema.json','ReceiptObservationAnomalyV1')]:
        for r in db.execute(f'SELECT * FROM {table} ORDER BY 1'):
            obj={'schema':tag,**dict(r)}
            if table=='blobs':obj['digest_algorithm']='sha256'
            yield name,obj
    for (rid,) in db.execute('SELECT record_id FROM records ORDER BY record_id'):yield 'receipt-envelope-v2.schema.json',envelope(db,rid)

def validate_projection(db)->dict:
    try:import jsonschema
    except ImportError as e:raise FoundationError('SCHEMA_VALIDATOR_UNAVAILABLE') from e
    ss=schemas();vs={name:jsonschema.Draft7Validator(s) for name,s in ss.items()}
    for s in ss.values():jsonschema.Draft7Validator.check_schema(s)
    counts={};errors=[]
    for name,obj in contract_rows(db):
        counts[name]=counts.get(name,0)+1
        for e in vs[name].iter_errors(obj):
            # No instance values or native payload in validation errors.
            errors.append({'schema_file':name,'row_ordinal':counts[name]-1,'validator':e.validator,'path_depth':len(e.path)})
            if len(errors)>1000:raise FoundationError('SCHEMA_ERROR_LIMIT')
    return {'schema':'ReceiptProjectionContractValidationV1','status':'FAIL' if errors else 'PASS','validated_rows':counts,
            'errors':errors,'native_receipt_validation':'not_performed','generated_schemas_are_source':'contracts.py'}

def load_request(path:pathlib.Path)->dict:
    with open_source(path) as f:raw=f.read(32769)
    if len(raw)>32768:raise FoundationError('INGESTION_REQUEST_SIZE_LIMIT')
    obj=plain(strict_loads(raw))
    expected=schemas()['ingestion-request-v1.schema.json']['properties']
    if not isinstance(obj,dict) or set(obj)!=set(expected):raise FoundationError('INGESTION_REQUEST_FIELDS_INVALID')
    import re
    for key,definition in expected.items():
        value=obj[key]
        if 'const' in definition and (type(value) is not type(definition['const']) or value!=definition['const']):raise FoundationError('INGESTION_REQUEST_VALUE_INVALID')
        if 'enum' in definition and value not in definition['enum']:raise FoundationError('INGESTION_REQUEST_VALUE_INVALID')
        if 'pattern' in definition and (not isinstance(value,str) or not re.fullmatch(definition['pattern'],value)):raise FoundationError('INGESTION_REQUEST_VALUE_INVALID')
    return {'expected_archive_sha256':obj['archive_sha256'],'compression':obj['compression'],
            'recover_streams':obj['parser_mode']=='explicit-json-streams-v1','require_collection_manifest':obj['collection_contract']=='AresReceiptCollectionManifestV1'}
