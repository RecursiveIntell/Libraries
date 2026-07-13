import importlib.util
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "check_path_dependency_versions.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("checker", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class PathDependencyVersionTests(unittest.TestCase):
    def test_release_preflight_wires_path_dependency_checker(self):
        preflight = SCRIPT.parent / "release_preflight.sh"
        text = preflight.read_text(encoding="utf-8")
        self.assertIn("check_path_dependency_versions.py", text)

    def test_reports_only_mismatches_for_normal_and_target_dependencies(self):
        checker = load_checker()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "dep").mkdir()
            (root / "dep" / "Cargo.toml").write_text('[package]\nname="dep"\nversion="2.0.0"\n')
            (root / "Cargo.toml").write_text(
                '[package]\nname="app"\nversion="1.0.0"\n'
                '[dependencies]\ndep={path="dep", version="1.0.0"}\n'
                "[target.'cfg(unix)'.dev-dependencies]\n"
                'ok={path="dep", package="dep", version="2.0.0"}\n'
            )
            mismatches = checker.find_mismatches(root)
            self.assertEqual(len(mismatches), 1)
            self.assertEqual(mismatches[0].declared_version, "1.0.0")
            self.assertEqual(mismatches[0].actual_version, "2.0.0")

    def test_workspace_inherited_dependency_is_checked(self):
        checker = load_checker()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "dep").mkdir()
            (root / "app").mkdir()
            (root / "dep" / "Cargo.toml").write_text('[package]\nname="dep"\nversion="0.2.0"\n')
            (root / "Cargo.toml").write_text(
                '[workspace]\nmembers=["app", "dep"]\n'
                '[workspace.dependencies]\ndep={path="dep", version="0.1.0"}\n'
            )
            (root / "app" / "Cargo.toml").write_text(
                '[package]\nname="app"\nversion="0.1.0"\n[dependencies]\ndep={workspace=true}\n'
            )
            self.assertEqual(len(checker.find_mismatches(root)), 1)

    def test_semver_compatible_requirement_is_accepted(self):
        checker = load_checker()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "dep").mkdir()
            (root / "dep" / "Cargo.toml").write_text('[package]\nname="dep"\nversion="1.4.2"\n')
            (root / "Cargo.toml").write_text(
                '[package]\nname="app"\nversion="1.0.0"\n'
                '[dependencies]\ndep={path="dep", version="^1.2"}\n'
            )
            self.assertEqual(checker.find_mismatches(root), [])

    def test_exact_prerelease_requirement_is_accepted(self):
        checker = load_checker()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "dep").mkdir()
            (root / "dep" / "Cargo.toml").write_text(
                '[package]\nname="dep"\nversion="0.1.0-beta.4"\n'
            )
            (root / "Cargo.toml").write_text(
                '[package]\nname="app"\nversion="1.0.0"\n'
                '[dependencies]\ndep={path="dep", version="0.1.0-beta.4"}\n'
            )
            self.assertEqual(checker.find_mismatches(root), [])

    def test_nested_workspace_dependencies_are_checked(self):
        checker = load_checker()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            nested = root / "nested"
            (nested / "dep").mkdir(parents=True)
            (nested / "dep" / "Cargo.toml").write_text('[package]\nname="dep"\nversion="0.4.0"\n')
            (nested / "Cargo.toml").write_text(
                '[workspace]\nmembers=["dep"]\n'
                '[workspace.dependencies]\ndep={path="dep", version="0.3"}\n'
            )
            mismatches = checker.find_mismatches(root)
            self.assertEqual(len(mismatches), 1)
            self.assertEqual(mismatches[0].manifest, Path("nested/Cargo.toml"))

    def test_unversioned_path_dependency_requires_explicit_unpublished_policy(self):
        checker = load_checker()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "dep").mkdir()
            (root / "dep" / "Cargo.toml").write_text('[package]\nname="dep"\nversion="1.0.0"\n')
            (root / "Cargo.toml").write_text(
                '[package]\nname="app"\nversion="1.0.0"\n'
                '[dependencies]\ndep={path="dep"}\n'
            )
            mismatches = checker.find_mismatches(root)
            self.assertEqual(len(mismatches), 1)
            self.assertEqual(mismatches[0].declared_version, "<missing>")

            (root / "Cargo.toml").write_text(
                '[package]\nname="app"\nversion="1.0.0"\npublish=false\n'
                '[dependencies]\ndep={path="dep"}\n'
            )
            self.assertEqual(checker.find_mismatches(root), [])

    def test_archived_salvage_manifests_are_not_release_inputs(self):
        checker = load_checker()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            archived = root / "_salvage_from_libraries2" / "snapshot"
            dependency = root / "dep"
            archived.mkdir(parents=True)
            dependency.mkdir()
            dependency.joinpath("Cargo.toml").write_text(
                '[package]\nname="dep"\nversion="1.0.0"\n'
            )
            archived.joinpath("Cargo.toml").write_text(
                '[package]\nname="archive"\nversion="1.0.0"\n'
                '[dependencies]\ndep={path="../../dep"}\n'
            )
            self.assertEqual(checker.find_mismatches(root), [])


if __name__ == "__main__":
    unittest.main()
