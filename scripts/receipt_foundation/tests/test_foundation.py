"""Synthetic-only regression fixtures. No production prompts or secrets committed."""
from __future__ import annotations
import dataclasses, io, json, os, pathlib, random, shutil, sqlite3, subprocess, tarfile, tempfile, unittest
from receipt_foundation.archive import Archive, TarReader, path_problem
from receipt_foundation.common import FoundationError, Limits, digest, identifier
from receipt_foundation.serialization import Number, parse, strict_loads, structural_digest
from receipt_foundation.normalize import normalize
from receipt_foundation.privacy import scan
from receipt_foundation.projection import build, connect_readonly, logical_manifest
from receipt_foundation.query import query, source_bytes

def tar_bytes(entries, fmt=tarfile.GNU_FORMAT):
    stream=io.BytesIO()
    with tarfile.open(fileobj=stream,mode='w',format=fmt) as archive:
        for name,raw,typ in entries:
            item=tarfile.TarInfo(name);item.type=typ;item.mode=0o600;item.mtime=0
            if typ==tarfile.REGTYPE:item.size=len(raw)
            if typ in (tarfile.SYMTYPE,tarfile.LNKTYPE):item.linkname='../../never-follow'
            archive.addfile(item,io.BytesIO(raw) if item.size else None)
    return stream.getvalue()

def regular(name,raw=b'{"schema":"ExampleReceiptV1","run_id":"run-1"}'):
    return (name,raw,tarfile.REGTYPE)

def problem_codes(result):return {i['code'] for i in result.issues}

