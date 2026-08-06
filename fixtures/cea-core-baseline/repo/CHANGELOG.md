# Changelog

## Unreleased

- Breaking: `EditOpSignature` now uses typed enums/newtypes (`EditOpKind`, `AnchorKind`, `ScopeTag`, `OpIndex`, `FileIndex`) and `AttributionTriple` now carries fractional `weight`.
- Breaking: `CausalGraph::node_index_map` is now a `HashMap` instead of a `BTreeMap`.
- Non-breaking: attribution is now proportional and deterministic, pass credit is differential, run hashes are order-stable, graph persistence/decay is supported, and prediction uses Beta-backed edge confidence plus fuzzy fallback.
