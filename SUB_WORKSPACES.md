# Libraries workspace — sub-workspaces and nested crates

The `Libraries/` root Cargo workspace (`Cargo.toml`) contains 53
first-class workspace members, all of which share a single `target/`
build directory and the same `Cargo.lock`. There are **three
intentional sub-workspaces** in the tree that live outside the root
workspace for the reasons documented below.

## Why sub-workspaces exist

Some crate families in RecursiveIntell have grown to the point where:

- They have their own release cadence (poly-kv, scr-runtime)
- They span **more than one binary** (scr-cli + scr-kernel + scr-audit-adapter
  + scr-reference)
- They carry a **separate lockfile** for their own dependencies
  (AiDENs), and we don't want to force those pins onto the parent
  workspace's other crates
- They are vendored from a separate project (AiDENs/) and may move in
  or out of the parent tree independently

For each, the choice to keep them as sub-workspaces — not nested members
of the root `Cargo.toml` — is a deliberate fragility boundary.

## The three sub-workspaces

### 1. `poly-kv/` — shared compressed KV-cache pool

`poly-kv/Cargo.toml` is its own `[workspace]`. Members:

- `poly-kv` — the pool itself (v0.1.0-alpha.1, 76 tests)
- `poly-kv-python` — optional PyO3 sidecar bindings
- `quant-codec-core` — shared codec/profile/shape traits (v0.1.0-alpha.1)

Two of those three (`poly-kv` + `quant-codec-core`) are also declared
as members of the **parent** Libraries workspace — the parent
references them by their full path (`poly-kv/crates/poly-kv`). This
intentional duplication means the parent can depend on them via path
deps while the poly-kv sub-workspace still resolves them internally
for its own tests and examples. The rust-version is aligned (1.75
across both) so the union compiles.

Build/test the poly-kv sub-workspace in isolation:

```
cd poly-kv
cargo test --workspace
```

The sub-workspace's `target/` is separate from the parent's.

### 2. `AiDENs/` — agent infrastructure kit

`AiDENs/Cargo.toml` is its own `[workspace]`. It contains 34
sub-crates, each a "kit" (`aidens-contracts`, `aidens-boundary-kit`,
`aidens-receipts`, `aidens-capability-kit`, `aidens-provider-kit`,
`aidens-tool-kit`, etc.). AiDENs is a parallel product surface that
**does not** participate in the parent Libraries workspace — it
has its own dependencies, its own lockfile, and its own version cadence.

Build/test AiDENs in isolation:

```
cd AiDENs
cargo test --workspace
```

Treat AiDENs as a vendored upstream — modify it carefully and
prefer upstream PRs when possible.

### 3. `scr-runtime/` — semantic compression runtime

`scr-runtime/Cargo.toml` is its own `[workspace]`. Members:

- `scr-kernel` — the SCR kernel
- `scr-cli` — the SCR command-line interface
- `scr-audit-adapter` — bridge to the audit trail
- `scr-reference` — reference implementations

`sscr-runtime-compression` at the parent workspace level is the
*integration* crate (codec_dispatch, policy routing through
quant-governor) — it is a parent workspace member. The kernel/CLI
live in the sub-workspace.

Build/test scr-runtime in isolation:

```
cd scr-runtime
cargo test --workspace
```

## What is NOT a sub-workspace

- `Primitives/` — contains 10 sub-crates, **all members of the parent
  workspace** (`Primitives/cea-core`, `Primitives/check-runner-sys`,
  etc.). They share the parent's `target/` and `Cargo.lock`.
- `living-memory/` — the directory contains 56 markdown design docs
  plus one Rust crate (`living-memory/living-memory/`, package name
  `forge-engine`, v0.2.0, 170 tests). The crate is a parent workspace
  member; the markdown is design material only.

## Adding a new sub-workspace

If you need to add a fourth sub-workspace:

1. The sub-workspace's `Cargo.toml` MUST start with `[workspace]`
   (not `[package]`) and have its own `members = [...]` list.
2. The sub-workspace's `rust-version` MUST match the parent's (1.75)
   so cross-crate builds don't fail with edition-mismatch errors.
3. The sub-workspace's crates that the parent depends on MUST be
   declared in **both** workspaces (parent `members = [...]` with
   the full sub-path, and sub-workspace's own `members = [...]`).
4. Document it here, with build/test commands and a one-line
   rationale for why it can't be a parent member.

## Rules of thumb

- One workspace is the default. Reach for a sub-workspace only when
  one of the conditions above applies.
- Sub-workspaces cost you: no parent-level `cargo test --workspace`
  reachability, no shared `Cargo.lock`, more `Cargo.toml` files to
  keep in sync.
- Sub-workspaces buy you: independent release cadence, scoped
  `target/` directories, isolation from sibling crates that have
  fragile path deps.