class ArchiveTests(unittest.TestCase):
    def test_regular_hash_and_position(self):
        raw=b'{"a":1}';m=list(TarReader(io.BytesIO(tar_bytes([regular('x.json',raw)]))))[0]
        self.assertEqual(m.data,raw);self.assertEqual(m.blob_sha256,digest(raw));self.assertEqual(m.header_offset,0);self.assertEqual(m.data_offset,512)
    def test_longname_controls_accounted(self):
        reader=TarReader(io.BytesIO(tar_bytes([regular('deep/'+'a'*190+'.json')])));members=list(reader)
        self.assertEqual(len(members),1);self.assertEqual(len(reader.controls),1);self.assertEqual(reader.header_count,2)
    def test_ustar(self):self.assertEqual(len(list(TarReader(io.BytesIO(tar_bytes([regular('x')],tarfile.USTAR_FORMAT))))),1)
    def test_unsafe_paths(self):
        for value in ('../x','/x','C:/x','a/../x','a//x','././x','a\\x','a\x00x','e\u0301/x','a/CON','a/x.','a/x ','a/\u202ex'):
            with self.subTest(path_repr=repr(value)):self.assertTrue(path_problem(value)[1])
    def test_utf8_normal_path(self):self.assertEqual(path_problem('./caf\u00e9/x.json')[0],'caf\u00e9/x.json')
    def test_duplicate_member_names_not_deduplicated(self):
        ms=list(TarReader(io.BytesIO(tar_bytes([regular('x'),regular('./x')]))))
        self.assertEqual(len(ms),2);self.assertIn('DUPLICATE_MEMBER_PATH',ms[1].issues);self.assertNotEqual(ms[0].header_offset,ms[1].header_offset)
    def test_casefold_collision_is_observation(self):
        ms=list(TarReader(io.BytesIO(tar_bytes([regular('A'),regular('a')]))))
        self.assertIn('PATH_CASEFOLD_COLLISION',ms[1].issues);self.assertEqual(ms[1].path_key,'a')
    def test_link_and_special_types_rejected(self):
        for typ in (tarfile.SYMTYPE,tarfile.LNKTYPE,tarfile.CHRTYPE,tarfile.BLKTYPE,tarfile.FIFOTYPE):
            with self.subTest(type=typ):
                m=list(TarReader(io.BytesIO(tar_bytes([('x',b'',typ)]))))[0]
                self.assertIn('UNSAFE_MEMBER_TYPE',m.issues);self.assertIsNone(m.data)
    def test_checksum_corruption(self):
        b=bytearray(tar_bytes([regular('x')]));b[10]^=1
        with self.assertRaisesRegex(FoundationError,'TAR_CHECKSUM_MISMATCH'):list(TarReader(io.BytesIO(b)))
    def test_truncation(self):
        for cut in (0,100,511,514,1025):
            with self.subTest(cut=cut),self.assertRaises(FoundationError):list(TarReader(io.BytesIO(tar_bytes([regular('x')])[:cut])))
    def test_nonzero_tail(self):
        with self.assertRaisesRegex(FoundationError,'TAR_NONZERO_TRAILING_DATA'):list(TarReader(io.BytesIO(tar_bytes([regular('x')])+b'x')))
    def test_member_size_limit(self):
        with self.assertRaisesRegex(FoundationError,'MEMBER_SIZE_LIMIT'):list(TarReader(io.BytesIO(tar_bytes([regular('x',b'1234')])),dataclasses.replace(Limits(),member_bytes=3)))
    def test_decompressed_size_limit(self):
        with self.assertRaisesRegex(FoundationError,'DECOMPRESSED_SIZE_LIMIT'):list(TarReader(io.BytesIO(tar_bytes([regular('x')])),dataclasses.replace(Limits(),decompressed_bytes=513)))
    def test_member_count_limit(self):
        with self.assertRaisesRegex(FoundationError,'MEMBER_COUNT_LIMIT'):list(TarReader(io.BytesIO(tar_bytes([regular('x'),regular('y')])),dataclasses.replace(Limits(),members=1)))
    def test_pax_is_explicitly_unsupported(self):
        b=tar_bytes([regular('a'*190)],tarfile.PAX_FORMAT)
        with self.assertRaisesRegex(FoundationError,'TAR_EXTENSION_UNSUPPORTED'):list(TarReader(io.BytesIO(b)))
    def test_source_symlink_denied(self):
        with tempfile.TemporaryDirectory() as t:
            p=pathlib.Path(t);(p/'a').write_bytes(tar_bytes([regular('x')]));(p/'s').symlink_to(p/'a')
            with self.assertRaisesRegex(FoundationError,'SOURCE_OPEN_FAILURE'):list(Archive(p/'s',compression='tar').members())
    def test_zstd_roundtrip_and_truncation(self):
        self.assertIsNotNone(shutil.which('zstd'),'zstd must exist in validation environment')
        with tempfile.TemporaryDirectory() as t:
            p=pathlib.Path(t)/'source.zst';raw=tar_bytes([regular('x')]);data=subprocess.run(['zstd','-q','-c'],input=raw,capture_output=True,check=True).stdout
            p.write_bytes(data);self.assertEqual(len(list(Archive(p).members())),1)
            p.write_bytes(data[:-4])
            with self.assertRaises(FoundationError):list(Archive(p).members())
    def test_seeded_path_properties(self):
        rng=random.Random(8431)
        for _ in range(1000):
            parts=[''.join(rng.choice('abcXYZ012') for _ in range(rng.randint(1,12))) for _ in range(rng.randint(1,8))]
            good='/'.join(parts);self.assertEqual(path_problem(good)[0],good)
            parts.insert(rng.randrange(len(parts)+1),'..');self.assertIsNone(path_problem('/'.join(parts))[0])

