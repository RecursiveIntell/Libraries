#!/usr/bin/env python3
"""P28 regression check for z.py strict text/binary detection."""

from pathlib import Path
import importlib.util
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]
ZPY = ROOT / "z.py"


def load_zpy():
    spec = importlib.util.spec_from_file_location("aidens_zpy", ZPY)
    if spec is None or spec.loader is None:
        raise RuntimeError("unable to load z.py")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def main() -> int:
    zpy = load_zpy()
    with tempfile.TemporaryDirectory(prefix="aidens-p28-zpy-text-") as tmp:
        tmp = Path(tmp)
        good = tmp / "good.md"
        good.write_text("# ok\n", encoding="utf-8")
        if zpy.text_file_policy_reason(good) is not None:
            print("FAIL: valid UTF-8 text was rejected", file=sys.stderr)
            return 2

        invalid = tmp / "invalid.md"
        invalid.write_bytes(b"# invalid\n\xff\n")
        if zpy.text_file_policy_reason(invalid) != "non-utf8-text-file":
            print("FAIL: invalid UTF-8 text was not rejected", file=sys.stderr)
            return 2

        binary = tmp / "binary.md"
        binary.write_bytes(b"# binary\n\x00\n")
        if zpy.text_file_policy_reason(binary) != "binary-null-byte":
            print("FAIL: NUL-bearing text file was not rejected as binary", file=sys.stderr)
            return 2

    print("PASS: z.py strict text/binary detection rejects invalid text")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
