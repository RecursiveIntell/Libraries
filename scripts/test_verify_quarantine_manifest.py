"""Python3.14+ archive/recovery integrity and precise source-pack exclusion."""
from pathlib import Path
import importlib.util
import copy
import hashlib
import io
import json
import subprocess
import sys
import tarfile
import tempfile
import unittest

assert sys.version_info >= (3, 14)
ROOT = Path(__file__).resolve().parents[1]
PREFIX = '_salvage_from_libraries2/Libraries2/target_schema_gen'


class QuarantineTests(unittest.TestCase):
    def setUp(self):
        # The existing zipper checks absolute ancestors and excludes /tmp.
        # Keep this fixture under the isolated source so its retained-file
        # positive controls test the intended subtree exclusion, not that quirk.
        self.tmp = tempfile.TemporaryDirectory(prefix='.quarantine-test-', dir=ROOT)
        self.addCleanup(self.tmp.cleanup)
        self.base = Path(self.tmp.name)
        self.repo = self.base / 'repo'
        self.repo.mkdir()
        subprocess.run(['git', 'init', '-q', str(self.repo)], check=True)
        folder = self.repo / PREFIX
        folder.mkdir(parents=True)
        (folder / 'binary').write_bytes(b'\x00\xfffixture-binary\x00')
        (folder / 'binary').chmod(0o755)
        (folder / 'metadata.json').write_text('{"fixture":true}\n')
        (folder / 'metadata.json').chmod(0o644)
        (self.repo / 'keep.rs').write_text('// retained source\n')
        subprocess.run(['git', 'add', '.'], cwd=self.repo, check=True)
        subprocess.run(['git', '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '-qm', 'fixture'], cwd=self.repo, check=True)
        self.sha = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=self.repo, text=True).strip()
        self.out = self.base / 'external'
        self.out.mkdir()
        self.manifest = self.out / 'manifest.json'

    def cli(self, *extra):
        return subprocess.run([sys.executable, str(ROOT / 'scripts/verify_quarantine_manifest.py'), '--repo', str(self.repo), '--manifest', str(self.manifest), *extra], text=True, capture_output=True, timeout=30)

    def snapshot(self):
        result = self.cli('--create', '--source', self.sha)
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        self.document = json.loads(self.manifest.read_text())
        self.archive = self.out / self.document['archive']['file']

    def save_manifest(self):
        self.manifest.write_text(json.dumps(self.document))

    def assert_denied(self, code, *extra):
        result = self.cli(*extra)
        self.assertEqual(result.returncode, 2, result.stderr + result.stdout)
        self.assertEqual(json.loads(result.stderr)['detail'], code)

    def rewrite_archive(self, change):
        with tarfile.open(self.archive, 'r:xz') as stream:
            members = []
            for member in stream:
                body = stream.extractfile(member)
                assert body is not None
                members.append((copy.copy(member), body.read()))
        members = change(members)
        with tarfile.open(self.archive, 'w:xz') as stream:
            for member, data in members:
                stream.addfile(member, io.BytesIO(data))
        self.document['archive']['sha256'] = hashlib.sha256(self.archive.read_bytes()).hexdigest()
        self.save_manifest()

    def test_roundtrip_recovers_exact_git_bytes_and_modes(self):
        created = self.cli('--create', '--source', self.sha)
        self.assertEqual(created.returncode, 0, created.stderr + created.stdout)
        verified = self.cli('--extract-to', str(self.base / 'restored'))
        self.assertEqual(verified.returncode, 0, verified.stderr + verified.stdout)
        for name in ('binary', 'metadata.json'):
            original = self.repo / PREFIX / name
            restored = self.base / 'restored' / PREFIX / name
            self.assertEqual(restored.read_bytes(), original.read_bytes())
            self.assertEqual(restored.stat().st_mode & 0o777, original.stat().st_mode & 0o777)

    def test_source_zip_excludes_only_the_quarantined_tree(self):
        spec = importlib.util.spec_from_file_location('libraries_source_zip', ROOT / 'zip.py')
        assert spec is not None and spec.loader is not None
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        other = self.repo / 'unrelated/target_schema_gen/keep.json'
        other.parent.mkdir(parents=True)
        other.write_text('{}')
        archive = self.out / 'source.zip'
        module.build_archive(self.repo, archive)
        import zipfile
        with zipfile.ZipFile(archive) as stream:
            files = set(stream.namelist())
        self.assertIn('unrelated/target_schema_gen/keep.json', files)
        self.assertIn('keep.rs', files)
        self.assertFalse(any(name.startswith(PREFIX + '/') for name in files))

    def test_archive_corruption_is_not_a_pass(self):
        self.snapshot()
        self.archive.write_bytes(self.archive.read_bytes() + b'corrupt')
        self.assert_denied('ARCHIVE_DIGEST_MISMATCH')

    def test_manifest_digest_tampering_is_rejected(self):
        self.snapshot()
        self.document['entries'][0]['sha256'] = '0' * 64
        self.save_manifest()
        self.assert_denied('CONTENT_SHA256_MISMATCH')

    def test_archive_and_manifest_cotampering_still_fails_git_identity(self):
        self.snapshot()
        def change(members):
            member, data = members[0]
            data = bytes([data[0] ^ 1]) + data[1:]
            self.document['entries'][0]['sha256'] = hashlib.sha256(data).hexdigest()
            return [(member, data), *members[1:]]
        self.rewrite_archive(change)
        self.assert_denied('CONTENT_GIT_OID_MISMATCH')

    def test_missing_member_is_rejected(self):
        self.snapshot()
        self.rewrite_archive(lambda members: members[:-1])
        self.assert_denied('MISSING_ARCHIVE_MEMBER')

    def test_duplicate_member_is_rejected(self):
        self.snapshot()
        self.rewrite_archive(lambda members: [*members, members[0]])
        self.assert_denied('EXTRA_OR_DUPLICATE_ARCHIVE_MEMBER')

    def test_extra_member_is_rejected(self):
        self.snapshot()
        def change(members):
            member = copy.copy(members[0][0])
            member.name = PREFIX + '/extra'
            return [*members, (member, members[0][1])]
        self.rewrite_archive(change)
        self.assert_denied('EXTRA_OR_DUPLICATE_ARCHIVE_MEMBER')

    def test_traversal_never_writes_outside_the_restore_root(self):
        self.snapshot()
        def change(members):
            members[0][0].name = PREFIX + '/../../escape'
            return members
        self.rewrite_archive(change)
        self.assert_denied('UNSAFE_PATH', '--extract-to', str(self.base / 'restore'))
        self.assertFalse((self.base / 'escape').exists())

    def test_symlinks_and_hardlinks_are_rejected(self):
        self.snapshot()
        original = self.archive.read_bytes()
        for kind in (tarfile.SYMTYPE, tarfile.LNKTYPE):
            with self.subTest(kind=kind):
                self.archive.write_bytes(original)
                def change(members):
                    member = members[0][0]
                    member.type = kind
                    member.linkname = '../../outside'
                    member.size = 0
                    return [(member, b''), *members[1:]]
                self.rewrite_archive(change)
                self.assert_denied('NONREGULAR_ARCHIVE_MEMBER')

    def test_archive_mode_is_part_of_the_contract(self):
        self.snapshot()
        def change(members):
            members[0][0].mode = 0o777
            return members
        self.rewrite_archive(change)
        self.assert_denied('ARCHIVE_MEMBER_METADATA')

    def test_existing_extraction_destination_is_not_modified(self):
        self.snapshot()
        destination = self.base / 'existing'
        destination.mkdir()
        sentinel = destination / 'sentinel'
        sentinel.write_bytes(b'preserve')
        self.assert_denied('DESTINATION_EXISTS', '--extract-to', str(destination))
        self.assertEqual(sentinel.read_bytes(), b'preserve')

    def test_extracting_into_source_repo_is_rejected(self):
        self.snapshot()
        destination = self.repo / 'restored'
        self.assert_denied('RESTORE_MUST_BE_EXTERNAL', '--extract-to', str(destination))
        self.assertFalse(destination.exists())

    def test_wrong_expected_source_cannot_be_repaired_by_lookup(self):
        self.snapshot()
        self.assert_denied('SOURCE_COMMIT_MISMATCH', '--source', '0' * 40)
        self.assert_denied('EXPECTED_SOURCE_FORMAT', '--source', 'main')

    def test_a_tree_oid_is_not_a_source_commit(self):
        self.snapshot()
        tree = subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=self.repo, text=True).strip()
        self.document['source']['commit'] = tree
        self.save_manifest()
        self.assert_denied('SOURCE_OBJECT_NOT_COMMIT')

    def test_manifest_cannot_omit_a_git_member(self):
        self.snapshot()
        self.document['entries'].pop()
        self.document['entry_count'] -= 1
        self.document['total_bytes'] = sum(x['bytes'] for x in self.document['entries'])
        self.save_manifest()
        self.assert_denied('SOURCE_INVENTORY_MISMATCH')

    def test_unknown_and_duplicate_json_fields_are_rejected(self):
        self.snapshot()
        self.document['allow_root'] = '/'
        self.save_manifest()
        self.assert_denied('MANIFEST_FIELDS')
        self.manifest.write_text('{"schema":"x","schema":"y"}')
        self.assert_denied('DUPLICATE_JSON_KEY')

    def test_wrong_mode_type_and_boolean_count_fail_closed(self):
        self.snapshot()
        original = copy.deepcopy(self.document)
        self.document['entries'][0]['mode'] = {}
        self.save_manifest()
        self.assert_denied('ENTRY_IDENTITY')
        self.document = original
        self.document['entry_count'] = True
        self.save_manifest()
        self.assert_denied('ENTRY_COUNT')

    def test_create_refuses_to_overwrite_an_existing_manifest(self):
        self.snapshot()
        before = self.archive.read_bytes(), self.manifest.read_bytes()
        self.assert_denied('MANIFEST_EXISTS', '--create', '--source', self.sha)
        self.assertEqual(before, (self.archive.read_bytes(), self.manifest.read_bytes()))

    def test_versioned_manifest_uses_external_default_scratch(self):
        self.snapshot()
        versioned = self.repo / 'manifest.json'
        versioned.write_bytes(self.manifest.read_bytes())
        result = subprocess.run([sys.executable, str(ROOT / 'scripts/verify_quarantine_manifest.py'), '--repo', str(self.repo), '--manifest', str(versioned), '--archive', str(self.archive), '--source', self.sha], text=True, capture_output=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        self.assertTrue(json.loads(result.stdout)['restore_readback'])

    def test_unsorted_or_duplicate_paths_fail_closed(self):
        self.snapshot()
        original = copy.deepcopy(self.document)
        self.document['entries'].reverse()
        self.save_manifest()
        self.assert_denied('UNSORTED_MANIFEST_PATHS')
        self.document = original
        self.document['entries'][1] = self.document['entries'][0]
        self.save_manifest()
        self.assert_denied('DUPLICATE_MANIFEST_PATH')

    def test_unknown_pax_extension_is_rejected(self):
        self.snapshot()
        def change(members):
            members[0][0].pax_headers['comment'] = 'unsupported'
            return members
        self.rewrite_archive(change)
        self.assert_denied('UNSUPPORTED_TAR_EXTENSION')

    def test_existing_archive_is_not_clobbered(self):
        archive = self.out / ('target_schema_gen-' + self.sha + '.tar.xz')
        archive.write_bytes(b'keep-existing')
        self.assert_denied('ARCHIVE_EXISTS', '--create', '--source', self.sha)
        self.assertEqual(archive.read_bytes(), b'keep-existing')
        self.assertFalse(self.manifest.exists())

    def test_failed_creation_does_not_publish_final_artifacts(self):
        from unittest.mock import patch
        spec = importlib.util.spec_from_file_location('quarantine_helper', ROOT / 'scripts/verify_quarantine_manifest.py')
        assert spec is not None and spec.loader is not None
        helper = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(helper)
        with patch.object(helper, 'restore_check', side_effect=helper.QuarantineError('INJECTED_READBACK_FAILURE')):
            with self.assertRaisesRegex(helper.QuarantineError, 'INJECTED_READBACK_FAILURE'):
                helper.create(self.repo, self.sha, self.manifest)
        self.assertEqual(list(self.out.iterdir()), [])

    def test_dirty_target_is_not_silently_archived(self):
        (self.repo / PREFIX / 'binary').write_bytes(b'dirty')
        self.assert_denied('SOURCE_WORKTREE_DIRTY', '--create', '--source', self.sha)
        self.assertFalse(self.manifest.exists())


if __name__ == '__main__':
    unittest.main()
