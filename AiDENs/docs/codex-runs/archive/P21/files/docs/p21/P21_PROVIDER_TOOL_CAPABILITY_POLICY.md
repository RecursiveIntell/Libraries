# P21 Provider and Tool Capability Policy

## Provider statuses

| Provider | P21 default status | Allowed claim |
|---|---|---|
| mock | supported | executable fixture provider |
| ollama | partial | chat-only if configured and endpoint reachable |
| openai-compatible | stretch only | chat-only if implemented/tested |
| openai | deferred unless implemented | unavailable/deferred |
| anthropic | deferred unless implemented | unavailable/deferred |
| openrouter | deferred unless implemented | unavailable/deferred |

## Hard rule

No provider may claim native tool-loop support without executable integration or fixture tests proving the loop.

## Tool policy

- read/list/search/stat tools may be enabled by `coding-agent` default;
- patch-propose may be enabled as proposal-only;
- patch-apply requires explicit scoped permit;
- run-checks requires explicit permit unless configured in a safe fixture;
- shell/network/admin tools are disabled by default.

## Required reports

`provider-check` must expose:

- configured;
- executable;
- native_tool_loop;
- structured_output;
- degraded;
- reason_codes.

`tools inspect` must expose:

- declared;
- registered;
- executable;
- exposed_this_turn;
- blocked_this_turn;
- requires_permit;
- provider_schema_tool_ids.
