# Phase 03 — Generated Agent Project Proof

Make `aidens new coding-agent target/demo-agent` create a runnable safe project.

Then prove:

```bash
cargo run -p aidens-cli -- run --config target/demo-agent/aidens.toml "read README"
```

Generated project must contain useful operator docs and safe defaults.
