#!/usr/bin/env python3
"""P28 regression check for z.py safe_relative fail-closed behavior."""

from pathlib import Path
import importlib.util
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]
ZPY = ROOT / "z.py"


def load_zpy():
    spec = importlib.util.spec_from_file_location("aidens_zpy", ZPY)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def main() -> int:
    zpy = load_zpy()
    with tempfile.TemporaryDirectory(prefix="aidens-p28-safe-relative-") as tmp:
        base = Path(tmp)
        root = base / "root"
        outside = base / "outside"
        root.mkdir()
        outside.mkdir()
        inside = root / "inside.txt"
        inside.write_text("inside\n", encoding="utf-8")
        outside_file = outside / "outside.txt"
        outside_file.write_text("outside\n", encoding="utf-8")

        assert zpy.safe_relative(inside, root) == Path("inside.txt")

        try:
            zpy.safe_relative(outside_file, root)
        except ValueError:
            pass
        else:
            raise AssertionError("safe_relative accepted an outside path")

        symlink = root / "outside-link.txt"
        try:
            symlink.symlink_to(outside_file)
        except (OSError, NotImplementedError):
            return 0

        try:
            zpy.safe_relative(symlink, root)
        except ValueError:
            pass
        else:
            raise AssertionError("safe_relative accepted a symlink escape")

    print("PASS: z.py safe_relative fails closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
