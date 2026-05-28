#!/usr/bin/env python3
"""Conservative script-ref checker for included source trees."""
from pathlib import Path
import re, sys
root = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
errors = []
patterns = [
    re.compile(r"(?:^|\s)(?:python3?|bash|sh|zsh)\s+([A-Za-z0-9_./\-]+\.(?:py|sh|bash|zsh))(?:\s|$)"),
    re.compile(r"(?:^|\s)(?:source|\.)\s+([A-Za-z0-9_./\-]+\.sh)(?:\s|$)"),
]
for script in sorted(list(root.rglob("*.sh")) + list(root.rglob("*.bash")) + list(root.rglob("*.zsh"))):
    rel = script.relative_to(root).as_posix()
    if any(part in {"target", ".git"} for part in script.parts):
        continue
    if rel.startswith("docs/codex-runs/archive/"):
        continue
    text = script.read_text(errors="replace")
    for lineno, line in enumerate(text.splitlines(), 1):
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        for pat in patterns:
            for m in pat.finditer(stripped):
                ref = m.group(1)
                candidates = [(script.parent / ref).resolve(), (root / ref).resolve()]
                if not any(c.exists() for c in candidates):
                    errors.append(f"{rel}:{lineno}: missing script ref {ref}")
if errors:
    print("FAIL: script references missing:", file=sys.stderr)
    for e in errors[:200]: print("  " + e, file=sys.stderr)
    sys.exit(2)
print("ok: script references resolve")
