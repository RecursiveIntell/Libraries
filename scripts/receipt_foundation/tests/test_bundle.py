"""Synthetic-only publication, corruption, failure and concurrency witnesses."""
from __future__ import annotations
import json, os, pathlib, sqlite3, subprocess, sys, tempfile, unittest
from unittest.mock import patch
from receipt_foundation.bundle import build_bundle, verify_bundle, bundle_query, verified_bundle, _publish_noreplace
from receipt_foundation.common import FoundationError, digest, write_json
from receipt_foundation.projection import code_digest
from receipt_foundation.doctor import doctor
from test_foundation import tar_bytes, regular


def collection_bytes(*, secret:bool=False) -> bytes:
    body={'schema':'SyntheticReceiptV1','run_id':'run-1','session_id':'s-1','episode_id':'e-1','task_id':'t-1','goal_id':'g-1'}
    if secret:body['api_key']='ghp_'+'A'*36
    raw=json.dumps(body,sort_keys=True).encode()
    index=json.dumps({'schema':'AresReceiptLocationIndexV1','counts':{'all_candidates':1},'artifacts':{'synthetic':[{'path':'/synthetic/source.json'}]}},sort_keys=True).encode()
    manifest=json.dumps({'schema':'AresReceiptCollectionManifestV1','entry_count':1,'total_source_bytes':len(raw),
        'selection_index':{'sha256':digest(index)},'created_utc':'2026-01-01T00:00:00Z','entries':[{'archive_member':'receipts/source.json','source_path':'/synthetic/source.json','bytes':len(raw),'sha256':digest(raw),'evidence_class':'synthetic'}]},sort_keys=True).encode()
    members={'receipts/source.json':raw,'metadata/receipt-location-index.json':index,'RECEIPT_COLLECTION_MANIFEST.json':manifest}
    checksums=''.join(f'{digest(value)}  {name}\n' for name,value in members.items()).encode()
    members['RECEIPT_COLLECTION_CHECKSUMS.sha256']=checksums
    return tar_bytes([regular(k,v) for k,v in members.items()])

class BundleTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.root=pathlib.Path(self.tmp.name)
        self.archive=self.root/'source.tar';self.archive.write_bytes(collection_bytes())
        self.sha=digest(self.archive.read_bytes());self.target=self.root/'result'
    def tearDown(self):self.tmp.cleanup()
    def build(self):return build_bundle(self.archive,self.target,expected_archive_sha256=self.sha,compression='tar')
    def verify(self):return verify_bundle(self.target,expected_archive_sha256=self.sha)
    def command(self,target=None):
        return [sys.executable,'-m','receipt_foundation','bundle-build',str(self.archive),'--destination',str(target or self.target),'--expect-sha256',self.sha,'--compression','tar']
    def test_build_verify_query_and_distinct_identities(self):
        self.assertEqual(self.build()['status'],'PASS');self.assertEqual(self.verify()['status'],'PASS')
        for kind,value in [('session','s-1'),('episode','e-1'),('task','t-1'),('goal','g-1')]:
            r=bundle_query(self.target,kind,value,expected_archive_sha256=self.sha)
            self.assertEqual(len(r['rows']),1);self.assertEqual(r['bundle_verification']['status'],'PASS')
        self.assertFalse(bundle_query(self.target,'session','e-1',expected_archive_sha256=self.sha)['rows'])
    def test_existing_directory_not_replaced(self):
        self.target.mkdir()
        with self.assertRaisesRegex(FoundationError,'BUNDLE_ALREADY_EXISTS'):self.build()
        self.assertEqual(list(self.target.iterdir()),[])
    def test_existing_file_not_replaced(self):
        self.target.write_bytes(b'keep')
        with self.assertRaisesRegex(FoundationError,'BUNDLE_ALREADY_EXISTS'):self.build()
        self.assertEqual(self.target.read_bytes(),b'keep')
    def test_existing_symlink_not_replaced(self):
        self.target.symlink_to(self.archive)
        with self.assertRaisesRegex(FoundationError,'BUNDLE_ALREADY_EXISTS'):self.build()
        self.assertTrue(self.target.is_symlink())
    def test_wrong_source_digest_no_final_name(self):
        with self.assertRaisesRegex(FoundationError,'ARCHIVE_DIGEST_EXPECTATION_MISMATCH'):
            build_bundle(self.archive,self.target,expected_archive_sha256='0'*64,compression='tar')
        self.assertFalse(self.target.exists())
    def test_missing_collection_contract_no_admission(self):
        self.archive.write_bytes(tar_bytes([regular('x.json')]))
        with self.assertRaisesRegex(FoundationError,'BUNDLE_COLLECTION_NOT_VERIFIED'):
            build_bundle(self.archive,self.target,expected_archive_sha256=digest(self.archive.read_bytes()),compression='tar')
        self.assertFalse(self.target.exists())
    def test_detached_receipt_write_failure_no_final_name(self):
        def fail(path,obj,**kw):
            if path.name=='build.json':raise FoundationError('SYNTHETIC_WRITE_FAILURE')
            return write_json(path,obj,**kw)
        with patch('receipt_foundation.bundle.write_json',side_effect=fail):
            with self.assertRaisesRegex(FoundationError,'SYNTHETIC_WRITE_FAILURE'):self.build()
        self.assertFalse(self.target.exists())
        stages=list(self.root.glob('.receipt-bundle-staging-*'))
        self.assertEqual(len(stages),1);self.assertTrue((stages[0]/'FAILURE.json').is_file())
    def test_private_parent_required(self):
        self.root.chmod(0o755)
        with self.assertRaisesRegex(FoundationError,'BUNDLE_DIRECTORY_NOT_PRIVATE'):self.build()
        self.root.chmod(0o700)
    def test_symlink_ancestor_rejected(self):
        link=self.root/'alias';link.symlink_to(self.root,target_is_directory=True)
        with self.assertRaises((FoundationError,OSError)):
            build_bundle(self.archive,link/'result',expected_archive_sha256=self.sha,compression='tar')
    def test_tampered_database_rejected(self):
        self.build()
        with (self.target/'projection.sqlite').open('ab') as f:f.write(b'changed')
        with self.assertRaisesRegex(FoundationError,'BUNDLE_FILE_DIGEST_MISMATCH'):self.verify()
    def test_tampered_receipt_rejected(self):
        self.build();(self.target/'build.json').write_text('{}')
        with self.assertRaisesRegex(FoundationError,'BUNDLE_FILE_DIGEST_MISMATCH'):self.verify()
    def test_extra_file_rejected(self):
        self.build();(self.target/'extra').write_text('no')
        with self.assertRaisesRegex(FoundationError,'BUNDLE_MEMBER_SET_MISMATCH'):self.verify()
    def test_manifest_missing_rejected(self):
        self.build();(self.target/'bundle.json').unlink()
        with self.assertRaisesRegex(FoundationError,'BUNDLE_MEMBER_SET_MISMATCH'):self.verify()
    def test_wrong_expected_archive_rejected(self):
        self.build()
        with self.assertRaisesRegex(FoundationError,'BUNDLE_ARCHIVE_BINDING_MISMATCH'):
            verify_bundle(self.target,expected_archive_sha256='0'*64)
    def test_stale_code_rejected(self):
        self.build()
        with patch('receipt_foundation.bundle.code_digest',return_value='0'*64):
            with self.assertRaisesRegex(FoundationError,'BUNDLE_IMPLEMENTATION_BINDING_MISMATCH'):self.verify()
    def test_wide_file_permissions_rejected(self):
        self.build();(self.target/'projection.sqlite').chmod(0o644)
        with self.assertRaisesRegex(FoundationError,'BUNDLE_FILE_NOT_PRIVATE'):self.verify()
    def test_hardlink_rejected(self):
        self.build();os.link(self.target/'build.json',self.root/'copy')
        with self.assertRaisesRegex(FoundationError,'BUNDLE_FILE_NOT_SINGLE_REGULAR'):self.verify()
    def test_bundle_file_symlink_rejected(self):
        self.build();p=self.target/'build.json';p.rename(self.root/'moved.json');p.symlink_to(self.root/'moved.json')
        with self.assertRaises((OSError,FoundationError)):self.verify()
    def test_secret_candidate_does_not_leak_or_break_schema(self):
        self.archive.write_bytes(collection_bytes(secret=True));self.sha=digest(self.archive.read_bytes());self.build()
        self.assertEqual(self.verify()['status'],'PASS')
        self.assertNotIn(('ghp_'+'A'*36).encode(),(self.target/'projection.sqlite').read_bytes())
    def test_same_descriptor_survives_bundle_rename(self):
        self.build()
        with verified_bundle(self.target,expected_archive_sha256=self.sha) as (db,_):
            self.target.rename(self.root/'renamed')
            self.assertEqual(db.execute('SELECT count(*) FROM records').fetchone()[0],3)
    def test_concurrent_publish_exactly_one_winner(self):
        procs=[subprocess.Popen(self.command(),stdout=subprocess.PIPE,stderr=subprocess.PIPE) for _ in range(2)]
        results=[p.communicate(timeout=30) for p in procs]
        self.assertEqual(sorted(p.returncode for p in procs),[0,2],results)
        self.assertEqual(self.verify()['status'],'PASS')
    def test_process_exit_before_publication_leaves_no_final(self):
        code='''import os,pathlib
import receipt_foundation.bundle as b
b._publish_noreplace=lambda *args: os._exit(73)
b.build_bundle(pathlib.Path(ARCHIVE),pathlib.Path(TARGET),expected_archive_sha256=SHA,compression='tar')'''
        code=code.replace('ARCHIVE',repr(str(self.archive))).replace('TARGET',repr(str(self.target))).replace('SHA',repr(self.sha))
        p=subprocess.run([sys.executable,'-c',code],capture_output=True,timeout=30)
        self.assertEqual(p.returncode,73,p.stderr);self.assertFalse(self.target.exists());self.assertTrue(list(self.root.glob('.receipt-bundle-staging-*')))
    def test_process_exit_after_rename_is_valid_but_no_success_ack(self):
        code='''import os,pathlib
import receipt_foundation.bundle as b
real=b._publish_noreplace
def stop(*args):
 real(*args)
 os._exit(74)
b._publish_noreplace=stop
b.build_bundle(pathlib.Path(ARCHIVE),pathlib.Path(TARGET),expected_archive_sha256=SHA,compression='tar')'''
        code=code.replace('ARCHIVE',repr(str(self.archive))).replace('TARGET',repr(str(self.target))).replace('SHA',repr(self.sha))
        p=subprocess.run([sys.executable,'-c',code],capture_output=True,timeout=30)
        self.assertEqual(p.returncode,74,p.stderr);self.assertEqual(p.stdout,b'');self.assertEqual(self.verify()['status'],'PASS')
    def test_raw_rename_noreplace_retains_source_and_destination(self):
        (self.root/'stage').mkdir();self.target.mkdir()
        fd=os.open(self.root,os.O_RDONLY|os.O_DIRECTORY)
        try:
            with self.assertRaisesRegex(FoundationError,'BUNDLE_ALREADY_EXISTS'):_publish_noreplace(fd,'stage','result')
            self.assertTrue((self.root/'stage').is_dir());self.assertTrue(self.target.is_dir())
        finally:os.close(fd)
    def test_doctor_is_read_only(self):
        before=set(self.root.iterdir());r=doctor(self.archive,self.root,expected_archive_sha256=self.sha)
        self.assertEqual(r['status'],'PASS');self.assertEqual(before,set(self.root.iterdir()));self.assertFalse(r['owner_approval_established'])
    def test_doctor_wrong_archive_blocks(self):
        r=doctor(self.archive,self.root,expected_archive_sha256='0'*64);self.assertEqual(r['status'],'BLOCKED')
