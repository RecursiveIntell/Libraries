# Zip Source Certifier Report

## Summary

- Script version: `2026.05.22-p31`
- Created UTC: `2026-05-25T17:08:34Z`
- Root: `/home/sikmindz/Coding`
- Archive root: `/home/sikmindz/Coding`
- Output: `/home/sikmindz/Coding/Libraries/docs/post-salvage-validation/sidecars/Coding-post-salvage-20260525.zip`
- Include roots: `1`
- External Cargo path dependency roots: `0`
- Profile: `research` requested as `research`
- Mode: `next-codex-context`
- Package role: `next-codex-context`
- Strict: `False`
- Dry run: `True`
- Included files: `92499`
- Included bytes: `1276475920`
- Excluded files: `77057`
- Pruned dirs: `257`
- Findings: `514` (`0` errors, `514` warnings)
- Content manifest SHA-256: `de0e668c6e66e6d47ef4ee7e83e59d6f2a26cf750fc98fe0df072b8140639444`
- Ecosystems detected: `rust, python`
- Codex archive enabled: `False`
- Codex archive planned: `0`
- Codex archive moved: `0`
- Codex active stale after normalization: `0`
- Root Markdown archive enabled: `False`
- Root Markdown inspected: `46`
- Root Markdown protected: `3`
- Root Markdown candidates: `4`
- Root Markdown ambiguous: `39`
- Root Markdown moved: `0`
- Root Markdown collisions: `0`
- Root package archive enabled: `False`
- Root package inspected: `65`
- Root package protected: `2`
- Root package candidates: `7`
- Root package moved: `0`
- Root package skipped existing: `0`
- Root package collisions: `0`

## Ecosystem parity

| Ecosystem | Detected | Manifests | Missing expected | Dry-run status |
|---|---:|---:|---:|---|
| `rust` | `True` | 2027 | 0 | `available-not-run` |
| `python` | `True` | 34 | 213 | `available-not-run` |
| `node` | `False` | 0 | 0 | `not-applicable` |
| `go` | `False` | 0 | 0 | `not-applicable` |
| `docker` | `False` | 0 | 0 | `not-applicable` |
| `git` | `False` | 0 | 0 | `not-applicable` |

## Decision provenance

- Decisions recorded: `169813`
- Includes: `92499`
- Excludes: `77057`
- Pruned dirs: `257`

## Validation findings

