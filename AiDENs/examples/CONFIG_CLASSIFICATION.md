# Example Config Classification

Record date: `2026-05-07`

These examples are fixtures, not support claims. A config can only become runtime-visible after validation and a config-apply receipt.

| Config | Provider class | Tool authority | Support meaning |
|---|---|---|---|
| `examples/aidens.toml` | local Ollama chat endpoint | no write/admin grants declared | local endpoint example; not native tool-loop support |
| `examples/aidens.mock.toml` | explicit mock | safe-coding bundle, side effects still require approval | fixture-backed local coding-agent example |
| `examples/aidens.disabled.toml` | disabled | safe-coding bundle, provider unavailable | unavailable-provider example |
| `examples/aidens.ollama.toml` | local Ollama chat endpoint | safe-coding bundle, side effects still require approval | local chat route only |
| `examples/aidens.chat-only.toml` | explicit mock | no tool bundles | chat-only fixture |
| `examples/aidens.memory.toml` | explicit mock | no tool bundles | memory fixture; canonical memory owner remains sibling crates |
| `examples/aidens.daemon.toml` | explicit mock | no broad daemon authority | scaffold/profile fixture, not broad autonomy |
| `examples/aidens.research.toml` | explicit mock | no live research authority | research fixture, not external citation proof |
| `examples/aidens.openai-unavailable.toml` | cloud provider without key | no tool bundles | unavailable-cloud route example |
| `examples/configs/*.toml` | fixture/mock unless explicitly unavailable | no write/admin default without approval | template fixtures |
| `examples/flagship-local-coding-agent/aidens.toml` | explicit mock | patch-apply listed but still permit-gated | local fixture, not autonomous write authority |

## Rules

- Secret-like fields outside `provider.api_key` are rejected before TOML deserialization can ignore them.
- `provider.api_key` is rejected for `mock`, `disabled`, `local`, and `ollama` providers.
- Embedded endpoint credentials are rejected.
- Supported-local configs require `security.write_policy = "approval-required"` and `security.network_policy` of `disabled` or `local-only`.
- Config source paths are canonicalized and fingerprinted when loaded.
