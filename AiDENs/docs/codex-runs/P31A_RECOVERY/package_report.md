# P31A Package Report

**Run:** P31A
**Timestamp:** 2026-05-29T06:51:18.134130Z
**Crates:** 34
**Changed files:** 404

## Verification Summary

| Gate | Result |
|------|--------|
| cargo check | ✅ PASS |
| cargo fmt | ✅ PASS |
| cargo clippy | ✅ PASS |
| cargo test | ✅ 429/429 |
| assert_release_ledger_schema | ✅ PASS |
| assert_release_truth_consistency | ✅ PASS |
| assert_support_claims_have_evidence | ✅ PASS |
| assert_adapter_delegation | ✅ PASS |
| assert_root_markdown_archive_policy | ✅ PASS |
| assert_codex_artifact_classification | ✅ PASS |

## Key Changes

1. **Provider status fix:** 4 cloud providers reclassified Unsupported→BoundaryUnavailable
2. **child.kill() honest failure:** terminate_timed_out_command returns bool, kill-failure receipt
3. **boundary-compiler-core:** merged from scaffold/ to crates/ workspace member
4. **Strict boundary compilation:** wired into parser-fallback path with receipt-bearing degradation
5. **Forbidden phrase fix:** negation-aware is_forbidden() in assertion scripts
6. **Tool call detection:** looks_like_tool_call_payload recognizes tool_call: text prefixes
7. **llm_tool_runtime delegation:** pub use re-export added to aidens-tool-kit
8. **Root markdown archive:** 65+ stale P24-P30 docs archived to P31A_archive/