class SerializationTests(unittest.TestCase):
    def test_exact_numbers(self):
        o=strict_loads(b'{"large":9007199254740993,"negative_zero":-0,"exp":1e400}')
        self.assertEqual(o['large'],Number('9007199254740993'));self.assertEqual(o['exp'],Number('1e400'))
        self.assertNotEqual(structural_digest(strict_loads(b'1.0')),structural_digest(strict_loads(b'1')))
    def test_structural_key_order(self):self.assertEqual(structural_digest(strict_loads(b'{"b":2,"a":1}')),structural_digest(strict_loads(b'{"a":1,"b":2}')))
    def test_duplicate_keys(self):self.assertIn('JSON_DUPLICATE_KEY',problem_codes(parse(b'{"a":1,"a":2}','x.json')))
    def test_nonfinite(self):
        for b in (b'NaN',b'Infinity',b'-Infinity'):
            with self.subTest(raw=b):self.assertIn('JSON_NONFINITE_NUMBER',problem_codes(parse(b,'x.json')))
    def test_invalid_utf8(self):self.assertEqual(parse(b'\xff','x.json').status,'unsupported')
    def test_bom(self):self.assertEqual(parse(b'\xef\xbb\xbf{}','x.json').status,'quarantined')
    def test_unpaired_surrogate(self):self.assertIn('JSON_UNPAIRED_SURROGATE',problem_codes(parse(b'"\\ud800"','x.json')))
    def test_valid_surrogate_pair(self):self.assertEqual(strict_loads(b'"\\ud83d\\ude00"'),'\U0001f600')
    def test_unicode_line_separator_not_jsonl_newline(self):
        raw='{"s":"a\u2028b\u2029c"}\n{"a":2}\n'.encode();r=parse(raw,'x.jsonl')
        self.assertEqual(r.observed,'strict-jsonl');self.assertEqual(len(r.values),2)
    def test_offsets_are_utf8_bytes(self):
        raw=' {"s":"\u00e9"}\n{"x":"\U0001f600"}\n'.encode();r=parse(raw,'x.jsonl')
        for v in r.values:self.assertEqual(structural_digest(strict_loads(raw[v.start:v.end])),v.structural_sha256)
    def test_multiline_stream_requires_opt_in(self):
        raw=b'{\n"a":1\n}\n{\n"b":2\n}\n'
        self.assertEqual(parse(raw,'x.jsonl').status,'quarantined')
        r=parse(raw,'x.jsonl',recover_streams=True);self.assertEqual(r.observed,'pretty-json-stream');self.assertEqual(len(r.values),2)
        self.assertIn('DECLARED_LINE_FORMAT_NONCONFORMANT',problem_codes(r))
    def test_concatenation_explicit(self):
        r=parse(b'{}[]{}','x.json',recover_streams=True);self.assertEqual(r.observed,'concatenated-json-values');self.assertEqual(len(r.values),3)
    def test_partial_recovery_does_not_skip_garbage(self):
        r=parse(b'{} BAD {}','x.json',recover_streams=True);self.assertEqual(r.status,'partial_quarantined');self.assertEqual(len(r.values),1)
    def test_blank_jsonl_line(self):self.assertEqual(parse(b'{}\n\n{}\n','x.jsonl').status,'quarantined')
    def test_ndjson_missing_lf(self):self.assertIn('NDJSON_MISSING_TERMINAL_LF',problem_codes(parse(b'{}','x.ndjson')))
    def test_empty_file(self):self.assertIn('EMPTY_DOCUMENT',problem_codes(parse(b'','x.ndjson')))
    def test_depth_guard(self):self.assertIn('JSON_DEPTH_LIMIT',problem_codes(parse(b'[[[0]]]','x.json',limits=dataclasses.replace(Limits(),json_depth=2))))
    def test_flat_token_guard(self):self.assertIn('JSON_TOKEN_COUNT_LIMIT',problem_codes(parse(b'[0,0,0,0]','x.json',limits=dataclasses.replace(Limits(),json_tokens=2))))
    def test_string_guard(self):self.assertIn('JSON_STRING_LENGTH_LIMIT',problem_codes(parse(b'"abcdef"','x.json',limits=dataclasses.replace(Limits(),json_string_chars=3))))
    def test_number_guard(self):self.assertIn('JSON_NUMBER_LENGTH_LIMIT',problem_codes(parse(b'1234','x.json',limits=dataclasses.replace(Limits(),json_number_chars=3))))
    def test_count_guard(self):self.assertIn('JSON_VALUE_COUNT_LIMIT',problem_codes(parse(b'{}{}{}','x.json',recover_streams=True,limits=dataclasses.replace(Limits(),json_values=2))))
    def test_markdown_fence_offsets(self):
        raw='\u00e9 prose\n```json\n{"a":1}\n```\n'.encode();r=parse(raw,'doc.md')
        self.assertEqual(len(r.values),1);v=r.values[0];self.assertEqual(raw[v.start:v.end],b'{"a":1}')
    def test_unknown_text_and_binary(self):
        self.assertEqual(parse(b'hello','x.txt').status,'classified_non_data');self.assertEqual(parse(b'\x00\x01','x.bin').status,'unsupported')
    def test_seeded_json_boundary_properties(self):
        rng=random.Random(2049)
        for _ in range(250):
            objs=[{'n':rng.randrange(-100000,100000),'s':rng.choice(['\u00e9','\U0001f600','x\u2028y','\\"'])} for i in range(rng.randrange(1,8))]
            raw=''.join(json.dumps(x,ensure_ascii=False) for x in objs).encode();r=parse(raw,'x.json',recover_streams=True)
            self.assertEqual(len(r.values),len(objs))
            for v in r.values:self.assertEqual(structural_digest(strict_loads(raw[v.start:v.end])),v.structural_sha256)
    def test_seeded_malformed_bytes_no_crash(self):
        rng=random.Random(99)
        for _ in range(1000):
            raw=bytes(rng.randrange(256) for i in range(rng.randrange(128)))
            r=parse(raw,'x.json',recover_streams=True);self.assertIn(r.status,{'parsed','parsed_with_issues','partial_quarantined','quarantined','unsupported'})

