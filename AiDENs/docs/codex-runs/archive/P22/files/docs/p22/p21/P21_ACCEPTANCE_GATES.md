# P21 Acceptance Gates

## Global gate

Every phase must satisfy:

- no shadow truth;
- no local duplicate canonical ownership;
- no missing package fixtures/scripts;
- no deleted tests to hide failures;
- no fake provider/tool support;
- no silent widening;
- no agency bypass when `agency.enabled = true`;
- no scaffold promotion;
- all changes are accompanied by evidence reports.

## Phase gates

### Phase 00 — Package/source closure

Pass only if:

```bash
python3 scripts/p21_scan_package_integrity.py .
python3 scripts/p21_scan_source_cross_refs.py .
python3 scripts/p20_2_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl
bash scripts/p21_verify.sh
```

No missing `include_str!`, fixture, script, manifest, or expected event file references.

### Phase 01 — Build certification

Pass only if:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Logs must be written to `target/p21/phase01/` and summarized in `handoffs/p21/PHASE_01_BUILD_CERTIFICATION.md`.

### Phase 02 — Test-agent CLI

Pass only if:

```bash
cargo run -p aidens-cli -- run-test-agent fixtures/test-agent/basic-agent.toml
cargo test -p aidens-integration-tests test_agent_vertical_slice -- --nocapture
```

The command must emit output bundle files and receipts.

### Phase 03 — Generated agent project

Pass only if:

```bash
cargo run -p aidens-cli -- new coding-agent target/p21/example-coding-agent
cargo run -p aidens-cli -- run --config target/p21/example-coding-agent/aidens.toml "read README"
```

Generated project must include config, README, AGENT.md/tools/permits/receipts docs or equivalent.

### Phase 04 — Profiles and plan-kit

Pass only if:

```bash
cargo run -p aidens-cli -- profile list
cargo run -p aidens-cli -- profile explain coding-agent
cargo run -p aidens-cli -- profile explain chat-only
cargo run -p aidens-cli -- plan compile --config fixtures/test-agent/basic-agent.toml --out target/p21/basic-agent.plan.json
cargo run -p aidens-cli -- plan validate --config fixtures/test-agent/basic-agent.toml
```

### Phase 05 — Provider/tool certification

Pass only if provider and tool inspection commands produce machine-readable capability truth and no unsupported cloud/native-tool-loop claims.

### Phase 06 — Agency v0.2

Pass only if agency eval cases expand and runner cannot bypass agency when enabled.

### Phase 07 — Recall/Recall-Coding extraction

Pass only if reusable patterns are extracted into docs/templates/tests without importing app-specific assumptions into AiDENs core.

### Phase 08 — Archive replay

Pass only if a fresh release zip is created, unpacked into temp dir, and verified.

### Phase 09 — Guarded stretch

Pass only if mandatory phases are already green. Stretch work must remain bounded and revertible.

### Phase 10 — Final hostile audit

Pass only if final audit artifacts exist and list unsupported/deferred surfaces honestly.
