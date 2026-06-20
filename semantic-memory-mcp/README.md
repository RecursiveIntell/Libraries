# semantic-memory-mcp

Local-first knowledge management MCP server with evidence-scored retrieval, contradiction detection, temporal awareness, and autonomous memory lifecycle management.

Works with **Hermes Agent**, **Claude Desktop**, **Cursor**, **Windsurf**, and any MCP-compatible client.

## What This Gives Your Agent

Your agent gets a knowledge base that:

- **Searches by meaning, not just keywords** — hybrid BM25 + vector + Reciprocal Rank Fusion
- **Tracks evidence confidence** — every item carries algebraic provenance (semiring confidence)
- **Detects and corrects contradictions** — syndrome detection + belief propagation on conflict graphs
- **Decays old knowledge** — temporal weight with age, supersession, support, and contradiction factors
- **Discovers related knowledge** — second-order retrieval through graph neighbors (discord)
- **Adapts search strategy per query** — adaptive routing decides which stages to run
- **Garbage-collects safely** — lawful subtraction with invariant verification and recovery
- **Audits every operation** — blake3-digested receipts for every mutation, replayable
- **Tracks causal history** — episodes link operations into causal chains

The combination of hybrid retrieval, provenance-weighted belief propagation, typed graph edges, and autonomous lifecycle management in a single Rust-first local substrate is uncommon. This is knowledge management, not just vector search.

## Installation

```bash
cargo install --path . --features full
```

Or build from source:

```bash
git clone https://github.com/RecursiveIntell/Libraries.git
cd Libraries/semantic-memory-mcp
cargo build --release --features full
# Binary: target/release/semantic-memory-mcp
```

## Configuration

### Hermes Agent

Add to `~/.hermes/config.yaml`:

```yaml
mcp_servers:
  semantic_memory:
    command: "semantic-memory-mcp"
    args: ["--memory-dir", "/home/user/.local/share/semantic-memory"]
```

### Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "semantic_memory": {
      "command": "semantic-memory-mcp",
      "args": ["--memory-dir", "/home/user/.local/share/semantic-memory"]
    }
  }
}
```

### Cursor / Windsurf

Add to MCP settings:

```json
{
  "mcpServers": {
    "semantic_memory": {
      "command": "semantic-memory-mcp",
      "args": ["--memory-dir", "/home/user/.local/share/semantic-memory"]
    }
  }
}
```

## CLI Options

```
semantic-memory-mcp --memory-dir <DIR> [OPTIONS]

Options:
  --memory-dir <DIR>         Path to the memory store directory (required, created if absent)
  --embedding-url <URL>      Ollama embedding server URL (default: http://localhost:11434)
  --embedding-model <NAME>   Embedding model name (default: nomic-embed-text)
  --embedding-dims <N>       Embedding dimensions (default: 768)
```

Note: `--memory-dir` is a directory path, not a SQLite file path. The SQLite database is created as `memory.db` inside this directory. (The old `--db-path` flag has been renamed to avoid confusion — it was always a directory, not a file.)

## Prerequisites

- **Ollama** running locally with an embedding model (e.g. `nomic-embed-text` or `all-minilm:latest`)
  ```bash
  ollama pull nomic-embed-text
  ```

## Tools Exposed

| Tool | Description |
|------|-------------|
| `sm_search` | Hybrid BM25+vector+RRF semantic search |
| `sm_search_explained` | Search with full score breakdown (RRF, BM25, vector contributions) |
| `sm_add_fact` | Add a fact to the knowledge base |
| `sm_ingest_document` | Ingest a document with automatic chunking |
| `sm_stats` | Get knowledge base statistics (facts, chunks, graph edges, DB size) |
| `sm_graph_path` | Find path between items in the knowledge graph (with edge evidence) |
| `sm_route_query` | Profile a query and get adaptive routing decision |
| `sm_search_with_routing` | Adaptive search with factor graph belief propagation |
| `sm_decoder_analyze` | Detect contradictions and compute corrections |
| `sm_discord_search` | Second-order retrieval from graph neighbors (store-backed) |
| `sm_set_provenance` | Set evidence confidence for an item |
| `sm_run_lifecycle` | Run autonomous memory health check |
| `sm_add_graph_edge` | Add a durable, typed graph edge between two nodes |
| `sm_list_graph_edges` | List graph edges for a node or all edges |
| `sm_invalidate_graph_edge` | Invalidate a stored graph edge (append-only) |
| `sm_factor_graph` | Run factor graph belief propagation (store-backed) |
| `sm_topology` | Find topological voids in the knowledge graph (store-backed) |
| `sm_community` | Detect communities in the knowledge graph (store-backed) |

Note: The README tool table may lag behind the actual tool surface. Use `tools/list` as the source of truth for available tools.

## Feature Flags

| Feature | Description |
|---------|-------------|
| `default` | All features (full tool surface) — this is the default build |
| `search` | Core search only (BM25+vector+RRF, add facts, stats, graph) — minimal build |
| `full` | All features: provenance, temporal, decoder, discord, routing, subtraction, integration, topology, community |

## Architecture

```
semantic-memory-mcp (MCP stdio server)
  └── semantic-memory (Rust library)
        ├── SQLite (authoritative storage, FTS5, WAL)
        ├── Vector backend (usearch / hnsw / brute-force)
        ├── Provenance (4 semirings: Boolean, Tropical, Probability, Confidence)
        ├── Temporal weight (age + supersession + support + contradiction + wells)
        ├── Decoder (syndromes + corrections + belief propagation)
        ├── Subtraction (lawful forgetting + invariant verification)
        ├── Compression governor (importance-driven per-vector quantization)
        ├── Routing (query profiling + adaptive stage selection)
        ├── Discord (second-order graph-neighbor retrieval)
        ├── Stored graph edges (durable, typed, append-only with invalidation)
        ├── Factor graph (unified probabilistic reasoning over all edge types)
        ├── Topology (Betti numbers, void detection)
        └── Integration (cross-feature wiring)
```

## License

Apache-2.0

## Links

- [semantic-memory crate](https://github.com/RecursiveIntell/Libraries/tree/main/semantic-memory)
- [MCP Protocol](https://modelcontextprotocol.io/)
- [rmcp Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)