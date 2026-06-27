# Host adapter integration notes

This crate is host-agnostic. OpenCode, Codex, Claude Code, Hermes, and other agent hosts should integrate through the same narrow adapter contract instead of adding host-specific dependencies to the core crate.

## Adapter contract

1. Convert host transcript items into `CompactRequest.messages`.
2. Preserve the latest user task as a normal `user` message.
3. Put host-only metadata in `Message.metadata`; the core crate treats it as receipt context, not execution authority.
4. Call one of:
   - `context_governor::compact_context(request)` from Rust
   - `context-governor compact < request.json` from any host
5. Persist the returned `CompactResponse` under the host/profile data directory.
6. Expose receipt tools around:
   - `context-governor search --dir DIR --query TEXT`
   - `context-governor expand --dir DIR --receipt RECEIPT --item ITEM`
   - `context-governor diff < response.json`
7. If semantic-memory archival is enabled, the host adapter performs the write and records the returned external IDs in the receipt. The core crate does not perform network or MCP writes.

## Suggested metadata keys

These are conventions only; unknown keys are preserved but not trusted.

- `provider`: model provider name, for host-side token accounting/audit.
- `model`: concrete model name, for host-side token accounting/audit.
- `tool_name`: host tool name for tool outputs.
- `command_exit_code`: integer exit code for shell/tool results.
- `pinned`: boolean; host requests verbatim preservation.
- `must_preserve`: boolean; host requests exact fallback at minimum.
- `sensitive`: boolean; host marks content unsuitable for semantic archival.
- `source_path`: source file/path for retrieved context.
- `receipt_ref`: upstream receipt or task id.

## Host-specific status

- Hermes: implemented as a local ContextEngine plugin under the Hermes profile, with receipt tools and host-side semantic-memory writes.
- Codex/OpenCode/Claude Code: use the CLI contract above today. Do not claim native host plugins until each host has a tested adapter package.

## Safe rollout policy

- Keep the core crate deterministic and dependency-light.
- Keep provider-native tokenizer support host-side or feature-gated until exact model mappings are verified.
- Keep semantic-memory writes host-side so receipts contain real external IDs instead of placeholder claims.
- Use `soft_warn` for first live trials; use `hard_cascade` only after replay receipts show acceptable recoverability and fail-open behavior.
