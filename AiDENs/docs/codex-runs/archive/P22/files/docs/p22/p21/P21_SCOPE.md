# P21 Scope — Usable Agent Builder Proof + Cross-App Extraction Superpass

## Mission

Turn AiDENs from a v0.1-candidate orchestration foundation into a usable agent-builder layer with executable proof.

## Finish target

After P21, an operator should be able to run:

```bash
cargo run -p aidens-cli -- run-test-agent fixtures/test-agent/basic-agent.toml
cargo run -p aidens-cli -- new coding-agent target/demo-agent
cargo run -p aidens-cli -- run --config target/demo-agent/aidens.toml "read README"
cargo run -p aidens-cli -- doctor --config target/demo-agent/aidens.toml
cargo run -p aidens-cli -- provider-check --config target/demo-agent/aidens.toml
cargo run -p aidens-cli -- tools inspect --config target/demo-agent/aidens.toml
bash scripts/p21_verify_release_archive.sh target/p21/aidens-v0.1-candidate.zip
```

## Required non-goals

P21 must not turn into a generalized feature grab-bag. These are deferred unless all mandatory phases pass:

- multi-agent fanout;
- full desktop daemon UX;
- federated settlement;
- regional fixpoint runtime;
- mechanism/theory search;
- provider-native tool loops beyond controlled stretch gates.

## Stretch lane

If and only if all mandatory gates pass, Codex may execute stretch phases:

1. minimal OpenAI-compatible chat provider if bounded and tested;
2. first coding-agent useful workflow;
3. daemon/queue schedule smoke path;
4. Recall/Recall-Coding extraction docs/templates.
