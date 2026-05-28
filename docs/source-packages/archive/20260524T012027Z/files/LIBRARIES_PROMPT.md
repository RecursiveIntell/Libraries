# PROMPT — Libraries Closure Pass

## You Are Working On

The Libraries workspace (~125K lines, 30+ crates). It is in closure mode — convergence and consumption-readiness, not new architecture.

## Read First
```
cat LIBRARIES_CLAUDE.md
cat Cargo.toml
cat STATUS_DASHBOARD.md
cat CONFORMANCE_GATES.md
```

## Task 1: Enable forge-governance by Default in Recall

**Why:** The governance observation pipeline (6 surfaces: effect preflight, assurance readiness, authority delegation, continuity incidents, constitutional amendments, mechanism fit) is fully implemented but gated behind an optional feature flag. It should be the default.

**File:** `recall-session/Cargo.toml`
```toml
[features]
default = ["preview", "forge-governance"]
```

**Verify:** `cargo test -p recall-session --features forge-governance` passes.

## Task 2: CI-Enforce Schema Compatibility

**File:** `.github/workflows/ci.yml` or `Makefile`

Add:
```bash
python scripts/check_v25_json_surface.py
bash scripts/check_schema_registry_uniqueness.sh
bash scripts/check_schema_compat.sh
```

These scripts already exist — they just need CI integration.

## Task 3: Update Supported-Lane Manifest

**File:** `support_lane.toml`, `STATUS_DASHBOARD.md`

Verify each crate's maturity level matches reality:
- `semantic-memory`: production
- `knowledge-runtime`: production
- `llm-tool-runtime`: production
- `forge-pilot`: production
- `verification-*`: production (control, policy) / beta (adjudication, calibration)
- `effect-runtime`: beta
- `stack-ids`: production
- `Primitives`: excluded (separately governed)

## Task 4: Document Primitives Policy

**File:** `Primitives/README.md`

Add a section explaining:
- Primitives are excluded from the default workspace
- They have different unwrap/unsafe policies (77 unwraps in cea-sqlite, unsafe in check-runner)
- This is intentional — they're sandbox/experimental crates
- They do not represent the workspace's quality standard

## Task 5: Attestation Exchange Scope Note

**File:** `forge-pilot/src/governance_gate.rs` or `SCOPE_NOTES.md`

Document GOV-002: attestation-exchange is wired but not consumed by the governance observation pipeline. Planned for V2. Not a gap — it's informational, not execution-gating.

## DO NOT
- Add new crates to the workspace
- Expand artifact families
- Modify stack-ids, semantic-memory, or llm-tool-runtime APIs
- Let library work derail Recall product closure
