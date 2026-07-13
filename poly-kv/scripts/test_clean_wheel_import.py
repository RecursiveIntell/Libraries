import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).parents[1]


class CleanWheelImportTest(unittest.TestCase):
    def test_built_wheel_imports_outside_source_tree(self):
        maturin = shutil.which("maturin")
        if maturin is None:
            self.skipTest("maturin executable is not installed")
        with tempfile.TemporaryDirectory() as tmp:
            tmp = Path(tmp)
            wheels = tmp / "wheels"
            target = tmp / "site"
            wheels.mkdir()
            subprocess.run(
                [maturin, "build", "--out", str(wheels)],
                cwd=ROOT,
                check=True,
            )
            wheel = next(wheels.glob("*.whl"))
            subprocess.run([sys.executable, "-m", "pip", "install", "--no-deps", "--target", str(target), str(wheel)], check=True)
            env = os.environ.copy()
            env["PYTHONPATH"] = str(target)
            smoke = """
import poly_kv
assert poly_kv.native_available()
shape = poly_kv.ShapeV2(
    batch=1,
    layers=1,
    num_q_heads=1,
    num_kv_heads=1,
    seq_len=2,
    head_dim=4,
)
built = poly_kv.build_synthetic_pool(shape)
assert built["manifest"]
decoded = poly_kv.decode_synthetic_slice(
    shape, role="key", layer=0, start=0, end=1
)
assert decoded["data_len"] > 0
assert decoded["receipt"]
"""
            subprocess.run(
                [sys.executable, "-c", smoke],
                cwd=tmp,
                env=env,
                check=True,
            )


if __name__ == "__main__":
    unittest.main()
