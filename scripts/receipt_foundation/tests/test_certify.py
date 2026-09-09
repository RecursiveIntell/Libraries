"""Certification orchestration fails closed and never overwrites an existing run."""
import json,pathlib,tempfile,unittest
from unittest.mock import patch
import certify_local
from receipt_foundation.common import FoundationError

class CertificationDriverTests(unittest.TestCase):
    def test_existing_work_directory_unchanged(self):
        with tempfile.TemporaryDirectory() as t:
            root=pathlib.Path(t);work=root/'existing';work.mkdir();(work/'keep').write_bytes(b'keep')
            with self.assertRaises(FileExistsError):certify_local.certify(root/'source',work,'0'*64)
            self.assertEqual(list(p.name for p in work.iterdir()),['keep'])
    def test_relative_paths_rejected_before_writes(self):
        with self.assertRaisesRegex(FoundationError,'ABSOLUTE_PATHS_REQUIRED'):
            certify_local.certify(pathlib.Path('source'),pathlib.Path('result'),'0'*64)
    def test_failed_first_command_receipted_and_no_build(self):
        with tempfile.TemporaryDirectory() as t:
            root=pathlib.Path(t);work=root/'new'
            with patch('certify_local.subprocess.run') as run:
                run.return_value.returncode=3
                r=certify_local.certify(root/'source',work,'0'*64)
                self.assertEqual(run.call_count,1)
            self.assertEqual(r['status'],'FAIL');self.assertFalse((work/'observation-bundle').exists())
            saved=json.loads((work/'LOCAL_CERTIFICATION.json').read_text())
            self.assertEqual(saved['commands'][0]['exit_code'],3)
    def test_unexpected_command_timeout_is_typed(self):
        import subprocess
        with tempfile.TemporaryDirectory() as t:
            root=pathlib.Path(t);work=root/'new'
            with patch('certify_local.subprocess.run',side_effect=subprocess.TimeoutExpired('synthetic',240)):
                r=certify_local.certify(root/'source',work,'0'*64)
            self.assertEqual(r['status'],'FAIL');self.assertEqual(r['error']['code'],'LOCAL_CERTIFICATION_FAILURE')
            self.assertIsNone(r['commands'][0]['exit_code'])
