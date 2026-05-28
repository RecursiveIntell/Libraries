# P21 Provider Expansion Plan

This is a planning artifact only. It does not implement, advertise, or certify
cloud provider execution or native tool loops.

## Current Boundary

- `mock` remains fixture-supported, local, and not cloud support.
- `ollama` remains partial local chat only; native tool loops remain false.
- `openai`, `openrouter`, `anthropic`, `openai-compatible`, and `compatible`
  remain unavailable until each provider has executable tests and receipt
  evidence.
- Parser fallback is degraded execution evidence, not native tool calling.

## Entry Gates

P21 may start only after the P20.2 release archive replay passes from a clean
unpack. Before any provider is promoted, these gates must pass:

- provider backend matrix test for the exact provider kind;
- route receipt test proving configured, executable, degraded, and blocked
  states;
- no native tool-loop flag unless the loop is executable and tested;
- tool exposure and permit receipts for every tool path;
- no fallback path counted as provider support;
- archive replay with package integrity, testkit purity, static scan, and
  scanner self-test.

## Sequence

1. Add provider-specific fixtures that describe expected unavailable behavior.
2. Add route/readiness tests that keep the provider unavailable.
3. Implement the smallest chat boundary only after tests prove unavailable
   behavior first.
4. Add receipt tests for provider route, retry/degraded states, and final
   run report linkage.
5. Keep native tool calling false until a provider-native loop has its own
   executable vertical slice.
6. Update examples only after the executable tests pass.

## Non-Goals

- No broad "OpenAI-compatible" umbrella support without per-route tests.
- No cloud provider support labels based only on environment variables or API
  keys.
- No native tool-loop claims through parser fallback.
- No provider fallback that silently changes provider identity.

## Required Proof

P21 provider expansion is not certifiable until these commands pass in the real
sibling-crate workspace:

```bash
cargo test -p aidens-provider-kit --all-targets
cargo test -p aidens-runner --all-targets
cargo test -p aidens-tool-kit --all-targets
cargo test -p aidens-integration-tests --all-targets
P20_2_REQUIRE_CARGO=1 bash scripts/p20_2_verify.sh
```

The release archive must also pass:

```bash
bash scripts/p20_2_verify_release_zip.sh target/p20-2/phase10/aidens-p20-2-release.zip
```