class ObservationTests(unittest.TestCase):
    def test_unknown_not_fabricated(self):
        n,e=normalize({},'x.json');self.assertEqual(n['fields']['actor']['state'],'unknown');self.assertEqual(n['times'],{});self.assertEqual(n['export_eligibility'],'denied')
    def test_null_distinct(self):self.assertEqual(normalize({'actor':None},'x.json')[0]['fields']['actor']['state'],'source_null')
    def test_version_suffix_not_inferred(self):self.assertEqual(normalize({'schema':'AresProfilePanelReceiptV2'},'x.json')[0]['native_version']['state'],'unknown')
    def test_alias_conflict_not_selected(self):
        n,e=normalize({'status':'returned','outcome':'failed'},'x.json');self.assertEqual(n['fields']['outcome']['state'],'ambiguous');self.assertEqual(e[0]['code'],'FIELD_ALIAS_CONFLICT')
    def test_time_roles_distinct(self):
        n,e=normalize({'timestamp':'2026-01-01T00:00:00Z','recorded_at':'2026-01-02T00:00:00Z','valid_at':None},'x.json')
        self.assertEqual(n['times']['timestamp']['role'],'source_timestamp');self.assertEqual(n['times']['valid_at']['state'],'source_null')
    def test_naive_time_unknown(self):self.assertEqual(normalize({'timestamp':'2026-01-01 00:00:00'},'x.json')[1][0]['code'],'TIME_FORMAT_OR_ZONE_UNKNOWN')
    def test_time_contradiction(self):
        n,e=normalize({'started_at':'2026-01-02T00:00:00Z','ended_at':'2026-01-01T00:00:00Z'},'x.json');self.assertIn('TIME_ORDER_CONTRADICTION',{x['code'] for x in e})
    def test_submicrosecond_not_truncated_for_order(self):
        n,e=normalize({'started_at':'2026-01-01T00:00:00.1234569Z','ended_at':'2026-01-01T00:00:00.1234568Z'},'x.json')
        self.assertNotIn('TIME_ORDER_PRECISION_UNSUPPORTED',{x['code'] for x in e});self.assertIn('TIME_ORDER_CONTRADICTION',{x['code'] for x in e})
    def test_examples_not_runtime(self):self.assertEqual(normalize({'schema':'AresProfilePanelReceiptV2'},'examples/x.json')[0]['classification']['category'],'example')
    def test_verification_assertion_not_certification(self):
        n,e=normalize({'schema':'AresProfilePanelVerificationV2','controller_verified':True},'x.json');self.assertEqual(n['fields']['controller_verified']['state'],'source_asserted');self.assertEqual(n['semantic_validation'],'not_performed')
    def test_privacy_canary_not_leaked(self):
        raw=b'{"api_key":"'+b'ghp_'+b'A'*36+b'"}'
        findings=scan(raw);self.assertTrue(findings);self.assertNotIn('ghp_',json.dumps(findings))
    def test_framed_identity(self):self.assertNotEqual(identifier('x','ab','c'),identifier('x','a','bc'))

class ProjectionTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.root=pathlib.Path(self.temp.name);self.archive=self.root/'source.tar';self.db=self.root/'projection.sqlite'
    def tearDown(self):self.temp.cleanup()
    def create(self,entries):
        self.archive.write_bytes(tar_bytes(entries));return build(self.archive,self.db,compression='tar',require_collection_manifest=False,recover_streams=True)
    def test_dedup_preserves_occurrences_and_rebuild(self):
        source=b'{"schema":"ReceiptV1","run_id":"r"}'
        a=self.create([regular('a.json',source),regular('b.json',source)])
        self.assertEqual(a['summary']['all_unique_blobs'],1);self.assertEqual(a['summary']['regular_files'],2);self.assertEqual(a['summary']['records'],2)
        self.db.unlink();b=build(self.archive,self.db,compression='tar',require_collection_manifest=False,recover_streams=True)
        self.assertEqual(a['logical_manifest'],b['logical_manifest'])
    def test_repeated_build_refuses_overwrite(self):
        self.create([regular('x.json')]);before=digest(self.db.read_bytes())
        with self.assertRaisesRegex(FoundationError,'OUTPUT_ALREADY_EXISTS'):build(self.archive,self.db,compression='tar',require_collection_manifest=False)
        self.assertEqual(before,digest(self.db.read_bytes()))
    def test_bad_archive_atomic_failure(self):
        self.archive.write_bytes(b'broken')
        with self.assertRaises(FoundationError) as caught:build(self.archive,self.db,compression='tar',require_collection_manifest=False)
        self.assertFalse(self.db.exists());self.assertEqual(list(self.root.glob('.receipt-staging-*')),[]);self.assertIn('archive_safety_partial',caught.exception.context)
    def test_expected_digest_failure(self):
        self.archive.write_bytes(tar_bytes([regular('x.json')]))
        with self.assertRaisesRegex(FoundationError,'ARCHIVE_DIGEST_EXPECTATION_MISMATCH'):build(self.archive,self.db,compression='tar',require_collection_manifest=False,expected_archive_sha256='0'*64)
        self.assertFalse(self.db.exists())
    def test_unsafe_accounting(self):
        r=self.create([regular('../escape.json'),('link',b'',tarfile.SYMTYPE),regular('x.json',b'bad'),regular('b.bin',b'\xff')])
        self.assertEqual(r['summary']['total_members'],4);self.assertEqual(r['summary']['unaccounted_members'],0);self.assertFalse((self.root.parent/'escape.json').exists())
    def test_query_and_source_roundtrip(self):
        raw=b'{"schema":"ReceiptV1","run_id":"r","trace_id":"t","repo_revision":"abc"}'
        self.create([regular('x.json',raw)])
        with connect_readonly(self.db) as db:
            record=query(db,'records')['rows'][0]['record_id']
            for kind,val in [('digest',digest(raw)),('occurrences',digest(raw)),('family','ReceiptV1'),('run','r'),('trace','t'),('revision','abc'),('provenance',record)]:
                with self.subTest(kind=kind):self.assertTrue(query(db,kind,val)['rows'])
            self.assertFalse(query(db,'family',"' OR 1=1 --")['rows'])
            self.assertNotIn('archive_member',query(db,'records')['rows'][0])
        output=self.root/'exact.json'
        with self.assertRaisesRegex(FoundationError,'PRIVATE_CONTENT_ACK_REQUIRED'):source_bytes(self.db,self.archive,record,output,compression='tar')
        source_bytes(self.db,self.archive,record,output,compression='tar',acknowledge_private=True);self.assertEqual(output.read_bytes(),raw)
    def test_readonly_context_closes_connection(self):
        self.create([regular('x.json')])
        with connect_readonly(self.db) as db:
            connection=db
            self.assertEqual(connection.execute('SELECT 1').fetchone()[0],1)
        with self.assertRaises(sqlite3.ProgrammingError):
            connection.execute('SELECT 1')
    def test_private_body_not_copied(self):
        canary='unique synthetic prompt body not searchable in projection '+('Z'*60)
        key='ghp_'+'A'*36
        self.create([regular('x.json',json.dumps({'schema':'ReceiptV1','prompt':canary,'api_key':key}).encode())])
        self.assertNotIn(canary.encode(),self.db.read_bytes());self.assertNotIn(key.encode(),self.db.read_bytes())
    def test_quarantine_is_queryable(self):
        self.create([regular('x.json',b'')])
        with connect_readonly(self.db) as db:self.assertEqual(query(db,'quarantine','EMPTY_DOCUMENT')['rows'][0]['code'],'EMPTY_DOCUMENT')
    def test_unknown_state_not_current_time(self):
        self.create([regular('x.json',b'{}')])
        with connect_readonly(self.db) as db:self.assertEqual(db.execute("SELECT count(*) FROM fields WHERE field_group='time'").fetchone()[0],0)
    def test_schema_and_time_queries(self):
        self.create([regular('x.json',b'{"schema":"ReceiptV1","version":1,"timestamp":"2026-01-01T00:00:00Z"}')])
        with connect_readonly(self.db) as db:
            self.assertTrue(query(db,'schemas')['rows']);self.assertTrue(query(db,'times')['rows'])
    def test_wrong_projection_version(self):
        self.db.write_bytes(b'not sqlite')
        with self.assertRaises(FoundationError):connect_readonly(self.db)
    def test_database_permissions_private(self):
        self.create([regular('x.json')]);self.assertEqual(self.db.stat().st_mode & 0o777,0o600)


