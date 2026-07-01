# Compressed Working-Memory Glossary

- Shared pool: immutable compressed KV/context pages reused across agents or sessions.
- Agent shell: per-agent/private compressed overlay for recent or unique context.
- Compressed page: transfer/scoring unit containing compressed keys or values plus role metadata.
- Candidate: approximate retrieval/attention item selected by compressed-domain scoring.
- Exact fallback: explicit path that decodes or reloads authoritative data and emits a receipt.
- PPL replay: model-quality validation comparing oracle forward-pass perplexity with replayed/compressed cache behavior.
- Score uncertainty: bound or heuristic describing whether compressed approximate top-k is safe without refinement.
- Guard token: token included regardless of compressed score, usually recent/sink/system tokens.
- Retrieval head: attention head/layer role treated as long-range evidence sensitive.
- Side-channel boundary: isolation rule that prevents shared-cache access patterns from leaking private context.
