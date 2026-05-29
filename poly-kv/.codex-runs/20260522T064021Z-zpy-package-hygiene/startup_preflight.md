== poly-kv next-pass preflight ==
run_id: 20260522T064021Z-zpy-package-hygiene
date_utc: 2026-05-22T06:47:21Z
pwd: /home/sikmindz/Coding/Libraries/poly-kv
/home/sikmindz/Coding/Libraries/poly-kv
master
f2d992f4eca6940a1d16a18deb5b5a44b32bd7c0
 D .codex/hooks.json
 D .codex/hooks/post_tool_use_receipt.py
 D .codex/hooks/pre_tool_use_policy.py
 D .codex/hooks/stop_final_gate.py
 D .codex/hooks/user_prompt_submit.py
 M BUNDLE_MANIFEST.json
 M README.md
 M codex/prompts/MASTER_PROMPT.md
 D docs/CODEX_INPUT_BRIEF.md
 M docs/README_DRAFT.md
?? .codex-runs/
?? Cargo.lock
?? Cargo.toml
?? README_BUNDLE.md
?? __pycache__/
?? codex/manual-injections/
?? codex/prompts/PHASE_00_PREFLIGHT_AND_INVENTORY.md
?? codex/prompts/PHASE_01_FIX_ALLOWLIST_AND_COMMAND_LOGS.md
?? codex/prompts/PHASE_01_SHAPE_AND_CONTRACTS.md
?? codex/prompts/PHASE_02_RECEIPTS_AND_ACCOUNTING.md
?? codex/prompts/PHASE_02_ROOT_PACKAGE_ARCHIVE.md
?? codex/prompts/PHASE_03_CORE_API_COMPAT.md
?? codex/prompts/PHASE_03_STRICT_HYGIENE_GATES.md
?? codex/prompts/PHASE_04_PYTHON_SIDECAR_SKELETON.md
?? codex/prompts/PHASE_04_VALIDATORS_AND_REGRESSION_TESTS.md
?? codex/prompts/PHASE_05_PACKAGE_AND_VERIFY.md
?? codex/prompts/PHASE_05_PYTHON_INTEROP_RECEIPTS.md
?? codex/prompts/PHASE_06_BENCHMARK_HARNESS.md
?? codex/prompts/PHASE_06_DOCS_AND_CLAIM_BOUNDARY.md
?? codex/prompts/PHASE_07_DOCS_AND_CLAIMS.md
?? codex/prompts/PHASE_07_FINAL_AUDIT.md
?? codex/prompts/PHASE_08_VALIDATION.md
?? codex/prompts/PHASE_09_FINAL_AUDIT.md
?? crates/
?? docs/BENCHMARK_AND_HARNESS_SPEC.md
?? docs/BENCHMARK_TIERS.md
?? docs/CLAIM_BOUNDARY.md
?? docs/CURRENT_PACKAGE_HYGIENE_AUDIT.md
?? docs/CURRENT_STATE_AUDIT.md
?? docs/FINAL_STATE.md
?? docs/HOSTILE_AUDITOR_HANDOFF_TEMPLATE.md
?? docs/ISSUE_MATRIX.md
?? docs/NEXT_RELEASE_PLAN.md
?? docs/PACKAGE_HYGIENE_CLASSIFICATION_POLICY.md
?? docs/PY_SIDECAR_SPEC.md
?? docs/ROLLBACK_AND_QUARANTINE_PLAN.md
?? docs/SOURCE_OF_TRUTH_MAP_NEXT.md
?? docs/TARGET_FINAL_STATE.md
?? docs/VALIDATION_GATES.md
?? docs/ZPY_FIX_SPEC.md
?? docs/codex-runs/
?? patches/
?? poly-kv-generic-rust-next-codex-context-20260520.codex-archive.json
?? poly-kv-generic-rust-next-codex-context-20260520.excluded.json
?? poly-kv-generic-rust-next-codex-context-20260520.findings.json
?? poly-kv-generic-rust-next-codex-context-20260520.manifest.json
?? poly-kv-generic-rust-next-codex-context-20260520.report.md
?? poly-kv-generic-rust-next-codex-context-20260520.zip
?? poly-kv-generic-rust-next-codex-context-20260522.codex-archive.json
?? poly-kv-generic-rust-next-codex-context-20260522.excluded.json
?? poly-kv-generic-rust-next-codex-context-20260522.findings.json
?? poly-kv-generic-rust-next-codex-context-20260522.manifest.json
?? poly-kv-generic-rust-next-codex-context-20260522.report.md
?? poly-kv-generic-rust-next-codex-context-20260522.zip
?? pyproject.toml
?? python/
?? scripts/__pycache__/
?? scripts/assert_no_boundary_drift.py
?? scripts/assert_python_sidecar_layout.py
?? scripts/assert_realized_accounting.py
?? scripts/assert_receipt_integrity.py
?? scripts/assert_source_package_hygiene.py
?? scripts/bench_boundary.py
?? scripts/bench_rust_synthetic.py
?? scripts/build_handoff_package.sh
?? scripts/compare_receipts.py
?? scripts/preflight_next_pass.sh
?? scripts/run_next_validation.sh
?? scripts/test_zpy_hygiene_regression.py
?? z.py
-- rust --
rustc 1.93.0 (254b59607 2026-01-19) (Fedora 1.93.0-1.fc43)
cargo 1.93.0 (083ac5135 2025-12-15) (Fedora 1.93.0-1.fc43)
-- python --
Python 3.14.2
Python 3.14.2
-- manifests --
./Cargo.toml
./crates/poly-kv-python/Cargo.toml
./crates/poly-kv/Cargo.toml
./crates/quant-codec-core/Cargo.toml
./Cargo.lock
-- external path deps --
./crates/poly-kv/Cargo.toml:10:quant-codec-core = { path = "../quant-codec-core" }
./crates/poly-kv-python/Cargo.toml:14:poly-kv = { path = "../poly-kv" }
-- codex/control files --
./.agents/skills
./.agents/skills/recursiveintell-final-audit
./.agents/skills/recursiveintell-final-audit/SKILL.md
./.agents/skills/recursiveintell-phase-gate
./.agents/skills/recursiveintell-phase-gate/SKILL.md
./.agents/skills/recursiveintell-public-claim-boundary
./.agents/skills/recursiveintell-public-claim-boundary/SKILL.md
./.agents/skills/recursiveintell-rust-crate-boundary
./.agents/skills/recursiveintell-rust-crate-boundary/SKILL.md
./.agents/skills/recursiveintell-startup-preflight
./.agents/skills/recursiveintell-startup-preflight/SKILL.md
./.codex-runs/20260520T174516Z-alpha1
./.codex-runs/20260520T174516Z-alpha1/changed_files.txt
./.codex-runs/20260520T174516Z-alpha1/commands_run.log
./.codex-runs/20260520T174516Z-alpha1/final_audit_report.md
./.codex-runs/20260520T174516Z-alpha1/invariant_report.md
./.codex-runs/20260520T174516Z-alpha1/phase_00_report.md
./.codex-runs/20260520T174516Z-alpha1/phase_01_report.md
./.codex-runs/20260520T174516Z-alpha1/phase_02_report.md
./.codex-runs/20260520T174516Z-alpha1/phase_03_report.md
./.codex-runs/20260520T174516Z-alpha1/phase_04_report.md
./.codex-runs/20260520T174516Z-alpha1/phase_05_report.md
./.codex-runs/20260520T174516Z-alpha1/phase_06_report.md
./.codex-runs/20260520T174516Z-alpha1/phase_07_report.md
./.codex-runs/20260520T174516Z-alpha1/phase_08_report.md
./.codex-runs/20260520T174516Z-alpha1/phase_09_report.md
./.codex-runs/20260520T174516Z-alpha1/remaining_delta.md
./.codex-runs/20260520T174516Z-alpha1/risk_register.md
./.codex-runs/20260520T174516Z-alpha1/rollback_plan.md
./.codex-runs/20260520T174516Z-alpha1/source_inventory.md
./.codex-runs/20260520T174516Z-alpha1/startup_preflight.md
./.codex-runs/20260520T174516Z-alpha1/validation_results.md
./.codex-runs/20260522T045320Z-poly-kv-next
./.codex-runs/20260522T045320Z-poly-kv-next/cargo_manifests.txt
./.codex-runs/20260522T045320Z-poly-kv-next/changed_files.txt
./.codex-runs/20260522T045320Z-poly-kv-next/commands_run.log
./.codex-runs/20260522T045320Z-poly-kv-next/commit_before.txt
./.codex-runs/20260522T045320Z-poly-kv-next/final_audit_report.md
./.codex-runs/20260522T045320Z-poly-kv-next/git_diff_stat.txt
./.codex-runs/20260522T045320Z-poly-kv-next/git_status_after.txt
./.codex-runs/20260522T045320Z-poly-kv-next/git_status_before.txt
./.codex-runs/20260522T045320Z-poly-kv-next/invariant_report.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_00_guardrail.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_00_report.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_01_guardrail.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_01_report.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_02_guardrail.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_02_report.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_03_guardrail.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_03_report.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_04_guardrail.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_04_report.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_05_guardrail.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_05_report.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_06_guardrail.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_06_report.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_07_guardrail.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_07_report.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_08_guardrail.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_08_report.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_09_guardrail.md
./.codex-runs/20260522T045320Z-poly-kv-next/phase_09_report.md
./.codex-runs/20260522T045320Z-poly-kv-next/python_boundary_bench.json
./.codex-runs/20260522T045320Z-poly-kv-next/receipt_parity_report.json
./.codex-runs/20260522T045320Z-poly-kv-next/remaining_delta.md
./.codex-runs/20260522T045320Z-poly-kv-next/risk_register.md
./.codex-runs/20260522T045320Z-poly-kv-next/rollback_plan.md
./.codex-runs/20260522T045320Z-poly-kv-next/rust_synthetic_bench.json
./.codex-runs/20260522T045320Z-poly-kv-next/source_inventory.txt
./.codex-runs/20260522T045320Z-poly-kv-next/startup_preflight.md
./.codex-runs/20260522T045320Z-poly-kv-next/touched_diff.patch
./.codex-runs/20260522T045320Z-poly-kv-next/validation_results.md
./.codex-runs/20260522T064021Z-zpy-package-hygiene
./.codex-runs/20260522T064021Z-zpy-package-hygiene/commands_run.log
./.codex-runs/20260522T064021Z-zpy-package-hygiene/phase_00_report.md
./.codex-runs/20260522T064021Z-zpy-package-hygiene/phase_01_report.md
./.codex-runs/20260522T064021Z-zpy-package-hygiene/phase_02_report.md
./.codex-runs/20260522T064021Z-zpy-package-hygiene/phase_03_report.md
./.codex-runs/20260522T064021Z-zpy-package-hygiene/phase_04_report.md
./.codex-runs/20260522T064021Z-zpy-package-hygiene/startup_preflight.md
./.codex-runs/_hook_receipts
./.codex-runs/_hook_receipts/2026-05-20T174451Z-user_prompt_submit-dba0e178e0a5a790.json
./.codex-runs/_hook_receipts/2026-05-20T174500Z-pre_tool_use_policy-035627457dc6f581.json
./.codex-runs/_hook_receipts/2026-05-20T174501Z-post_tool_use-6aa1ab2789bd674a.json
./.codex-runs/_hook_receipts/2026-05-20T174511Z-post_tool_use-38d18ddeaf086242.json
./.codex-runs/_hook_receipts/2026-05-20T174511Z-pre_tool_use_policy-f198b58f93382f40.json
./.codex-runs/_hook_receipts/2026-05-20T174516Z-post_tool_use-599a0b232e0c9588.json
./.codex-runs/_hook_receipts/2026-05-20T174516Z-pre_tool_use_policy-3796753877d36fef.json
./.codex-runs/_hook_receipts/2026-05-20T174526Z-post_tool_use-81723686f0fdf5ae.json
./.codex-runs/_hook_receipts/2026-05-20T174526Z-pre_tool_use_policy-a34ff28a68d7551c.json
./.codex-runs/_hook_receipts/2026-05-20T174543Z-pre_tool_use_policy-ad9a5ba1dd3d4445.json
./.codex-runs/_hook_receipts/2026-05-20T174544Z-post_tool_use-9eb3e0150522598b.json
./.codex-runs/_hook_receipts/2026-05-20T174606Z-post_tool_use-6959f625d129ab7d.json
./.codex-runs/_hook_receipts/2026-05-20T174606Z-pre_tool_use_policy-d819ce6371d22969.json
./.codex-runs/_hook_receipts/2026-05-20T174616Z-pre_tool_use_policy-2af69928fc77fcdb.json
./.codex-runs/_hook_receipts/2026-05-20T174617Z-post_tool_use-efc02c8237af1376.json
./.codex-runs/_hook_receipts/2026-05-20T174631Z-post_tool_use-988e60747ba956c3.json
./.codex-runs/_hook_receipts/2026-05-20T174631Z-pre_tool_use_policy-653c36d7c1d267cc.json
./.codex-runs/_hook_receipts/2026-05-20T174807Z-post_tool_use-7916329f0383d5cb.json
./.codex-runs/_hook_receipts/2026-05-20T174807Z-pre_tool_use_policy-25f12939a16bbce1.json
./.codex-runs/_hook_receipts/2026-05-20T174941Z-post_tool_use-8c166ceb29312026.json
./.codex-runs/_hook_receipts/2026-05-20T174941Z-pre_tool_use_policy-07fd48af4e1f7e93.json
./.codex-runs/_hook_receipts/2026-05-20T175051Z-post_tool_use-a1edf978462a5e04.json
./.codex-runs/_hook_receipts/2026-05-20T175051Z-pre_tool_use_policy-4fc37ca91c4130f0.json
./.codex-runs/_hook_receipts/2026-05-20T175057Z-pre_tool_use_policy-d328c4c8ad5a1c0d.json
./.codex-runs/_hook_receipts/2026-05-20T175105Z-post_tool_use-b809d97c72cd7ae8.json
./.codex-runs/_hook_receipts/2026-05-20T175111Z-post_tool_use-40d8b54de3179126.json
./.codex-runs/_hook_receipts/2026-05-20T175111Z-pre_tool_use_policy-10a93fc29604eead.json
./.codex-runs/_hook_receipts/2026-05-20T175158Z-post_tool_use-e5a29e96da35f53b.json
./.codex-runs/_hook_receipts/2026-05-20T175158Z-pre_tool_use_policy-8158d6c378a44fb1.json
./.codex-runs/_hook_receipts/2026-05-20T175203Z-pre_tool_use_policy-18dd378e17592beb.json
./.codex-runs/_hook_receipts/2026-05-20T175206Z-post_tool_use-79d96f7331b73afc.json
./.codex-runs/_hook_receipts/2026-05-20T175206Z-pre_tool_use_policy-49ec5e17ff9e3f07.json
./.codex-runs/_hook_receipts/2026-05-20T175210Z-post_tool_use-daf80d3074ab77a5.json
./.codex-runs/_hook_receipts/2026-05-20T175210Z-pre_tool_use_policy-241fa5787959d955.json
./.codex-runs/_hook_receipts/2026-05-20T175212Z-pre_tool_use_policy-886b066ebb556595.json
./.codex-runs/_hook_receipts/2026-05-20T175221Z-post_tool_use-455725bbb18e1660.json
./.codex-runs/_hook_receipts/2026-05-20T175226Z-post_tool_use-b7f4714cd7b4f6c9.json
./.codex-runs/_hook_receipts/2026-05-20T175226Z-pre_tool_use_policy-838c2536bdad07d6.json
./.codex-runs/_hook_receipts/2026-05-20T175232Z-post_tool_use-59398bcd6e2e3a0a.json
./.codex-runs/_hook_receipts/2026-05-20T175232Z-pre_tool_use_policy-5776356743659cc7.json
./.codex-runs/_hook_receipts/2026-05-20T175252Z-post_tool_use-b6ea6f4668756c13.json
./.codex-runs/_hook_receipts/2026-05-20T175252Z-pre_tool_use_policy-f6037829609e957d.json
./.codex-runs/_hook_receipts/2026-05-20T175301Z-post_tool_use-980e8b23e4a9c3a4.json
./.codex-runs/_hook_receipts/2026-05-20T175301Z-pre_tool_use_policy-c4bbd6be4805f7f2.json
./.codex-runs/_hook_receipts/2026-05-20T175306Z-pre_tool_use_policy-bf4dedf0031ac45e.json
./.codex-runs/_hook_receipts/2026-05-20T175309Z-post_tool_use-cdc9dc6e1254cf8b.json
./.codex-runs/_hook_receipts/2026-05-20T175315Z-post_tool_use-9a3fac5cc065fb1d.json
./.codex-runs/_hook_receipts/2026-05-20T175315Z-pre_tool_use_policy-14571fa726fb76b2.json
./.codex-runs/_hook_receipts/2026-05-20T175316Z-post_tool_use-09c33b0a95192949.json
./.codex-runs/_hook_receipts/2026-05-20T175322Z-post_tool_use-61fba960a4e90bd3.json
./.codex-runs/_hook_receipts/2026-05-20T175322Z-pre_tool_use_policy-80688a93f65c460a.json
./.codex-runs/_hook_receipts/2026-05-20T175328Z-post_tool_use-e06f78db5333d126.json
./.codex-runs/_hook_receipts/2026-05-20T175328Z-pre_tool_use_policy-9e361e3d6375fb10.json
./.codex-runs/_hook_receipts/2026-05-20T175332Z-pre_tool_use_policy-d9b852e856962aed.json
./.codex-runs/_hook_receipts/2026-05-20T175333Z-post_tool_use-a782736192ba2c1f.json
./.codex-runs/_hook_receipts/2026-05-20T175342Z-post_tool_use-a84c47d4ed070254.json
./.codex-runs/_hook_receipts/2026-05-20T175342Z-pre_tool_use_policy-f50c8de20822fd28.json
./.codex-runs/_hook_receipts/2026-05-20T175346Z-post_tool_use-7851b0e945184f54.json
./.codex-runs/_hook_receipts/2026-05-20T175346Z-pre_tool_use_policy-1c18dcbc23675d7c.json
./.codex-runs/_hook_receipts/2026-05-20T175351Z-pre_tool_use_policy-712a71c3d25689da.json
./.codex-runs/_hook_receipts/2026-05-20T175353Z-post_tool_use-f51ae596db0841b8.json
./.codex-runs/_hook_receipts/2026-05-20T175357Z-post_tool_use-0d26ed934028228d.json
./.codex-runs/_hook_receipts/2026-05-20T175357Z-pre_tool_use_policy-e426a838837020cc.json
./.codex-runs/_hook_receipts/2026-05-20T175401Z-post_tool_use-6721354de6b5004a.json
./.codex-runs/_hook_receipts/2026-05-20T175401Z-pre_tool_use_policy-579de9f745566fe2.json
./.codex-runs/_hook_receipts/2026-05-20T175406Z-post_tool_use-969ccb595a08df66.json
./.codex-runs/_hook_receipts/2026-05-20T175406Z-pre_tool_use_policy-4a289df30e2bbddc.json
./.codex-runs/_hook_receipts/2026-05-20T175504Z-post_tool_use-afcc153c907a80ce.json
./.codex-runs/_hook_receipts/2026-05-20T175504Z-pre_tool_use_policy-6fb9568a23cf7b4d.json
./.codex-runs/_hook_receipts/2026-05-20T175512Z-post_tool_use-cc3d72dc12d6a023.json
./.codex-runs/_hook_receipts/2026-05-20T175512Z-pre_tool_use_policy-c017b3ae9d4facc7.json
./.codex-runs/_hook_receipts/2026-05-20T175518Z-post_tool_use-f4154f232b6934cd.json
./.codex-runs/_hook_receipts/2026-05-20T175518Z-pre_tool_use_policy-7bfd6e7d756c3320.json
./.codex-runs/_hook_receipts/2026-05-20T175522Z-pre_tool_use_policy-0630e4852471c880.json
./.codex-runs/_hook_receipts/2026-05-20T175534Z-post_tool_use-cee888ea50e55665.json
./.codex-runs/_hook_receipts/2026-05-20T175536Z-pre_tool_use_policy-33fdaf69449da162.json
./.codex-runs/_hook_receipts/2026-05-20T175537Z-post_tool_use-f599890565749175.json
./.codex-runs/_hook_receipts/2026-05-20T175540Z-post_tool_use-b400f370ab525e43.json
./.codex-runs/_hook_receipts/2026-05-20T175540Z-pre_tool_use_policy-dae117e3e35f2fcd.json
./.codex-runs/_hook_receipts/2026-05-20T175600Z-stop_final_gate-6679d9c854e86016.json
./.codex-runs/_hook_receipts/2026-05-20T180700Z-user_prompt_submit-50fbab7d60f1f724.json
./.codex-runs/_hook_receipts/2026-05-20T180705Z-stop_final_gate-d5dacc4fad9f6bf1.json
./AGENTS.md
-- placeholders --
docs/codex-runs/archive/unclassified/20260520T180747Z/files/.codex/hooks/user_prompt_submit.py:34:bad = re.findall(r"@filename|\{feature\}|<placeholder>", text, flags=re.I)
scripts/check_forbidden_patterns.py:7:    ("todo", re.compile(r"\bTODO\b|\bFIXME\b|\bTBD\b")),
scripts/check_forbidden_patterns.py:8:    ("placeholder", re.compile(r"@filename|\{feature\}|<placeholder>", re.IGNORECASE)),
scripts/preflight_next_pass.sh:29:  grep -RIn "TODO\|FIXME\|TBD\|@filename\|{feature}\|<placeholder>" AGENTS.md README.md crates docs scripts python pyproject.toml 2>/dev/null || true
