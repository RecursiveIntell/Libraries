"""Regression witnesses for boundaries missed by the first handoff."""
from __future__ import annotations
import os, pathlib, tempfile, unittest
from unittest.mock import patch
from receipt_foundation.common import FoundationError, open_source
from receipt_foundation.normalize import normalize
from receipt_foundation.serialization import strict_loads
from receipt_foundation.projection import build
from test_foundation import tar_bytes, regular

class SemanticHardeningTests(unittest.TestCase):
    def test_episode_does_not_become_session(self):
        e,_=normalize(strict_loads(b'{"episode_id":"ep-1"}'),'x.json')
        self.assertEqual(e['fields']['session_id']['state'],'unknown')
        self.assertEqual(e['fields']['episode_id']['value'],'ep-1')
    def test_goal_does_not_become_task(self):
        e,_=normalize(strict_loads(b'{"goal_id":"goal-1"}'),'x.json')
        self.assertEqual(e['fields']['task_id']['state'],'unknown')
        self.assertEqual(e['fields']['goal_id']['value'],'goal-1')
    def test_capability_does_not_become_tool(self):
        e,_=normalize(strict_loads(b'{"capability":"cap-1"}'),'x.json')
        self.assertEqual(e['fields']['tool']['state'],'unknown')
        self.assertEqual(e['fields']['capability']['value'],'cap-1')
    def test_agent_does_not_become_actor(self):
        e,_=normalize(strict_loads(b'{"agent_id":"agent-1"}'),'x.json')
        self.assertEqual(e['fields']['actor']['state'],'unknown')
        self.assertEqual(e['fields']['agent_id']['value'],'agent-1')
    def test_effect_identity_does_not_become_effect_kind(self):
        e,_=normalize(strict_loads(b'{"effect_id":"effect-1"}'),'x.json')
        self.assertEqual(e['fields']['effect']['state'],'unknown')
        self.assertEqual(e['fields']['effect_id']['value'],'effect-1')
    def test_schema_version_conflict_is_not_first_wins(self):
        e,issues=normalize(strict_loads(b'{"schema_version":"1","version":"2"}'),'x.json')
        self.assertEqual(e['native_version']['state'],'ambiguous')
        self.assertIn('NATIVE_VERSION_ALIAS_CONFLICT',{i['code'] for i in issues})
    def test_schema_tag_conflict_is_not_first_wins(self):
        e,issues=normalize(strict_loads(b'{"schema":"AresProfilePanelReceiptV2","schema_name":"OtherV1"}'),'x.json')
        self.assertIsNone(e['classification']['native_schema'])
        self.assertIn('NATIVE_SCHEMA_TAG_CONFLICT',{i['code'] for i in issues})
    def test_invalid_timezone_minutes_not_normalized(self):
        e,issues=normalize(strict_loads(b'{"timestamp":"2026-09-09T12:00:00+00:99"}'),'x.json')
        self.assertNotEqual(e['times']['timestamp']['format_status'],'rfc3339')
        self.assertTrue(issues)
    def test_valid_distinct_session_episode_both_preserved(self):
        e,issues=normalize(strict_loads(b'{"session_id":"s","episode_id":"e"}'),'x.json')
        self.assertEqual(e['fields']['session_id']['value'],'s')
        self.assertEqual(e['fields']['episode_id']['value'],'e')
        self.assertFalse(issues)

class SourceHardeningTests(unittest.TestCase):
    def test_open_nonregular_uses_nonblocking_before_fstat(self):
        real=os.open;flags=[]
        def capture(path,flag,*a,**kw):
            flags.append(flag);return real(path,flag,*a,**kw)
        with tempfile.TemporaryDirectory() as t:
            p=pathlib.Path(t)/'source';p.write_bytes(b'bytes')
            with patch('receipt_foundation.common.os.open',side_effect=capture):
                with open_source(p):pass
        self.assertTrue(flags[0]&os.O_NONBLOCK)
    def test_code_mutation_during_build_never_publishes(self):
        with tempfile.TemporaryDirectory() as t:
            p=pathlib.Path(t);a=p/'a.tar';o=p/'output.sqlite';a.write_bytes(tar_bytes([regular('x.json')]))
            with patch('receipt_foundation.projection.code_digest',side_effect=['a'*64,'b'*64]):
                with self.assertRaisesRegex(FoundationError,'IMPLEMENTATION_CHANGED_DURING_BUILD'):
                    build(a,o,compression='tar',require_collection_manifest=False)
            self.assertFalse(o.exists())

class PrivacyContractHardeningTests(unittest.TestCase):
    def test_credential_candidate_obeys_closed_sensitivity_schema(self):
        from receipt_foundation.projection import connect_readonly
        from receipt_foundation.contracts import validate_projection
        with tempfile.TemporaryDirectory() as t:
            p=pathlib.Path(t);a=p/'a.tar';o=p/'output.sqlite'
            raw=b'{"api_key":"'+b'ghp_'+b'A'*36+b'","schema":"SyntheticV1"}'
            a.write_bytes(tar_bytes([regular('x.json',raw)]))
            build(a,o,compression='tar',require_collection_manifest=False)
            with connect_readonly(o) as db:
                self.assertEqual(validate_projection(db)['status'],'PASS')

class DistinctNativeSemanticsTests(unittest.TestCase):
    def test_artifact_kind_is_not_a_schema_alias(self):
        e,issues=normalize(strict_loads(b'{"schema":"SyntheticReceiptV1","artifact_kind":"tool_effect"}'),'x.json')
        self.assertEqual(e['classification']['native_schema'],'SyntheticReceiptV1')
        self.assertEqual(e['fields']['artifact_kind']['value'],'tool_effect')
        self.assertFalse(issues)
    def test_unknown_local_offset_retains_known_utc_instant(self):
        e,issues=normalize(strict_loads(b'{"started_at":"2026-01-02T00:00:00-00:00","ended_at":"2026-01-01T00:00:00Z"}'),'x.json')
        self.assertEqual(e['times']['started_at']['format_status'],'rfc3339_unknown_local_offset')
        self.assertIn('TIME_ORDER_CONTRADICTION',{i['code'] for i in issues})
        self.assertEqual(e['times']['started_at']['value'],'2026-01-02T00:00:00-00:00')

class FifoReadBoundaryTests(unittest.TestCase):
    def test_fifo_source_rejected_without_blocking(self):
        import subprocess,sys
        with tempfile.TemporaryDirectory() as t:
            p=pathlib.Path(t)/'fifo';os.mkfifo(p)
            code='from pathlib import Path; from receipt_foundation.common import open_source,FoundationError; import sys\ntry:\n open_source(Path(sys.argv[1]))\nexcept FoundationError as e:\n print(e.code)\n'
            r=subprocess.run([sys.executable,'-c',code,str(p)],capture_output=True,text=True,timeout=5)
            self.assertEqual(r.returncode,0);self.assertEqual(r.stdout.strip(),'SOURCE_NOT_REGULAR')
