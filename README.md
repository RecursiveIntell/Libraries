# RecursiveIntell Libraries

Reusable Rust components for local-first AI: semantic memory, agent execution, evidence and provenance, governed tool use, and experimental compression.

This repository brings those components together so their contracts can be developed and tested in one place. SQLite-backed memory owns durable records. Graphs coordinate work. Bridges translate between owners. Receipts make decisions and transformations inspectable. Retrieval indexes, compressed representations, and runtime projections remain derived from their authoritative sources.

Use an individual crate when you need one capability, or explore the integration layers when you need the pieces to work together. This is a multi-workspace engineering repository, with different support and release boundaries for different components.

## Start here

| If you need… | Start with… |
|---|---|
| Local facts, documents, conversations, and hybrid keyword/vector search | [semantic-memory](semantic-memory/README.md) |
| Graph execution, routing, parallel branches, checkpoints, and interrupt/resume | [agent-graph](agent-graph/README.md) |
| LLM calls, prompt payloads, parsing, streaming, and sequential chains | [llm-pipeline](llm-pipeline/README.md) |
| Provider-independent tool contracts, registry, dispatch, and receipts | [llm-tool-runtime](llm-tool-runtime/README.md) |
| Claims, evidence links, support judgments, and an append-only provenance ledger | [claim-ledger](claim-ledger/README.md) |
| Verification evidence and its import into memory | [semantic-memory-forge](semantic-memory-forge/README.md) → [forge-memory-bridge](forge-memory-bridge/README.md) → [semantic-memory](semantic-memory/README.md) |
| Bounded memory orchestration and closed-loop execution | [knowledge-runtime](knowledge-runtime/README.md) and [forge-pilot](forge-pilot/README.md) |
| Transcript compaction with loss reports, receipts, and exact fallback references | [context-governor](context-governor/README.md) |
| Vector quantization and compression experiments | [turbo-quant](turbo-quant/README.md), [fib-quant](fib-quant/README.md), and [poly-kv](poly-kv/README.md) |
| Compression policy, runtime adapters, and evaluation | [quant-governor](quant-governor/README.md), [scr-runtime-compression](scr-runtime-compression/README.md), [quant-eval](quant-eval/README.md), and [receipt-bench](receipt-bench/README.md) |
| Local run evidence and activity inspection | [Agent Evidence Workbench](agent-evidence-workbench/README.md) and [stack-monitor](stack-monitor/README.md) |
| Agent runtime contracts and supporting execution primitives | [AiDENs](AiDENs/README.md) and [Primitives](Primitives/README.md) |
| Desktop queues and model/media integrations | [tauri-queue](tauri-queue/README.md), [tauri-react-hooks](tauri-react-hooks/README.md), [comfyui-rs](comfyui-rs/README.md), and [ollama-vision](ollama-vision/README.md) |

The root [Cargo manifest](Cargo.toml) is the source of truth for root workspace membership. Component READMEs describe their APIs, features, examples, and additional requirements.

## Try a local example

Install a current stable Rust toolchain and Git, then clone the repository:

```bash
git clone https://github.com/RecursiveIntell/Libraries.git
cd Libraries

# Inspect the workspace without compiling dependencies.
cargo metadata --no-deps --format-version 1

# Run the checked-in graph example. No model server or API key is required.
cargo run --locked -p agent-graph --example basic
```

The [basic example](agent-graph/examples/basic.rs) runs three nodes, passes state along their edges, and prints the result. More examples cover [parallel execution](agent-graph/examples/parallel.rs), [checkpointing](agent-graph/examples/checkpointing.rs), and [human-in-the-loop execution](agent-graph/examples/human_in_loop.rs).

For a focused package check:

```bash
cargo test --locked -p agent-graph
cargo test --locked -p semantic-memory --no-default-features --features brute-force
```

The second command selects semantic-memory's pure-Rust exact-search backend. Its default `usearch-backend` feature uses a C++ bridge and needs a C++ build toolchain. Examples that generate embeddings or call models have additional provider/model requirements; read the selected package's setup instructions before running them.

## How the pieces fit

### Memory and evidence

[semantic-memory](semantic-memory/src/lib.rs) stores durable records and embeddings in SQLite, combines FTS5 keyword search with vector retrieval, and exposes integrity and reconciliation operations. Approximate indexes accelerate retrieval; they do not replace the database as the record owner.

[claim-ledger](claim-ledger/src/lib.rs) models source-spanned claims, evidence, support admission, contradictions, and hash-chained ledger events. [semantic-memory-forge](semantic-memory-forge/README.md) owns verification evidence and export envelopes. The [bridge](forge-memory-bridge/src/lib.rs) transforms those envelopes into memory import batches without becoming another store or promotion authority.

