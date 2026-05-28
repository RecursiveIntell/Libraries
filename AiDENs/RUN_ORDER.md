# P23 Run Order

Execute in order. Paste the matching manual guardrail between phases.

1. `prompts/P23_CODEX_RUN_PROMPT.md`
2. `prompts/phases/PHASE_00_PREFLIGHT_HOSTILE_AUDIT.md`
3. Manual guardrail: `phase_injections/PHASE_00_TO_01_REVALIDATION.md`
4. `prompts/phases/PHASE_01_ZPY_TOTAL_CLOSURE.md`
5. Manual guardrail: `phase_injections/PHASE_01_TO_02_REVALIDATION.md`
6. `prompts/phases/PHASE_02_ARCHIVE_CLASSIFICATION_AND_REPO_HYGIENE.md`
7. Manual guardrail: `phase_injections/PHASE_02_TO_03_REVALIDATION.md`
8. `prompts/phases/PHASE_03_PACKAGE_MODE_AND_REPLAY_VERIFIER.md`
9. Manual guardrail: `phase_injections/PHASE_03_TO_04_REVALIDATION.md`
10. `prompts/phases/PHASE_04_AGENT_RUN_CAPABILITY_VERTICAL_SLICE.md`
11. Manual guardrail: `phase_injections/PHASE_04_TO_05_REVALIDATION.md`
12. `prompts/phases/PHASE_05_OPERATOR_CLI_AND_SUPPORT_TIER_PRODUCT_FLOW.md`
13. Manual guardrail: `phase_injections/PHASE_05_TO_06_REVALIDATION.md`
14. `prompts/phases/PHASE_06_PROVENANCE_RECEIPT_AND_EXECUTION_CONTEXT_HARDENING.md`
15. Manual guardrail: `phase_injections/PHASE_06_TO_07_REVALIDATION.md`
16. `prompts/phases/PHASE_07_TEST_MATRIX_CI_AND_HOSTILE_ASSERTIONS.md`
17. Manual guardrail: `phase_injections/PHASE_07_TO_08_REVALIDATION.md`
18. `prompts/phases/PHASE_08_FINAL_REPLAY_AUDIT_AND_HANDOFF.md`

## Final required commands

```bash
bash scripts/p23_verify.sh
P23_REQUIRE_CARGO=1 bash scripts/p23_verify.sh
python3 z.py --root . --profile aidens --mode codex-context --codex-current-run P23 --strict --dry-run
python3 z.py --root . --profile aidens --mode release-context --codex-current-run P23 --strict --dry-run
python3 z.py --root . --profile aidens --mode next-codex-context --codex-current-run P23 --strict --dry-run
python3 z.py --root . --profile aidens --mode audit-full --include-codex-archive --codex-current-run P23 --strict --dry-run
```

If `release-context` / `next-codex-context` modes do not exist yet, P23 must implement them or document an explicit equivalent with tests.
