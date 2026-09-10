"""Versioned observation rules; no authority, outcome or time promotion."""
from __future__ import annotations
import datetime as dt, re
from fractions import Fraction
from typing import Any
from .common import canonical_json, identifier, RULES_VERSION
from .privacy import scalar_safe
from .serialization import Number, plain

# Tags are preserved verbatim. Their categories are metadata projections, not
# native family validation. Unknown tags never acquire a guessed schema version.
EXACT_TAGS = {
    'AresProfilePanelReceiptV1':'profile_panel_receipt',
    'AresProfilePanelReceiptV2':'profile_panel_receipt',
    'AresProfilePanelVerificationV1':'verification_receipt',
    'AresProfilePanelVerificationV2':'verification_receipt',
    'AresReceiptCollectionManifestV1':'collection_manifest',
    'AresReceiptLocationIndexV1':'collection_index',
    'AresReceiptCollectionBuildSummaryV1':'collection_receipt',
    'AresReceiptCollectionManualVerificationV1':'collection_receipt',
    'ArtifactValidationReceiptV1':'verification_receipt',
    'ChangeReceiptV1':'change_receipt',
    'agent-graph-mcp-receipt-v2':'runtime_execution_receipt',
    'libraries.phase-receipt.v1':'build_test_receipt',
}
COMMON_FIELDS = {
    'native_id':('receipt_id','id'), 'artifact_kind':('artifact_kind',),
    'owner':('owner','source_owner'),
    'repository':('repository','repo','repository_url'),
    'revision':('runtime_revision','producer_revision','repo_revision','commit_sha','git_commit'),
    'workspace':('workspace','worktree'),
    'run_id':('run_id',), 'trace_id':('trace_id',), 'session_id':('session_id',), 'episode_id':('episode_id',),
    'task_id':('task_id',), 'goal_id':('goal_id',), 'execution_id':('execution_id',),
    'actor':('actor','actor_id'), 'agent_id':('agent_id',), 'profile':('profile',),
    'policy_ref':('policy_ref','policy_id'), 'permit_ref':('permit_ref','permit_id'),
    'authority_scope':('authority_scope',), 'tool':('tool','tool_name'), 'capability':('capability',),
    'effect':('effect',), 'effect_id':('effect_id',), 'outcome':('status','result_state','outcome'),
    'verification_state':('evidence_state','verification_state'),
    'exit_code':('exit_code','returncode','return_code'),
    'controller_verified':('controller_verified',), 'execution_complete':('execution_complete',),
    'dry_run':('dry_run',), 'runtime':('runtime',),
    'sensitivity_assertion':('sensitivity','privacy_class'),
}
TIME_KEYS = ('timestamp','source_timestamp','created_at','created_utc','started_at','ended_at','finished_at',
             'verified_at','recorded_at','valid_at','valid_from','valid_to','observed_at','collected_at')
RFC3339 = re.compile(r'^\d{4}-\d\d-\d\d[Tt]\d\d:\d\d:\d\d(?:\.\d+)?(?:[Zz]|[+-](?:[01]\d|2[0-3]):[0-5]\d)$')


def pointer_escape(s: str) -> str:
    return s.replace('~','~0').replace('/','~1')


def state(value: Any, pointer: str | None, rule: str, *, present: bool=True) -> dict:
    if not present:return {'state':'unknown','value':None,'source_pointer':None,'rule':'absent-v1'}
    if value is None:return {'state':'source_null','value':None,'source_pointer':pointer,'rule':rule}
    if isinstance(value,str) and not scalar_safe(value):
        return {'state':'redacted','value':None,'source_pointer':pointer,'rule':'metadata-sensitive-exclusion-v1'}
    if isinstance(value,(str,bool,Number)):
        return {'state':'source_asserted','value':plain(value),'source_pointer':pointer,'rule':rule}
    return {'state':'unsupported_type','value':None,'source_pointer':pointer,'rule':rule}


