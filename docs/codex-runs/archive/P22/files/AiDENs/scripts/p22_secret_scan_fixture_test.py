#!/usr/bin/env python3
"""P22 secret scanner fixture test for z.py.

Expected after P22:
- Rust field-copy of api_key does not emit false positive.
- Literal-looking secret still emits a warning/error.
- Secret-like filenames such as .env are still reported.
"""
from __future__ import annotations
import json, subprocess, sys, tempfile
from pathlib import Path


def main() -> int:
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        (root / "Cargo.toml").write_text("[workspace]\nresolver='2'\nmembers=[]\n", encoding="utf-8")
        (root / "src").mkdir()
        (root / "src" / "safe.rs").write_text("""
struct Provider { api_key: Option<String> }
fn f(provider: &Provider) -> Provider { Provider { api_key: provider.api_key.clone() } }
""", encoding="utf-8")
        (root / "src" / "bad.rs").write_text('fn f() { let api_key = "sk-test-hardcoded-secret"; }\n', encoding="utf-8")
        (root / ".env").write_text("OPENAI_API_KEY=sk-test-hardcoded-secret\n", encoding="utf-8")
        z = Path.cwd() / "z.py"
        result = subprocess.run([sys.executable, str(z), "--root", str(root), "--profile", "generic-rust", "--mode", "codex-context", "--strict", "--dry-run"], text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        print(result.stdout)
        findings_path = root / f"{root.name}-generic-rust-codex-context-{__import__('datetime').datetime.now(__import__('datetime').timezone.utc).strftime('%Y%m%d')}.findings.json"
        # The exact default output name may vary; search sidecars.
        findings_files = list(root.glob("*.findings.json"))
        if not findings_files:
            print("FAIL: no findings sidecar produced")
            return 1
        findings = json.loads(findings_files[0].read_text()).get("findings", [])
        if "sk-test-hardcoded-secret" in result.stdout:
            print("FAIL: literal secret value was printed to scanner output")
            return 1
        if any("safe.rs" in f.get("path", "") for f in findings):
            print("FAIL: safe Rust field-copy still reported as secret")
            return 1
        if not any("bad.rs" in f.get("path", "") and f.get("code") == "secret-content-openai-like-key" for f in findings):
            print("FAIL: literal secret fixture was not reported")
            return 1
        if not any(f.get("path") == ".env" and f.get("code") == "secret-like-filename" for f in findings):
            print("FAIL: secret-like .env filename was not reported")
            return 1
    print("PASS: secret scanner fixture behavior is correct")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