| Severity | Code | Path | Detail |
|---|---|---|---|
| warning | `cargo-path-dep-missing` | `Forge-Audit/Cargo.toml` | Cargo path dependency does not exist: ../Libraries/LLM-Pipeline |
| warning | `cargo-path-dep-missing` | `Libraries.BAK/semantic-memory/.semantic-memory-standalone/Cargo.toml` | Cargo path dependency does not exist: ../forge-memory-bridge |
| warning | `cargo-path-dep-missing` | `Libraries.BAK/semantic-memory/.semantic-memory-standalone/Cargo.toml` | Cargo path dependency does not exist: ../semantic-memory-forge |
| warning | `cargo-path-dep-missing` | `Libraries.BAK/semantic-memory/.semantic-memory-standalone/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/AI-Batch-Queue/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/ComfyUI-RS/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/LLM-Pipeline/Cargo.toml` | Cargo path dependency does not exist: ../living-memory/living-memory |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/LLM-Pipeline/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/Tauri-Queue/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/agent-graph/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/attestation-exchange/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/constraint-compiler/Cargo.toml` | Cargo path dependency does not exist: ../forge-memory-bridge |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/constraint-compiler/Cargo.toml` | Cargo path dependency does not exist: ../recursive-kernel-core |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/constraint-compiler/Cargo.toml` | Cargo path dependency does not exist: ../semantic-memory-forge |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/constraint-compiler/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/demo-tauri-libraries/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../forge-memory-bridge |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/demo-tauri-libraries/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../knowledge-runtime |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/demo-tauri-libraries/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../semantic-memory |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/demo-tauri-libraries/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../semantic-memory-forge |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/demo-tauri-libraries/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/discovery-portfolio/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/federated-settlement/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/job-queue/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/profile-runtime/Cargo.toml` | Cargo path dependency does not exist: ../assurance-runtime |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/profile-runtime/Cargo.toml` | Cargo path dependency does not exist: ../authority-delegation |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/profile-runtime/Cargo.toml` | Cargo path dependency does not exist: ../continuity-runtime |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/profile-runtime/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/profile-runtime/Cargo.toml` | Cargo path dependency does not exist: ../verification-policy |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/remote-oracle-admission/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/spec-execution/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Projects/StableMaster/s/Cargo.toml` | Cargo path dependency does not exist: ../../../Libraries/AI-Batch-Queue |
| warning | `cargo-path-dep-missing` | `Projects/StableMaster/s/Cargo.toml` | Cargo path dependency does not exist: ../../../Libraries/ComfyUI-RS |
| warning | `cargo-path-dep-missing` | `Projects/StableMaster/s/Cargo.toml` | Cargo path dependency does not exist: ../../../Libraries/LLM-Pipeline |
| warning | `cargo-path-dep-missing` | `Projects/StableMaster/s/Cargo.toml` | Cargo path dependency does not exist: ../../../Libraries/Ollama-Vision-RS |
| warning | `cargo-path-dep-missing` | `Projects/StableMaster/s/Cargo.toml` | Cargo path dependency does not exist: ../../../Libraries/Tauri-Queue |
| warning | `cargo-path-dep-missing` | `Projects/StableMaster/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../../Libraries/AI-Batch-Queue |
| warning | `cargo-path-dep-missing` | `Projects/StableMaster/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../../Libraries/ComfyUI-RS |
| warning | `cargo-path-dep-missing` | `Projects/StableMaster/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../../Libraries/LLM-Pipeline |
| warning | `cargo-path-dep-missing` | `Projects/StableMaster/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../../Libraries/Ollama-Vision-RS |
| warning | `cargo-path-dep-missing` | `Projects/StableMaster/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../../Libraries/Tauri-Queue |
| warning | `cargo-path-dep-missing` | `Utility Crates/AI-Batch-Queue/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Utility Crates/ComfyUI-RS/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Utility Crates/Ollama-Vision-RS/Cargo.toml` | Cargo path dependency does not exist: ../.parser-lib |
| warning | `cargo-path-dep-missing` | `Utility Crates/Tauri-Queue/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Utility Crates/agent-graph/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Utility Crates/job-queue/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `forge-workbench/Cargo.toml` | Cargo path dependency does not exist: ../Libraries/AI-Batch-Queue |
| warning | `cargo-path-dep-missing` | `forge-workbench/Cargo.toml` | Cargo path dependency does not exist: ../Libraries/LLM-Pipeline |
| warning | `cargo-path-dep-missing` | `forge-workbench/Cargo.toml` | Cargo path dependency does not exist: ../Libraries/Tauri-Queue |
| warning | `cargo-path-dep-missing` | `projmind/Cargo.toml` | Cargo path dependency does not exist: ../Libraries/LLM-Pipeline |
| warning | `rust-include-ref-not-archived` | `codex-openai/codex-rs/codex-client/src/custom_ca.rs` | include_str!/include_bytes! target exists but is not included in archive: codex-openai/codex-rs/codex-client/tests/fixtures/test-ca.pem |
| warning | `rust-include-ref-not-archived` | `codex-openai/codex-rs/codex-client/tests/ca_env.rs` | include_str!/include_bytes! target exists but is not included in archive: codex-openai/codex-rs/codex-client/tests/fixtures/test-ca-trusted.pem |
| warning | `rust-include-ref-not-archived` | `codex-openai/codex-rs/codex-client/tests/ca_env.rs` | include_str!/include_bytes! target exists but is not included in archive: codex-openai/codex-rs/codex-client/tests/fixtures/test-ca.pem |
| warning | `rust-include-ref-not-archived` | `codex-openai/codex-rs/codex-client/tests/ca_env.rs` | include_str!/include_bytes! target exists but is not included in archive: codex-openai/codex-rs/codex-client/tests/fixtures/test-intermediate.pem |
| warning | `rust-include-ref-not-archived` | `codex-openai/codex-rs/core/src/client_common.rs` | include_str!/include_bytes! target exists but is not included in archive: codex-openai/codex-rs/core/templates/review/exit_interrupted.xml |
| warning | `rust-include-ref-not-archived` | `codex-openai/codex-rs/core/src/client_common.rs` | include_str!/include_bytes! target exists but is not included in archive: codex-openai/codex-rs/core/templates/review/exit_success.xml |
| warning | `rust-include-ref-not-archived` | `codex-openai/codex-rs/execpolicy-legacy/src/lib.rs` | include_str!/include_bytes! target exists but is not included in archive: codex-openai/codex-rs/execpolicy-legacy/src/default.policy |
| warning | `rust-include-ref-not-archived` | `codex-openai/codex-rs/sandboxing/src/seatbelt.rs` | include_str!/include_bytes! target exists but is not included in archive: codex-openai/codex-rs/sandboxing/src/restricted_read_only_platform_defaults.sbpl |
| warning | `rust-include-ref-not-archived` | `codex-openai/codex-rs/sandboxing/src/seatbelt.rs` | include_str!/include_bytes! target exists but is not included in archive: codex-openai/codex-rs/sandboxing/src/seatbelt_base_policy.sbpl |
| warning | `rust-include-ref-not-archived` | `codex-openai/codex-rs/sandboxing/src/seatbelt.rs` | include_str!/include_bytes! target exists but is not included in archive: codex-openai/codex-rs/sandboxing/src/seatbelt_network_policy.sbpl |
| warning | `rust-include-ref-not-archived` | `codex-openai/codex-rs/tools/src/apply_patch_tool.rs` | include_str!/include_bytes! target exists but is not included in archive: codex-openai/codex-rs/tools/src/tool_apply_patch.lark |
| warning | `script-ref-missing` | `Libraries/scr-runtime/scripts/run_completion_checks.sh` | Possible script reference not found: .codex/tools/auto_phase_runner.py |
| warning | `script-ref-missing` | `agent-forge/deployment/scripts/deploy-local.sh` | Possible script reference not found: deployment/health-check.py |
| warning | `secret-like-filename` | `Agent/.env` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Cat Info App/.env` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Director/.env` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/gio/src/auto/credentials.rs` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/gio/src/auto/unix_credentials_message.rs` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/hyper-rustls/examples/sample.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/aia_test_cert.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/alt_name_cert.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/authority_key_identifier.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/cert.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/certs.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/certv3.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/cms.p12` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/corrupted-rsa.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/csr.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/dhparams.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/dsa.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/dsaparam.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/identity.p12` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/intermediate-ca.key` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/intermediate-ca.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/key.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/keystore-empty-chain.p12` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/leaf.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/nid_test_cert.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/nid_uid_test_cert.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/root-ca.key` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/root-ca.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/rsa-encrypted.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/openssl/test/rsa.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/ring/src/ec/suite_b/private_key.rs` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/ring/src/rsa/signature_rsa_example_private_key.der` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/ring/tests/ecdsa_test_private_key_p256.p8` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/ring/tests/ed25519_test_private_key.bin` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/ring/tests/ed25519_test_private_key.p8` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/ring/tests/rsa_test_private_key_2048.p8` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/schannel/test/cert.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/schannel/test/identity.p12` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/schannel/test/key.key` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/schannel/test/key.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/schannel/test/key_invalid_header.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/schannel/test/key_no_end_header.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/schannel/test/key_no_headers.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/schannel/test/key_wrong_header.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/tokio-native-tls/examples/identity.p12` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/tokio-native-tls/tests/identity.p12` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/tokio-rustls/tests/certs/chain.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/tokio-rustls/tests/certs/end.key` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/tokio-rustls/tests/certs/root.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/tower-http/src/cors/allow_credentials.rs` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/tower-http/src/follow_redirect/policy/filter_credentials.rs` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/untrusted/mk/llvm-snapshot.gpg.key` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/web-sys/src/features/gen_Credential.rs` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/webkit2gtk/src/auto/credential.rs` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/webkit2gtk/src/credential.rs` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/winapi-i686-pc-windows-gnu/lib/libwinapi_mincore-api-ms-win-security-credentials-l1-1-0.a` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/winapi-i686-pc-windows-gnu/lib/libwinapi_onecore-api-ms-win-security-credentials-l1-1-0.a` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/winapi-i686-pc-windows-gnu/lib/libwinapi_onecoreuap-api-ms-win-security-credentials-l1-1-0.a` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/winapi-x86_64-pc-windows-gnu/lib/libwinapi_mincore-api-ms-win-security-credentials-l1-1-0.a` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/winapi-x86_64-pc-windows-gnu/lib/libwinapi_onecore-api-ms-win-security-credentials-l1-1-0.a` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Gloss/src-tauri/vendor/crates/winapi-x86_64-pc-windows-gnu/lib/libwinapi_onecoreuap-api-ms-win-security-credentials-l1-1-0.a` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Kart/promptkart/backend/src/promptkart/util/secrets.py` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Libraries.BAK/docs/13_settings_persistence_and_secret_handling.md` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Libraries.BAK/libraries-source/docs/13_settings_persistence_and_secret_handling.md` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Libraries/_salvage_from_libraries2/Libraries2/docs/13_settings_persistence_and_secret_handling.md` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Medicine/backend/.env` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Phone/.buildozer/android/platform/python-for-android/.env` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Phone/webapp/.env.local` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `RecursiveOps/recursiveops/.secrets/jwt.secret` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Snap/.codex-runs/snap-prod-hardening/phase_reports/phase4_secret_scan.json` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Snap/.codex-runs/snap-prod-hardening/phase_reports/phase4_secret_scan_rerun.json` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Snap/.codex-runs/snap-prod-hardening/phase_reports/phase4_snap_secret_literal_check.json` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `Snap/tools/snap_validation/scan_no_secret_literals.py` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `agent-forge/deployment/k8s/secrets.yaml` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `codex-openai/.npmrc` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `codex-openai/codex-rs/codex-client/tests/fixtures/test-ca-trusted.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `codex-openai/codex-rs/codex-client/tests/fixtures/test-ca.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `codex-openai/codex-rs/codex-client/tests/fixtures/test-intermediate.pem` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `forge-workbench/crates/forge-workbench-core/src/services/secret_store.rs` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `forge-workbench/docs/13_settings_persistence_and_secret_handling.md` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `recursiveintell-web/.npmrc` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `stack-workbench/stack-workbench-codex-bundle/Libraries/docs/13_settings_persistence_and_secret_handling.md` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `website/.env` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `website/.npmrc` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `website/.settings.json` | File excluded because of secret-like-filename. |
| warning | `case-insensitive-path-collision` | `Libraries.BAK/libraries-source/master_issue_matrix.csv` | Path collides with Libraries.BAK/libraries-source/MASTER_ISSUE_MATRIX.csv on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/AI-Batch-Queue/.gitignore` | Path collides with Libraries.BAK/AI-Batch-Queue/.gitignore on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/AI-Batch-Queue/Cargo.lock` | Path collides with Libraries.BAK/AI-Batch-Queue/Cargo.lock on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/AI-Batch-Queue/Cargo.toml` | Path collides with Libraries.BAK/AI-Batch-Queue/Cargo.toml on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/AI-Batch-Queue/LICENSE` | Path collides with Libraries.BAK/AI-Batch-Queue/LICENSE on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/AI-Batch-Queue/README.md` | Path collides with Libraries.BAK/AI-Batch-Queue/README.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/AI-Batch-Queue/examples/basic_batch.rs` | Path collides with Libraries.BAK/AI-Batch-Queue/examples/basic_batch.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/AI-Batch-Queue/examples/eta_tracking.rs` | Path collides with Libraries.BAK/AI-Batch-Queue/examples/eta_tracking.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/AI-Batch-Queue/examples/model_optimization.rs` | Path collides with Libraries.BAK/AI-Batch-Queue/examples/model_optimization.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/AI-Batch-Queue/src/eta.rs` | Path collides with Libraries.BAK/AI-Batch-Queue/src/eta.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/AI-Batch-Queue/src/executor.rs` | Path collides with Libraries.BAK/AI-Batch-Queue/src/executor.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/AI-Batch-Queue/src/lib.rs` | Path collides with Libraries.BAK/AI-Batch-Queue/src/lib.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/AI-Batch-Queue/src/queue.rs` | Path collides with Libraries.BAK/AI-Batch-Queue/src/queue.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/AI-Batch-Queue/src/types.rs` | Path collides with Libraries.BAK/AI-Batch-Queue/src/types.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/AI-Batch-Queue/tests/integration_tests.rs` | Path collides with Libraries.BAK/AI-Batch-Queue/tests/integration_tests.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/ComfyUI-RS/.gitignore` | Path collides with Libraries.BAK/ComfyUI-RS/.gitignore on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/ComfyUI-RS/Cargo.lock` | Path collides with Libraries.BAK/ComfyUI-RS/Cargo.lock on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/ComfyUI-RS/Cargo.toml` | Path collides with Libraries.BAK/ComfyUI-RS/Cargo.toml on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/ComfyUI-RS/README.md` | Path collides with Libraries.BAK/ComfyUI-RS/README.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/ComfyUI-RS/examples/progress_tracking.rs` | Path collides with Libraries.BAK/ComfyUI-RS/examples/progress_tracking.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/ComfyUI-RS/examples/simple_generation.rs` | Path collides with Libraries.BAK/ComfyUI-RS/examples/simple_generation.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/ComfyUI-RS/examples/workflow_builder.rs` | Path collides with Libraries.BAK/ComfyUI-RS/examples/workflow_builder.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/ComfyUI-RS/src/client.rs` | Path collides with Libraries.BAK/ComfyUI-RS/src/client.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/ComfyUI-RS/src/error.rs` | Path collides with Libraries.BAK/ComfyUI-RS/src/error.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/ComfyUI-RS/src/lib.rs` | Path collides with Libraries.BAK/ComfyUI-RS/src/lib.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/ComfyUI-RS/src/types.rs` | Path collides with Libraries.BAK/ComfyUI-RS/src/types.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/ComfyUI-RS/src/workflow.rs` | Path collides with Libraries.BAK/ComfyUI-RS/src/workflow.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/.gitignore` | Path collides with Libraries.BAK/LLM-Pipeline/.gitignore on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/AGENTS.md` | Path collides with Libraries.BAK/LLM-Pipeline/AGENTS.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/ARCHITECTURE.md` | Path collides with Libraries.BAK/LLM-Pipeline/ARCHITECTURE.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/CLAUDE.md` | Path collides with Libraries.BAK/LLM-Pipeline/CLAUDE.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/Cargo.lock` | Path collides with Libraries.BAK/LLM-Pipeline/Cargo.lock on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/Cargo.toml` | Path collides with Libraries.BAK/LLM-Pipeline/Cargo.toml on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/LICENSE` | Path collides with Libraries.BAK/LLM-Pipeline/LICENSE on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/README.md` | Path collides with Libraries.BAK/LLM-Pipeline/README.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/examples/basic_pipeline.rs` | Path collides with Libraries.BAK/LLM-Pipeline/examples/basic_pipeline.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/examples/context_injection.rs` | Path collides with Libraries.BAK/LLM-Pipeline/examples/context_injection.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/examples/payload_chain.rs` | Path collides with Libraries.BAK/LLM-Pipeline/examples/payload_chain.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/examples/streaming_pipeline.rs` | Path collides with Libraries.BAK/LLM-Pipeline/examples/streaming_pipeline.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/examples/thinking_mode.rs` | Path collides with Libraries.BAK/LLM-Pipeline/examples/thinking_mode.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/src/chain.rs` | Path collides with Libraries.BAK/LLM-Pipeline/src/chain.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/src/client.rs` | Path collides with Libraries.BAK/LLM-Pipeline/src/client.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/src/error.rs` | Path collides with Libraries.BAK/LLM-Pipeline/src/error.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/src/events.rs` | Path collides with Libraries.BAK/LLM-Pipeline/src/events.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/src/exec_ctx.rs` | Path collides with Libraries.BAK/LLM-Pipeline/src/exec_ctx.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/src/lib.rs` | Path collides with Libraries.BAK/LLM-Pipeline/src/lib.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/src/llm_call.rs` | Path collides with Libraries.BAK/LLM-Pipeline/src/llm_call.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/src/parsing.rs` | Path collides with Libraries.BAK/LLM-Pipeline/src/parsing.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/src/payload.rs` | Path collides with Libraries.BAK/LLM-Pipeline/src/payload.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/src/pipeline.rs` | Path collides with Libraries.BAK/LLM-Pipeline/src/pipeline.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/src/prompt.rs` | Path collides with Libraries.BAK/LLM-Pipeline/src/prompt.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/src/stage.rs` | Path collides with Libraries.BAK/LLM-Pipeline/src/stage.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/src/streaming.rs` | Path collides with Libraries.BAK/LLM-Pipeline/src/streaming.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/src/types.rs` | Path collides with Libraries.BAK/LLM-Pipeline/src/types.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/LLM-Pipeline/tests/integration_tests.rs` | Path collides with Libraries.BAK/LLM-Pipeline/tests/integration_tests.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Ollama-Vision-RS/Cargo.lock` | Path collides with Libraries.BAK/Ollama-Vision-RS/Cargo.lock on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Ollama-Vision-RS/Cargo.toml` | Path collides with Libraries.BAK/Ollama-Vision-RS/Cargo.toml on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Ollama-Vision-RS/README.md` | Path collides with Libraries.BAK/Ollama-Vision-RS/README.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Ollama-Vision-RS/examples/caption_images.rs` | Path collides with Libraries.BAK/Ollama-Vision-RS/examples/caption_images.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Ollama-Vision-RS/examples/tag_images.rs` | Path collides with Libraries.BAK/Ollama-Vision-RS/examples/tag_images.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Ollama-Vision-RS/examples/thinking_mode.rs` | Path collides with Libraries.BAK/Ollama-Vision-RS/examples/thinking_mode.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Ollama-Vision-RS/src/captioner.rs` | Path collides with Libraries.BAK/Ollama-Vision-RS/src/captioner.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Ollama-Vision-RS/src/lib.rs` | Path collides with Libraries.BAK/Ollama-Vision-RS/src/lib.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Ollama-Vision-RS/src/parser.rs` | Path collides with Libraries.BAK/Ollama-Vision-RS/src/parser.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Ollama-Vision-RS/src/tagger.rs` | Path collides with Libraries.BAK/Ollama-Vision-RS/src/tagger.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Ollama-Vision-RS/src/types.rs` | Path collides with Libraries.BAK/Ollama-Vision-RS/src/types.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/README.md` | Path collides with Libraries.BAK/README.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-Queue/.gitignore` | Path collides with Libraries.BAK/Tauri-Queue/.gitignore on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-Queue/Cargo.lock` | Path collides with Libraries.BAK/Tauri-Queue/Cargo.lock on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-Queue/Cargo.toml` | Path collides with Libraries.BAK/Tauri-Queue/Cargo.toml on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-Queue/LICENSE` | Path collides with Libraries.BAK/Tauri-Queue/LICENSE on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-Queue/README.md` | Path collides with Libraries.BAK/Tauri-Queue/README.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-Queue/examples/basic_usage.rs` | Path collides with Libraries.BAK/Tauri-Queue/examples/basic_usage.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-Queue/examples/with_cancellation.rs` | Path collides with Libraries.BAK/Tauri-Queue/examples/with_cancellation.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-Queue/examples/with_cooldown.rs` | Path collides with Libraries.BAK/Tauri-Queue/examples/with_cooldown.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-Queue/examples/with_persistence.rs` | Path collides with Libraries.BAK/Tauri-Queue/examples/with_persistence.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-Queue/src/lib.rs` | Path collides with Libraries.BAK/Tauri-Queue/src/lib.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-Queue/tests/integration_tests.rs` | Path collides with Libraries.BAK/Tauri-Queue/tests/integration_tests.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-Queue/tests/test_helpers.rs` | Path collides with Libraries.BAK/Tauri-Queue/tests/test_helpers.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/.gitignore` | Path collides with Libraries.BAK/Tauri-React-Hooks/.gitignore on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/LICENSE` | Path collides with Libraries.BAK/Tauri-React-Hooks/LICENSE on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/README.md` | Path collides with Libraries.BAK/Tauri-React-Hooks/README.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/examples/demo-usage.md` | Path collides with Libraries.BAK/Tauri-React-Hooks/examples/demo-usage.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/package-lock.json` | Path collides with Libraries.BAK/Tauri-React-Hooks/package-lock.json on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/package.json` | Path collides with Libraries.BAK/Tauri-React-Hooks/package.json on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/src/index.ts` | Path collides with Libraries.BAK/Tauri-React-Hooks/src/index.ts on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/src/types.ts` | Path collides with Libraries.BAK/Tauri-React-Hooks/src/types.ts on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/src/useBufferedStream.ts` | Path collides with Libraries.BAK/Tauri-React-Hooks/src/useBufferedStream.ts on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/src/useTauriConfig.ts` | Path collides with Libraries.BAK/Tauri-React-Hooks/src/useTauriConfig.ts on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/src/useTauriEvent.ts` | Path collides with Libraries.BAK/Tauri-React-Hooks/src/useTauriEvent.ts on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/src/useTauriEvents.ts` | Path collides with Libraries.BAK/Tauri-React-Hooks/src/useTauriEvents.ts on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/src/useTauriMutation.ts` | Path collides with Libraries.BAK/Tauri-React-Hooks/src/useTauriMutation.ts on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/src/useTauriQuery.ts` | Path collides with Libraries.BAK/Tauri-React-Hooks/src/useTauriQuery.ts on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/tsconfig.json` | Path collides with Libraries.BAK/Tauri-React-Hooks/tsconfig.json on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/Tauri-React-Hooks/tsup.config.ts` | Path collides with Libraries.BAK/Tauri-React-Hooks/tsup.config.ts on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/AGENTS.md` | Path collides with Libraries.BAK/agent-graph/AGENTS.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/ARCHITECTURE.md` | Path collides with Libraries.BAK/agent-graph/ARCHITECTURE.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/CLAUDE.md` | Path collides with Libraries.BAK/agent-graph/CLAUDE.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/Cargo.lock` | Path collides with Libraries.BAK/agent-graph/Cargo.lock on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/Cargo.toml` | Path collides with Libraries.BAK/agent-graph/Cargo.toml on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/LICENSE` | Path collides with Libraries.BAK/agent-graph/LICENSE on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/README.md` | Path collides with Libraries.BAK/agent-graph/README.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/benches/graph_bench.rs` | Path collides with Libraries.BAK/agent-graph/benches/graph_bench.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/examples/basic.rs` | Path collides with Libraries.BAK/agent-graph/examples/basic.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/examples/checkpointing.rs` | Path collides with Libraries.BAK/agent-graph/examples/checkpointing.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/examples/conditional.rs` | Path collides with Libraries.BAK/agent-graph/examples/conditional.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/examples/human_in_loop.rs` | Path collides with Libraries.BAK/agent-graph/examples/human_in_loop.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/examples/loop_example.rs` | Path collides with Libraries.BAK/agent-graph/examples/loop_example.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/examples/map_reduce.rs` | Path collides with Libraries.BAK/agent-graph/examples/map_reduce.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/examples/parallel.rs` | Path collides with Libraries.BAK/agent-graph/examples/parallel.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/examples/pipeline_node.rs` | Path collides with Libraries.BAK/agent-graph/examples/pipeline_node.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/examples/reducers.rs` | Path collides with Libraries.BAK/agent-graph/examples/reducers.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/examples/research_agent.rs` | Path collides with Libraries.BAK/agent-graph/examples/research_agent.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/examples/retry.rs` | Path collides with Libraries.BAK/agent-graph/examples/retry.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/examples/streaming.rs` | Path collides with Libraries.BAK/agent-graph/examples/streaming.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/examples/subgraph.rs` | Path collides with Libraries.BAK/agent-graph/examples/subgraph.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/examples/visualization.rs` | Path collides with Libraries.BAK/agent-graph/examples/visualization.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/checkpoint.rs` | Path collides with Libraries.BAK/agent-graph/src/checkpoint.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/checkpoint_store.rs` | Path collides with Libraries.BAK/agent-graph/src/checkpoint_store.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/checkpointer.rs` | Path collides with Libraries.BAK/agent-graph/src/checkpointer.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/command.rs` | Path collides with Libraries.BAK/agent-graph/src/command.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/config.rs` | Path collides with Libraries.BAK/agent-graph/src/config.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/edge.rs` | Path collides with Libraries.BAK/agent-graph/src/edge.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/error.rs` | Path collides with Libraries.BAK/agent-graph/src/error.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/event_sink.rs` | Path collides with Libraries.BAK/agent-graph/src/event_sink.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/executor.rs` | Path collides with Libraries.BAK/agent-graph/src/executor.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/graph.rs` | Path collides with Libraries.BAK/agent-graph/src/graph.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/interrupt.rs` | Path collides with Libraries.BAK/agent-graph/src/interrupt.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/join.rs` | Path collides with Libraries.BAK/agent-graph/src/join.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/lib.rs` | Path collides with Libraries.BAK/agent-graph/src/lib.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/node.rs` | Path collides with Libraries.BAK/agent-graph/src/node.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/outcome.rs` | Path collides with Libraries.BAK/agent-graph/src/outcome.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/payload.rs` | Path collides with Libraries.BAK/agent-graph/src/payload.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/prelude.rs` | Path collides with Libraries.BAK/agent-graph/src/prelude.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/reducer.rs` | Path collides with Libraries.BAK/agent-graph/src/reducer.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/retry.rs` | Path collides with Libraries.BAK/agent-graph/src/retry.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/router.rs` | Path collides with Libraries.BAK/agent-graph/src/router.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/state.rs` | Path collides with Libraries.BAK/agent-graph/src/state.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/src/stream.rs` | Path collides with Libraries.BAK/agent-graph/src/stream.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/tests/checkpointer_tests.rs` | Path collides with Libraries.BAK/agent-graph/tests/checkpointer_tests.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/tests/execution_tests.rs` | Path collides with Libraries.BAK/agent-graph/tests/execution_tests.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/tests/integration_tests.rs` | Path collides with Libraries.BAK/agent-graph/tests/integration_tests.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/tests/interrupt_tests.rs` | Path collides with Libraries.BAK/agent-graph/tests/interrupt_tests.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/tests/parallel_tests.rs` | Path collides with Libraries.BAK/agent-graph/tests/parallel_tests.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/tests/reducer_tests.rs` | Path collides with Libraries.BAK/agent-graph/tests/reducer_tests.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/tests/retry_tests.rs` | Path collides with Libraries.BAK/agent-graph/tests/retry_tests.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/tests/routing_tests.rs` | Path collides with Libraries.BAK/agent-graph/tests/routing_tests.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/tests/runtime_tests.rs` | Path collides with Libraries.BAK/agent-graph/tests/runtime_tests.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/tests/state_tests.rs` | Path collides with Libraries.BAK/agent-graph/tests/state_tests.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/tests/streaming_tests.rs` | Path collides with Libraries.BAK/agent-graph/tests/streaming_tests.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Libraries.bak/agent-graph/tests/subgraph_tests.rs` | Path collides with Libraries.BAK/agent-graph/tests/subgraph_tests.rs on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `Recall-Coding/agents.md` | Path collides with Recall-Coding/AGENTS.md on case-insensitive filesystems. |
| warning | `case-insensitive-path-collision` | `forge-workbench/agents.md` | Path collides with forge-workbench/AGENTS.md on case-insensitive filesystems. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/PIL/_avif.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/PIL/_imaging.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/PIL/_imagingcms.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/PIL/_imagingft.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/PIL/_imagingmath.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/PIL/_imagingmorph.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/PIL/_imagingtk.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/PIL/_webp.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/PIL/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/_pytest/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/alembic/context.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/alembic/op.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/alembic/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/annotated_types/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/anyio/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/argon2/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/black/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/certifi/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/charset_normalizer/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/click/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/dns/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/dotenv/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/email_validator/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/fastapi/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/fontTools/misc/plistlib/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/h11/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/httpcore/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/httptools/parser/parser.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/httptools/parser/url_parser.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/httpx/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/idna/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/iniconfig/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/jinja2/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/limits/_version.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/limits/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/markupsafe/_speedups.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/markupsafe/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/packaging/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pathspec/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/cachecontrol/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/certifi/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/distlib.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/distro/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/idna/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/msgpack.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/packaging/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/pkg_resources.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/platformdirs/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/pygments.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/pyproject_hooks.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/requests.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/resolvelib/__init__.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/resolvelib/compat/collections_abc.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/resolvelib/providers.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/resolvelib/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/resolvelib/reporters.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/resolvelib/resolvers.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/resolvelib/structs.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/rich/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/tomli/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/truststore/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/typing_extensions.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/_vendor/urllib3.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pip/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pkg_resources/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/platformdirs/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pluggy/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pydantic/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pydantic/v1/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pydantic_core/_pydantic_core.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pydantic_core/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pydantic_settings/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pyotp/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pytest/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/pytest_asyncio/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/rsa/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/setuptools/_vendor/importlib_metadata/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/setuptools/_vendor/inflect/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/setuptools/_vendor/jaraco/collections/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/setuptools/_vendor/jaraco/functools/__init__.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/setuptools/_vendor/jaraco/functools/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/setuptools/_vendor/more_itertools/__init__.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/setuptools/_vendor/more_itertools/more.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/setuptools/_vendor/more_itertools/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/setuptools/_vendor/more_itertools/recipes.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/setuptools/_vendor/packaging/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/setuptools/_vendor/platformdirs/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/setuptools/_vendor/tomli/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/setuptools/_vendor/typeguard/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/slowapi/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/sniffio/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/sqlalchemy/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/starlette/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/urllib3/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/uvicorn/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/uvloop/loop.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/uvloop/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/watchfiles/_rust_notify.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/watchfiles/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/websockets/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/websockets/speedups.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/wrapt/__init__.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Medicine/backend/.venv311/lib/python3.11/site-packages/wrapt/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/Qt3DAnimation.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/Qt3DCore.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/Qt3DExtras.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/Qt3DInput.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/Qt3DLogic.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/Qt3DRender.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtBluetooth.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtCharts.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtConcurrent.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtCore.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtDBus.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtDataVisualization.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtDesigner.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtGraphs.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtGraphsWidgets.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtGui.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtHelp.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtHttpServer.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtLocation.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtMultimedia.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtMultimediaWidgets.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtNetwork.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtNetworkAuth.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtNfc.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtOpenGL.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtOpenGLWidgets.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtPdf.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtPdfWidgets.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtPositioning.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtPrintSupport.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtQml.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtQuick.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtQuick3D.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtQuickControls2.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtQuickTest.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtQuickWidgets.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtRemoteObjects.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtScxml.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtSensors.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtSerialBus.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtSerialPort.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtSpatialAudio.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtSql.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtStateMachine.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtSvg.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtSvgWidgets.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtTest.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtTextToSpeech.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtUiTools.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtWebChannel.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtWebEngineCore.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtWebEngineQuick.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtWebEngineWidgets.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtWebSockets.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtWebView.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtWidgets.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/QtXml.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/__feature__.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/PySide6/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/filelock/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/jinja2/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/markupsafe/_speedups.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/markupsafe/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/ninja/_version.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/ninja/ninja_syntax.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/ninja/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/packaging/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pip/_vendor/cachecontrol/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pip/_vendor/certifi/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pip/_vendor/dependency_groups/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pip/_vendor/distro/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pip/_vendor/idna/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pip/_vendor/packaging/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pip/_vendor/platformdirs/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pip/_vendor/pyproject_hooks/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pip/_vendor/resolvelib/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pip/_vendor/rich/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pip/_vendor/tomli/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pip/_vendor/tomli_w/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pip/_vendor/truststore/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pip/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pkg_resources/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pkginfo/__init__.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pkginfo/bdist.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pkginfo/commandline.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pkginfo/develop.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pkginfo/distribution.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pkginfo/index.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pkginfo/installed.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pkginfo/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pkginfo/sdist.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pkginfo/utils.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pkginfo/wheel.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/platformdirs/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/pyproject_hooks/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/setuptools/_vendor/importlib_metadata/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/setuptools/_vendor/inflect/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/setuptools/_vendor/jaraco/collections/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/setuptools/_vendor/jaraco/functools/__init__.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/setuptools/_vendor/jaraco/functools/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/setuptools/_vendor/more_itertools/__init__.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/setuptools/_vendor/more_itertools/more.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/setuptools/_vendor/more_itertools/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/setuptools/_vendor/more_itertools/recipes.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/setuptools/_vendor/packaging/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/setuptools/_vendor/platformdirs/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/setuptools/_vendor/tomli/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/setuptools/_vendor/typeguard/py.typed` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/shiboken6/Shiboken.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `Phone/.venv_android/lib/python3.11/site-packages/shiboken6/py.typed` | python adapter expected this existing file to be included. |

## Included files by extension

| Extension | Count |
|---|---:|
| `.rs` | 49063 |
| `.json` | 16734 |
| `.md` | 9970 |
| `.html` | 2867 |
| `.toml` | 2795 |
| `<no-extension>` | 2579 |
| `.py` | 2168 |
| `.lock` | 1436 |
| `.txt` | 1071 |
| `.yml` | 846 |
| `.ts` | 845 |
| `.sh` | 528 |
| `.tsx` | 420 |
| `.js` | 147 |
| `.patch` | 132 |
| `.csv` | 121 |
| `.0` | 114 |
| `.yaml` | 100 |
| `.css` | 80 |
| `.jsonl` | 77 |
| `.sql` | 59 |
| `.tpl` | 24 |
| `.spdx` | 20 |
| `.0_with_llvm-exception` | 16 |
| `.rst` | 13 |
| `.cfg` | 11 |
| `.conf` | 11 |
| `.mjs` | 11 |
| `.example` | 10 |
| `.service` | 9 |
| `.ini` | 8 |
| `.ndjson` | 8 |
| `.ps1` | 8 |
| `.typed` | 8 |
| `.jsx` | 7 |
| `.log` | 7 |
| `.tsv` | 5 |
| `.cjs` | 4 |
| `.macosx` | 4 |
| `.mkhybrid` | 4 |
| `.template` | 4 |
| `.audio` | 3 |
| `.cdrw` | 3 |
| `.cdtext` | 3 |
| `.clone` | 3 |
| `.compression` | 3 |
| `.copy` | 3 |
| `.diff` | 3 |
| `.diskt@2` | 3 |
| `.eltorito` | 3 |
| `.graft_dirs` | 3 |
| `.hfs_boot` | 3 |
| `.hfs_magic` | 3 |
| `.hide` | 3 |
| `.htm` | 3 |
| `.interface` | 3 |
| `.joliet` | 3 |
| `.multi` | 3 |
| `.parallel` | 3 |
| `.paranoia` | 3 |
| `.prep_boot` | 3 |
| `.pyi` | 3 |
| `.raw` | 3 |
| `.rootinfo` | 3 |
| `.rscsi` | 3 |
| `.session` | 3 |
| `.solaris-x86-ata-dma` | 3 |
| `.solaris-x86-atapi-dma` | 3 |
| `.sony` | 3 |
| `.sort` | 3 |
| `.sparcboot` | 3 |
| `.sun-lofi` | 3 |
| `.sunx86boot` | 3 |
| `.tmpl` | 3 |
| `.verify` | 3 |
| `.worm` | 3 |
| `.gpl` | 2 |
| `.lgpl` | 2 |
| `.mit` | 2 |
| `.mk` | 2 |
| `.mkd` | 2 |
| `.osx` | 2 |
| `.1` | 1 |
| `.2-3pulldown` | 1 |
| `.aix` | 1 |
| `.altivec` | 1 |
| `.apple` | 1 |
| `.avilib` | 1 |
| `.bsdi` | 1 |
| `.build` | 1 |
| `.dv` | 1 |
| `.fdl` | 1 |
| `.fonts` | 1 |
| `.freebsd` | 1 |
| `.gmake` | 1 |
| `.gplv2` | 1 |
| `.gplv3` | 1 |
| `.hpux` | 1 |
| `.install` | 1 |
| `.lavpipe` | 1 |
| `.lgplv3` | 1 |
| `.linux` | 1 |
| `.linux-shm` | 1 |
| `.macosx-old-versions` | 1 |
| `.mingw32` | 1 |
| `.msdos` | 1 |
| `.next` | 1 |
| `.openbsd` | 1 |
| `.os2` | 1 |
| `.ppc` | 1 |
| `.qnx` | 1 |
| `.sgi` | 1 |
| `.solaris` | 1 |
| `.sspm` | 1 |
| `.subtitles` | 1 |
| `.sunos` | 1 |
| `.texi` | 1 |
| `.transist` | 1 |
| `.win32` | 1 |
| `.xiph` | 1 |

## Included files by top-level path

| Top-level path | Count |
|---|---:|
| `gloss-replay-p3-final` | 31684 |
| `Gloss` | 28837 |
| `Libraries` | 4668 |
| `Libraries.BAK` | 3505 |
| `Hootie` | 3396 |
| `shannon` | 2940 |
| `codex-openai` | 2904 |
| `gloss-target-p3` | 2475 |
| `ClaimLedger` | 2459 |
| `stack-workbench` | 1402 |
| `Recall` | 1393 |
| `Recall-Coding` | 1323 |
| `Phone` | 816 |
| `agent-forge` | 562 |
| `rust-ai-quality-benchmark` | 500 |
| `aicc` | 287 |
| `visionforge` | 234 |
| `Utility Crates` | 212 |
| `Director` | 165 |
| `Rivot` | 163 |
| `Libraries.bak` | 157 |
| `RecursiveOps` | 147 |
| `Phase-Automation` | 146 |
| `Medicine` | 141 |
| `Palisade` | 133 |
| `forge-workbench` | 129 |
| `Agent` | 111 |
| `recursiveintell-web` | 111 |
| `website` | 111 |
| `Sortarr` | 110 |
| `Playground` | 109 |
| `Cat Info App` | 107 |
| `Snap` | 105 |
| `Projects` | 99 |
| `Kart` | 84 |
| `Coding` | 73 |
| `projmind` | 72 |
| `torrent-fetch` | 58 |
| `library-matrices` | 46 |
| `Forge-Audit` | 45 |
| `Chronicle` | 44 |
| `SocialBacklog` | 42 |
| `subgen` | 42 |
| `mine` | 38 |
| `forge-v2` | 36 |
| `Portal Doctor` | 32 |
| `Workbench` | 24 |
| `TestApp` | 18 |
| `30_CODEX_BUNDLE` | 17 |
| `Pictures` | 15 |
| `.review_cache` | 14 |
| `Research` | 14 |
| `dvddvd` | 14 |
| `20_CODEX_BUNDLE` | 12 |
| `40_VALIDATION` | 9 |
| `RokoTools` | 8 |
| `20_LEDGER` | 7 |
| `Game` | 7 |
| `10_EVIDENCE` | 4 |
| `60_AUDITOR_HANDOFF` | 4 |
| `scripts` | 4 |
| `10_LEDGER` | 3 |
| `50_RECEIPTS` | 3 |
| `docs` | 3 |
| `libs` | 3 |
| `recall-windows` | 3 |
| `00_OPERATOR` | 2 |
| `30_VALIDATION` | 2 |
| `40_AUDITOR_HANDOFF` | 2 |
| `recall-linux` | 2 |
| `.stfolder` | 1 |
| `00_README.md` | 1 |
| `01_OPERATOR_DECISION_BRIEF.md` | 1 |
| `02_SCOPE_AND_ASSUMPTIONS.md` | 1 |
| `03_REQUIRED_INPUTS.md` | 1 |
| `04_FORBIDDEN_CHANGES.md` | 1 |
| `05_RUN_ORDER.md` | 1 |
| `ACCEPTANCE_GATES.md` | 1 |
| `AGENT-SYSTEM.md` | 1 |
| `AGENTS-TEMPLATE.md` | 1 |
| `AGENTS.md` | 1 |
| `Agent.md` | 1 |
| `Cat Info App.md` | 1 |
| `Coding-research-next-codex-context-20260524T021546Z.codex-archive.json` | 1 |
| `Coding.md` | 1 |
| `Director.md` | 1 |
| `FINAL_REPORT_TEMPLATE.md` | 1 |
| `GENERATED_FILE_TREE.txt` | 1 |
| `MANUAL_PHASE_INJECTIONS.md` | 1 |
| `MASTER_CODEBASE_REFERENCE2.md` | 1 |
| `MASTER_CODEX_PROMPT.md` | 1 |
| `Medicine.md` | 1 |
| `PACK_METADATA.json` | 1 |
| `PHASE_00_PREFLIGHT.md` | 1 |
| `PHASE_01_LIBRARIES_CANONICAL_CLOSURE.md` | 1 |
| `PHASE_02_SALVAGE_TERMINAL_DECISIONS.md` | 1 |
| `PHASE_03_RESIDUAL_LIBRARIES2_REFS.md` | 1 |
| `PHASE_04_DOWNSTREAM_DEPENDENCY_REPAIR.md` | 1 |
| `PHASE_05_SEMANTIC_MEMORY_AND_GLOSS_BOUNDARY.md` | 1 |
| `PHASE_06_CLAIMLEDGER_FORGE_BOUNDARY.md` | 1 |
| `PHASE_07_GENERATED_ARTIFACT_HYGIENE.md` | 1 |
| `PHASE_08_VALIDATION_AND_RECEIPTS.md` | 1 |
| `PHASE_09_FINAL_AUDITOR_HANDOFF.md` | 1 |
| `PLANa.md` | 1 |
| `Phone.md` | 1 |
| `Pictures.md` | 1 |
| `Playground.md` | 1 |
| `Portal Doctor.md` | 1 |
| `ROLLBACK_PLAN.md` | 1 |
| `RecursiveOps.md` | 1 |
| `Research.md` | 1 |
| `STATEa.md` | 1 |
| `TRANSFER.md` | 1 |
| `VALIDATION_COMMANDS.md` | 1 |
| `backup.py` | 1 |
| `codex.md` | 1 |
| `gitdb.md` | 1 |
| `recall-codex.md` | 1 |
| `research-architectural-next-steps-2026-04-15.md` | 1 |
| `website.md` | 1 |
| `z.py` | 1 |
| `zip.py` | 1 |

## Exclusion reasons

| Reason | Count |
|---|---:|
| `unsupported-extension-or-basename` | 42732 |
| `binary-build-artifact` | 27269 |
| `max-file-size-exceeded` | 2806 |
| `image-disabled` | 2207 |
| `log-disabled` | 1403 |
| `archive-file` | 197 |
| `generated-sidecar` | 194 |
| `secret-like-filename` | 85 |
| `database-file` | 78 |
| `non-utf8-text-file` | 46 |
| `doc-binary-disabled` | 31 |
| `symlink-disabled` | 5 |
| `binary-null-byte` | 4 |

## Sidecar files

- Manifest: `/home/sikmindz/Coding/Libraries/docs/post-salvage-validation/sidecars/Coding-post-salvage-20260525.manifest.json`
- Markdown report: `/home/sikmindz/Coding/Libraries/docs/post-salvage-validation/sidecars/Coding-post-salvage-20260525.report.md`
- Excluded file list: `/home/sikmindz/Coding/Libraries/docs/post-salvage-validation/sidecars/Coding-post-salvage-20260525.excluded.json`
- Findings: `/home/sikmindz/Coding/Libraries/docs/post-salvage-validation/sidecars/Coding-post-salvage-20260525.findings.json`

## Interpretation

This package has warnings. It is probably usable, but the warnings should be reviewed before using it as a Codex or audit handoff.