def classify(o: Any, path: str) -> dict:
    segments=set(path.lower().split('/'))
    context='template' if 'templates' in segments or 'template' in segments else 'example' if 'examples' in segments or 'fixtures' in segments else 'archived_snapshot' if any(x in segments for x in ('archive','archives','snapshots','worktrees','.worktrees')) else 'unspecified'
    tag=None; tag_pointer=None
    tag_conflict=False
    if isinstance(o,dict):
        tags=[(k,o[k]) for k in ('schema','schema_name','report_schema') if isinstance(o.get(k),str) and scalar_safe(o[k])]
        tag_conflict=len({value for _,value in tags})>1
        if tags and not tag_conflict:tag_pointer='/'+tags[0][0];tag=tags[0][1]
        if '$schema' in o or '$id' in o and 'properties' in o:
            return {'category':'schema','context':context,'method':'exact-key','confidence':'high','rule':'json-schema-key-v1','evidence':'/$schema' if '$schema' in o else '/$id','native_schema':tag,'native_schema_pointer':tag_pointer}
    if tag_conflict:
        return {'category':'unknown','context':context,'method':'ambiguous','confidence':'unknown','rule':'conflicting-native-tags-v2','evidence':None,'native_schema':None,'native_schema_pointer':None}
    if context in {'template','example'}:
        return {'category':context,'context':context,'method':'path-heuristic','confidence':'medium','rule':'path-context-v1','evidence':'archive_member','native_schema':tag,'native_schema_pointer':tag_pointer}
    if tag in EXACT_TAGS:
        return {'category':EXACT_TAGS[tag],'context':context,'method':'exact-tag','confidence':'high','rule':'observed-tags-v1','evidence':tag_pointer,'native_schema':tag,'native_schema_pointer':tag_pointer}
    # These are explicitly heuristic name classifications, never schema validation.
    if tag:
        candidates=(('verification','verification_receipt'),('attestation','provenance_bundle'),('manifest','artifact_manifest'),('benchmark','benchmark_receipt'),('release','release_receipt'),('publication','publication_receipt'),('publish','publication_receipt'),('claim','claim_evidence_receipt'),('permit','policy_permit_receipt'),('policy','policy_permit_receipt'),('effect','tool_effect_receipt'),('tool','tool_effect_receipt'),('build','build_test_receipt'),('test','build_test_receipt'),('execution','runtime_execution_receipt'),('receipt','other_receipt'))
        for fragment,cat in candidates:
            if fragment in tag.lower():return {'category':cat,'context':context,'method':'tag-heuristic','confidence':'medium','rule':'tag-keywords-v1','evidence':tag_pointer,'native_schema':tag,'native_schema_pointer':tag_pointer}
    return {'category':'unknown','context':context,'method':'unresolved','confidence':'unknown','rule':'unknown-v1','evidence':None,'native_schema':tag,'native_schema_pointer':tag_pointer}


