# Claude Findings Normalized Against Current Snapshot

This doc maps the included Claude assessment into current live issues.

## Findings kept as live

### 1. Oracle combinatorial growth
Still live.
Mapped to:
- `ORC-001`
- `ORC-002`

### 2. Digest code duplication
Still live.
Mapped to:
- `DIG-001`

### 3. V2→V3 sentinel digest fallback
Still live.
Mapped to:
- `DIG-002`

### 4. ForgeStore single mutex / concurrency ambiguity
Still live.
Mapped to:
- `LIV-001`
- `LIV-002`

### 5. Workspace member depending on excluded crates
Still live.
Mapped to:
- `CI-001`
- `CONF-002`

### 6. Compiler-path unwrap in graph hash path
Still live.
Mapped to:
- `CC-002`

## Findings that need reframing rather than literal adoption

### “The kernel lane does not really exist yet”
Outdated.
The current snapshot clearly has a real kernel lane.
The live issue is **hardening and modularity**, not existence.

### “The exporter is still basically a toy”
Outdated as phrased.
The exporter is materially richer now.
The live issue is **coverage completeness and grouped semantics**, not “exporter absent.”

## Net result

Claude’s assessment is still useful, but it must be treated as a **finding source**, not as a literal live status dashboard.