### Execution and authority

[agent-graph](agent-graph/src/lib.rs) owns graph orchestration. [llm-pipeline](llm-pipeline/src/lib.rs) supplies the payloads that run inside nodes. [llm-tool-runtime](llm-tool-runtime/README.md) carries tool contracts and dispatch receipts. This separation keeps model calls, graph scheduling, and tool authority explicit.

The verification, policy, kernel, and governance crates provide typed contracts and bounded evaluators. Their names do not imply that each crate is a standalone service or a complete security system. See the [support profile](SUPPORT_PROFILE.md) and [scope notes](SCOPE_NOTES.md) for the narrower boundaries.

### Compression and recovery

Compression is an opt-in, workload-dependent path. The codec crates own encoding and decoding; policy and adapter crates select or integrate those paths. Exact source records and fallback behavior remain part of the design.

Read a benchmark together with its input corpus, source revision, feature flags, serialization format, and receipt. Synthetic-fixture recall, projected bit rates, and CPU measurements do not establish universal retrieval parity, deployed GPU memory savings, or lossless reconstruction. The [claim manifest](docs/claims_manifest.json) tracks claim-specific evidence and qualifications.

## Workspace layout

- [Cargo.toml](Cargo.toml): root Rust workspace and default package selection
- [support_lane.toml](support_lane.toml): narrower supported closeout lane used by release checks
- [AiDENs/](AiDENs/), [Primitives/](Primitives/), and [poly-kv/](poly-kv/): additional workspace manifests with their own boundaries
- [context-governor/](context-governor/): standalone Rust workspace for context compaction
- [agent-graph-python/](agent-graph-python/), [llm-pipeline-python/](llm-pipeline-python/), and [context-governor-python/](context-governor-python/): Python-facing integrations
- [tauri-react-hooks/](tauri-react-hooks/): TypeScript package, outside the Rust workspace
- [semantic-memory-mcp](https://github.com/RecursiveIntell/semantic-memory-mcp): Git submodule, excluded from the root Cargo workspace
- [schemas/](schemas/), [fixtures/](fixtures/), and [conformance/](conformance/): contracts and conformance inputs
- [scripts/](scripts/) and [.github/workflows/](.github/workflows/): executable checks and CI definitions
- [docs/](docs/README.md): design, evidence, plans, and historical closeouts

Initialize the MCP submodule only if you need to work on it:

```bash
git submodule update --init semantic-memory-mcp
```

Workspace membership, default members, the supported lane, and published packages are separate sets. A version in a local manifest is not evidence that the same source is available on crates.io. Check the component's own release documentation before choosing a registry dependency or installation command.

## Validation

Start with focused tests for the component you change. The repository exposes additional checks through its [Makefile](Makefile):

```bash
# Documentation and repository-pack consistency.
bash scripts/check_doc_truth.sh
bash scripts/check_pack_truth.sh

# Format, strict Clippy, and tests for the supported lane.
make release-lane

# Verify the recorded release evidence against its bound source/environment.
make gate
```

`make gate` is a read-only evidence verifier. It does not run the full test suite or create fresh evidence. It can reject stale receipts, changed source, shallow history, or a different toolchain/platform. Evidence recording is a separate workflow in [scripts/record_release_evidence.py](scripts/record_release_evidence.py); a failed verification should be investigated rather than relabeled as a pass.

The [CI workflow](.github/workflows/ci.yml) also defines full-workspace format, Clippy, test, and documentation lanes, plus feature-specific checks. The root all-workspace lane includes Tauri/GTK dependencies on Linux. Consult the workflow for its system packages; a successful focused crate test does not certify every workspace or feature combination.

## Contributing and navigating history

Read the applicable [AGENTS.md](AGENTS.md), component instructions, manifests, and tests before making changes. Keep ownership boundaries explicit, add regression coverage for behavior changes, and preserve the receipts and limitations behind public claims.

This repository also retains dated audit packs, implementation prompts, and superseded plans. They are useful history, but current source and executable checks determine what exists today. The [documentation index](docs/README.md) points to the relevant material, including the [archive supersession index](docs/archive/SUPERSESSION_INDEX.md). The older FibQuant implementation bundle is linked there rather than serving as this repository's introduction.

## Licensing

Licenses are specified per component in its `Cargo.toml` or package manifest and accompanying license files. This repository contains multiple license declarations; check the exact component and its dependencies before redistribution. No single license is asserted here for the entire tree.