def normalize(o: Any, path: str, pointer: str='') -> tuple[dict, list[dict]]:
    cl=classify(o,path);fields={};issues=[]
    native_version=state(None,None,'absent-v1',present=False)
    if not isinstance(o,dict):
        return {'schema':'ReceiptObservationEnvelopeV2','classification':cl,'native_version':native_version,
                'fields':{k:state(None,None,'absent-v1',present=False) for k in COMMON_FIELDS},'times':{},
                'semantic_validation':'not_performed','export_eligibility':'denied'},[]
    # Version conflicts are evidence, not first-wins parsing.
    if cl['rule']=='conflicting-native-tags-v2':issues.append({'code':'NATIVE_SCHEMA_TAG_CONFLICT','pointer':pointer})
    versions=[key for key in ('schema_version','version') if key in o]
    if len({canonical_json(plain(o[key])) for key in versions})>1:
        native_version={'state':'ambiguous','value':None,'source_pointer':None,'rule':'conflicting-native-versions-v2'}
        issues.append({'code':'NATIVE_VERSION_ALIAS_CONFLICT','pointer':pointer})
    elif versions:
        key=versions[0];native_version=state(o[key],pointer+'/'+key,'explicit-native-version-v1')
    for target,keys in COMMON_FIELDS.items():
        found=[k for k in keys if k in o]
        if len(found)>1:
            vals=[canonical_json(plain(o[k])) for k in found]
            if len(set(vals))>1:
                fields[target]={'state':'ambiguous','value':None,'source_pointer':None,'rule':'conflicting-aliases-v1'}
                issues.append({'code':'FIELD_ALIAS_CONFLICT','pointer':pointer,'field':target});continue
        fields[target]=state(o[found[0]],pointer+'/'+found[0],'direct-field-v1') if found else state(None,None,'absent-v1',present=False)
    # Two observed contracts expose unambiguous named fields, not invented aliases.
    if cl['native_schema']=='agent-graph-mcp-receipt-v2' and 'trace' in o and fields['trace_id']['state']=='unknown':
        fields['trace_id']=state(o['trace'],pointer+'/trace','agent-graph-trace-field-v1')
    if cl['native_schema']=='ChangeReceiptV1' and isinstance(o.get('source_identity'),dict):
        if 'head' in o['source_identity'] and fields['revision']['state']=='unknown':
            fields['revision']=state(o['source_identity']['head'],pointer+'/source_identity/head','change-receipt-source-head-v1')
    times={};parsed_times={}
    for key in TIME_KEYS:
        if key not in o:continue
        ob=state(o[key],pointer+'/'+key,'source-time-field-v1')
        ob['role']='valid_time_source_assertion' if key.startswith('valid_') else 'recorded_time_source_assertion' if key=='recorded_at' else 'observation_time_source_assertion' if key in {'observed_at','collected_at'} else 'source_timestamp'
        ob['format_status']='unknown'
        if isinstance(o[key],str) and ob['state']=='source_asserted':
            if not RFC3339.fullmatch(o[key]):
                ob['format_status']='ambiguous_or_invalid';issues.append({'code':'TIME_FORMAT_OR_ZONE_UNKNOWN','pointer':pointer+'/'+key})
            else:
                try:
                    # Parse whole seconds separately; datetime never receives or truncates the fraction.
                    fraction=re.search(r'\.(\d+)',o[key])
                    whole=re.sub(r'\.\d+','',o[key]).upper().replace('Z','+00:00')
                    t=dt.datetime.fromisoformat(whole)
                    delta=t-dt.datetime(1970,1,1,tzinfo=dt.timezone.utc)
                    subsecond=Fraction(int(fraction.group(1)),10**len(fraction.group(1))) if fraction else Fraction(0)
                    parsed_times[key]=Fraction(delta.days*86400+delta.seconds)+subsecond
                    ob['format_status']='rfc3339_unknown_local_offset' if o[key].endswith('-00:00') else 'rfc3339'
                except ValueError:
                    ob['format_status']='invalid';issues.append({'code':'TIME_INVALID','pointer':pointer+'/'+key})
        elif isinstance(o[key],Number):
            ob['format_status']='numeric_semantics_unknown';issues.append({'code':'TIME_NUMERIC_SEMANTICS_UNKNOWN','pointer':pointer+'/'+key})
        times[key]=ob
    for start,end in [('started_at','ended_at'),('started_at','finished_at'),('valid_from','valid_to')]:
        if start in parsed_times and end in parsed_times and parsed_times[end]<parsed_times[start]:
            issues.append({'code':'TIME_ORDER_CONTRADICTION','pointer':pointer,'start_field':start,'end_field':end})
    return {'schema':'ReceiptObservationEnvelopeV2','classification':cl,'native_version':native_version,
            'fields':fields,'times':times,'semantic_validation':'not_performed','export_eligibility':'denied'},issues
