# Risk Register

| Risk | Severity | Likelihood | Detection | Mitigation |
|---|---:|---:|---|---|
| Duplicate local type survives under alias | P0 | Medium | duplicate gate + grep aliases | ban compat aliases; quarantine |
| Generated gate misses macro-defined type | P1 | Low | cargo doc/rustdoc if available | manual grep review; future rustdoc JSON gate |
| AiDENs display wrapper becomes truth | P0 | Medium | wrapper backpointer gate | require canonical refs and non-authoritative naming |
| Codex adds feature while fixing compile | P1 | Medium | diff review | explicit non-goal gate |
| Missing canonical dependency causes local substitute | P0 | Medium | dependency gate | add real dependency or quarantine |
| Cargo full workspace too expensive | P1 | Medium | skipped check report | targeted checks + exact skip rationale |
| Schema generation remains local | P0 | Medium | schema scope gate | route through contract-schema-gen |
| Digest identity remains local | P0 | Medium | digest gate | route through stack-ids |
| Stale docs mislead next run | P1 | High | source-basis gate | update docs and final proof |
