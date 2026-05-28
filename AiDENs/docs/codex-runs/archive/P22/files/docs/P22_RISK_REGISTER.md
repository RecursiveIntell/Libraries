# P22 Risk Register

| Risk | Severity | Trigger | Mitigation | Gate |
|---|---:|---|---|---|
| z.py misarchives source/truth docs | High | loose regex or broad path match | protected allowlist + selftest fixtures | `p22_zpy_archival_selftest.py` |
| Existing archives rewritten | High | archive path collision | no-overwrite rule + collision suffix | archive manifest collision test |
| Normal package includes archive history | High | default include path too broad | package clean assertion | `assert_p22_release_package_clean.py` |
| Stale P20/P21 prompts remain active | High | missed path pattern | active hygiene assertion | `assert_p22_codex_archival_hygiene.py` |
| Secret scanner weakened | High | broad allowlist | fixture for field-copy and literal secret | `p22_secret_scan_fixture_test.py` |
| P22 docs overstate support | Medium | stretch work optimism | support-tier matrix and final audit | final hostile audit |
| Cargo gates skipped | Medium | env var not set | `P22_REQUIRE_CARGO=1` final command | p22 final gate |
| Audit logs lost | Medium | target ignored | final audit collection and explicit artifact path | final handoff |
| AiDENs becomes semantic owner | Critical | local replacements in wrappers | ownership map + no-shadow assertions | existing + P22 assertions |
