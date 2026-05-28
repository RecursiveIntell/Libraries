# P30 Command Bar

Run from archive root `/home/sikmindz/Coding/Libraries` unless the pass creates a proven standalone mode.

```bash
cd /home/sikmindz/Coding/Libraries
python3 AiDENs/scripts/p30_guard.py --repo AiDENs
bash AiDENs/scripts/p30_verify.sh
cargo metadata --manifest-path AiDENs/Cargo.toml --locked --format-version 1
cargo fmt --manifest-path AiDENs/Cargo.toml --all -- --check
cargo check --manifest-path AiDENs/Cargo.toml --workspace --all-targets --locked
cargo test --manifest-path AiDENs/Cargo.toml --workspace --all-targets --locked
cargo clippy --manifest-path AiDENs/Cargo.toml --workspace --all-targets --locked -- -D warnings
```

If `--manifest-path` with workspace causes path issues, run from the actual workspace root that owns the sibling crates and record the reason.

Optional static sweeps:

```bash
rg -n "unwrap_or_default\(|filter_map\(|permissive_degraded_repair|Uuid::new_v4|ArtifactId::new\("tool-exposure"\)|VerificationAttemptState::Succeeded|CheckMethod::AdvisoryOnly|let _ = write_file_atomically|std::fs::read_to_string\(&path\)" AiDENs/crates
rg -n "panic!\(|todo!\(|unimplemented!\(|\.unwrap\(|\.expect\(" AiDENs/crates
rg -n "serde_json::Value|json!\(|as_object\(|as_array\(" AiDENs/crates
```