class ContractTests(unittest.TestCase):
    def test_generated_schemas_match_code(self):
        from receipt_foundation.contracts import schemas
        root=pathlib.Path(__file__).resolve().parents[1]/'schemas'
        for name,value in schemas().items():self.assertEqual(json.loads((root/name).read_text()),value)
    def test_all_observation_rows_validate(self):
        from receipt_foundation.contracts import validate_projection,envelope
        with tempfile.TemporaryDirectory() as t:
            root=pathlib.Path(t);source=root/'in.tar';output=root/'out.sqlite'
            source.write_bytes(tar_bytes([regular('x.json',b'{"schema":"ReceiptV1","version":1,"status":"ok","actor":null}'),regular('bad.json',b'')]))
            build(source,output,compression='tar',require_collection_manifest=False)
            with connect_readonly(output) as db:
                self.assertEqual(validate_projection(db)['status'],'PASS')
                rid=db.execute('SELECT record_id FROM records').fetchone()[0]
                e=envelope(db,rid);self.assertEqual(e['fields']['actor']['state'],'source_null');self.assertEqual(e['fields']['owner']['state'],'unknown')
    def test_unknown_fields_rejected(self):
        import jsonschema
        from receipt_foundation.contracts import schemas
        schema=schemas()['receipt-blob-v1.schema.json']
        obj={'schema':'ReceiptObservationBlobV1','digest_algorithm':'sha256','sha256':'a'*64,'byte_length':0,'verified':True}
        self.assertFalse(jsonschema.Draft7Validator(schema).is_valid(obj))
    def test_request_refuses_authority_widening(self):
        from receipt_foundation.contracts import load_request
        request={'schema':'ReceiptObservationRequestV1','archive_sha256':'a'*64,'compression':'tar','parser_mode':'strict-v1',
                 'collection_contract':'generic-archive-v1','source_handle':'collection-1','authority_effect':'none','network_fetch':False}
        with tempfile.TemporaryDirectory() as t:
            p=pathlib.Path(t)/'request.json';p.write_text(json.dumps(request));opts=load_request(p);self.assertFalse(opts['recover_streams'])
            request['network_fetch']=True;p.write_text(json.dumps(request))
            with self.assertRaisesRegex(FoundationError,'INGESTION_REQUEST_VALUE_INVALID'):load_request(p)
    def test_request_no_extra_fields_or_version_coercion(self):
        from receipt_foundation.contracts import load_request
        with tempfile.TemporaryDirectory() as t:
            p=pathlib.Path(t)/'r.json';p.write_text('{"schema":"ReceiptObservationRequestV2","fallback":true}')
            with self.assertRaisesRegex(FoundationError,'INGESTION_REQUEST_FIELDS_INVALID'):load_request(p)
    def test_verified_run_pack_not_manufactured(self):
        n,errors=normalize({'schema':'RunPackEvidenceProjectionV1','verification':{'outcome':'Verified'}},'x.json')
        self.assertEqual(n['semantic_validation'],'not_performed');self.assertEqual(n['fields']['verification_state']['state'],'unknown')

