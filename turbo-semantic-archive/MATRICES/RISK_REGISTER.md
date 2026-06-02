# Risk Register

| Risk | Severity | Why it matters | Mitigation |
|---|---:|---|---|
| TurboQuant current byte layout is not a storage win | High | If promoted early, integration may make storage worse | Shadow/eval first; byte-size gates |
| Absolute path dependency | High | Breaks portability and zip/source certifier behavior | Require sibling crate or stop |
| Local reimplementation in semantic-memory | High | Creates drift and shadow codec | Adapter-only rule + grep script |
| Approximate results hidden as exact | High | Violates provenance/truth discipline | Result provenance + tests |
| Existing SQ8/HNSW regression | High | Loses known stable behavior | Existing tests must pass |
| Evaluation too synthetic | Medium | MockEmbedder may hide real retrieval behavior | Start with fixture; next pass real corpus |
| Dense rotation cost | Medium | Query/index may be too slow | Prepared query; future SRHT |
| Schema drift | Medium | Old encoded vectors become ambiguous | Profile digest + versioned artifact |
| Shadow failure breaks writes | Medium | Optional feature becomes reliability risk | non-strict degradation default |
