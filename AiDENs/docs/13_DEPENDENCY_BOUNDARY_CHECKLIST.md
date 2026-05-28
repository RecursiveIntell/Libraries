# 13 — Dependency Boundary Checklist

Before every commit, check:

- `aidens-contracts` imports no `aidens-*` crate.
- `aidens-boundary-kit` imports no runner/app/shell crate.
- `aidens-receipts` imports no memory/provider/tool runtime crate.
- `aidens-provider-kit` imports no tool/memory/queue crate.
- `aidens-tool-kit` imports no app-specific tool crate.
- `aidens-runner` imports no Tauri/daemon/web shell crate.
- `aidens-app-kit` does not define canonical artifact law.
- shell crates do not mutate runtime truth directly.

If a dependency violation looks convenient, it is probably the blob returning.
