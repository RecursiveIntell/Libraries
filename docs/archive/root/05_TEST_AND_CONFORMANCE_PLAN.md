# Test and Conformance Plan — V29

## Gate verification per issue

### TRUTH-001: Snapshot date unity
```bash
# Acceptance: all four files reference "20260330" or "2026-03-30"
grep -c '20260330\|2026-03-30' README.md SOURCE_BASIS.md STATUS_DASHBOARD.md PACK_MANIFEST.json
# Expected: each file returns ≥1 match
# Negative: no file references 20260323, 20260324, or 20260328
grep -c '20260323\|20260324\|20260328' README.md SOURCE_BASIS.md STATUS_DASHBOARD.md PACK_MANIFEST.json
# Expected: all return 0
```

### GATE-001: Permit type regex
```bash
python3 scripts/check_commit_permit_paths.py
# Expected: "commit permit path checks passed"
```

### DOC-002: README rewrite
```bash
# Acceptance: README does not contain "finish_pack" or "remediation"
grep -c 'finish_pack\|remediation' README.md
# Expected: 0
# README contains "RecursiveIntell" and "cargo build"
grep -c 'RecursiveIntell' README.md && grep -c 'cargo build\|cargo test' README.md
# Expected: ≥1 each
```

### TRUTH-002: Archive cleanup
```bash
# Root control doc count
ls *.md *.json 2>/dev/null | wc -l
# Expected: <20
# Supersession index exists
test -f docs/archive/SUPERSESSION_INDEX.md && echo "OK"
```

### TRUTH-003: Archive manifest
```bash
python3 scripts/check_root_archive_manifest.py
# Expected: passes
```

### GATE-002: Hotspot budgets
```bash
bash scripts/check_hotspot_budgets.sh
# Expected: "hotspot budget checks passed"
```

### WIRE-001: Serde rename_all
```bash
# Scan for serializable enums without rename_all
find . -name "*.rs" -not -path "./target*" -not -path "./Primitives/*" -not -name "*test*" | \
  xargs python3 -c "
import re, sys
for path in sys.argv[1:]:
    text = open(path).read()
    for d in re.finditer(r'#\[derive\(([^)]*Serialize[^)]*)\)\]', text):
        after = text[d.end():d.end()+500]
        m = re.search(r'(?:#\[serde\([^)]*\)\]\s*)*pub enum (\w+)', after)
        if m:
            ctx = text[d.start():d.end()+m.end()]
            if 'rename_all' not in ctx:
                print(f'{path}:{m.group(1)}')
" 2>/dev/null
# Expected: 0 output lines
# Then:
cargo check --workspace
cargo test --workspace
```

### DOC-001: Doc comment coverage
```bash
# Per-crate doc coverage check
for crate in semantic-memory forge-pilot knowledge-runtime effect-runtime forge-memory-bridge verification-control; do
  total=$(find "$crate/src" -name "*.rs" -not -name "*test*" | xargs grep -c 'pub struct\|pub enum\|pub trait' 2>/dev/null | awk -F: '{s+=$2}END{print s}')
  documented=$(find "$crate/src" -name "*.rs" -not -name "*test*" | xargs grep -B1 'pub struct\|pub enum\|pub trait' 2>/dev/null | grep '///' | wc -l)
  pct=$((100 * documented / total))
  echo "$crate: $pct%"
done
# Expected: all ≥80%
```

### WIRE-002: Error swallowing
```bash
# Count undocumented .ok() calls in production code
find . -name "*.rs" -not -path "./target*" -not -path "./Primitives/*" \
  -not -name "*test*" -not -path "*/tests/*" -not -path "*/examples/*" | \
  xargs grep '\.ok();' 2>/dev/null | grep -v 'INTENTIONAL' | wc -l
# Expected: 0 (all documented or replaced)
```

### CONV-001: HashMap convention
```bash
# Count undocumented HashMap in production code
find . -name "*.rs" -not -path "./target*" -not -path "./Primitives/*" \
  -not -name "*test*" -not -path "*/tests/*" | \
  xargs grep 'HashMap' 2>/dev/null | grep -v 'BTreeMap' | \
  grep -v 'CONVENTION EXCEPTION' | wc -l
# Expected: 0
```

## Full gate verification

After all issues are closed:
```bash
make gate                                    # All gate scripts pass
cargo check --workspace                      # Clean compilation
cargo test --workspace                       # All tests pass
cargo clippy --workspace -- -D warnings      # No warnings
cargo doc --workspace --no-deps 2>&1 | grep -c 'warning'  # Minimal doc warnings
```