class ResourceTests(unittest.TestCase):
    def test_projection_record_global_limit(self):
        with tempfile.TemporaryDirectory() as t:
            p=pathlib.Path(t);(p/'a.tar').write_bytes(tar_bytes([regular('x.jsonl',b'{}\n{}\n{}\n')]))
            with self.assertRaisesRegex(FoundationError,'PROJECTION_RECORD_COUNT_LIMIT'):
                build(p/'a.tar',p/'p.sqlite',compression='tar',require_collection_manifest=False,limits=dataclasses.replace(Limits(),projection_records=2))
            self.assertFalse((p/'p.sqlite').exists())
    def test_projection_disk_limit(self):
        with tempfile.TemporaryDirectory() as t:
            p=pathlib.Path(t);(p/'a.tar').write_bytes(tar_bytes([regular('x.json')]))
            with self.assertRaisesRegex(FoundationError,'PROJECTION_SIZE_LIMIT'):
                build(p/'a.tar',p/'p.sqlite',compression='tar',require_collection_manifest=False,limits=dataclasses.replace(Limits(),projection_bytes=4096))
            self.assertFalse((p/'p.sqlite').exists())

class FinalBoundaryTests(unittest.TestCase):
    def test_negative_zero_observation_preserves_lexeme(self):
        from receipt_foundation.serialization import plain
        self.assertEqual(plain(Number('-0')),{'numeric_lexeme':'-0'})
        self.assertEqual(plain(Number('0')),0)
    def test_identity_collision_not_ignored(self):
        import sqlite3
        from receipt_foundation.projection import insert_exact
        db=sqlite3.connect(':memory:')
        db.execute('CREATE TABLE blobs(sha256 TEXT PRIMARY KEY,byte_length INTEGER)')
        insert_exact(db,'blobs',('a'*64,10));insert_exact(db,'blobs',('a'*64,10))
        self.assertEqual(db.execute('SELECT count(*) FROM blobs').fetchone()[0],1)
        with self.assertRaisesRegex(FoundationError,'PROJECTION_IDENTITY_COLLISION'):insert_exact(db,'blobs',('a'*64,11))
        db.close()
    def test_occurrence_lookup_and_panel_verification_for_child(self):
        with tempfile.TemporaryDirectory() as t:
            root=pathlib.Path(t);source=root/'a.tar';out=root/'p.sqlite'
            source.write_bytes(tar_bytes([regular('p.json',b'{"schema":"AresProfilePanelReceiptV2","run_id":"r","results":[{"status":"returned"}]}'),regular('v.json',b'{"schema":"AresProfilePanelVerificationV2","receipt":"p.json"}')]))
            build(source,out,compression='tar',require_collection_manifest=False)
            with connect_readonly(out) as db:
                row=db.execute("SELECT record_id,occurrence_id FROM records WHERE projection_kind='profile_result_projection'").fetchone()
                self.assertEqual(query(db,'occurrence',row[1])['rows'][0]['occurrence_id'],row[1])
                # Synthetic generic archive has no collector source_path. Join explicitly
                # to exercise query traversal without pretending a native producer wrote it.
            import sqlite3
            db=sqlite3.connect(out)
            parent=db.execute("SELECT record_id FROM records WHERE category='profile_panel_receipt'").fetchone()[0]
            db.execute("UPDATE edges SET target_record_id=?,resolution='test_fixture_only' WHERE kind='VERIFIES'",(parent,));db.commit();db.close()
            with connect_readonly(out) as db:
                results=query(db,'verification',row[0])['rows']
                self.assertEqual(len(results),1);self.assertEqual(results[0]['target_record_id'],parent)



class GoldenTests(unittest.TestCase):
    def test_known_hash_and_synthetic_fixture(self):
        self.assertEqual(digest(b'abc'),'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad')
        folder=pathlib.Path(__file__).parent/'fixtures'
        raw=(folder/'synthetic-receipt.json').read_bytes();gold=json.loads((folder/'golden.json').read_text())
        self.assertEqual(digest(raw),gold['sha256']);self.assertEqual(len(raw),gold['bytes'])
        observation,_=normalize(strict_loads(raw),'synthetic.json')
        self.assertEqual(observation['classification']['category'],gold['expected_category'])
        self.assertEqual(observation['classification']['native_schema'],gold['expected_native_schema'])

if __name__=='__main__':unittest.main(verbosity=2)
