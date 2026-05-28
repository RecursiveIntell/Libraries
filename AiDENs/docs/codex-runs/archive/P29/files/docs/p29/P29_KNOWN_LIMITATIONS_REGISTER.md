# P29 Known Limitations Register

Status: Phase 20 converged; update again after Phase 21 package replay.

## Active Limitations

| ID | Limitation | Support effect |
|---|---|---|
| P29-LIM-001 | Final strict package has not been generated. | No release claim. |
| P29-LIM-002 | Extracted package self-replay has not been run on a final P29 zip. | No package replay claim. |
| P29-LIM-003 | v11A evidence is implemented only for the declared supported-local coding-agent path; final package and extracted replay remain pending. v11B surfaces are executable seed only. | No broad v11A, full v11B, or v11C claim. |
| P29-LIM-004 | `BUG-190` through `BUG-200` were triaged as unaudited high-risk surfaces and quarantined rather than repaired during P29. | No support widening for those layers during P29. |
| P29-LIM-005 | Final command bar has not yet been rerun after Phase 20 docs convergence. | No final support label until Phase 21 passes. |
| P29-LIM-006 | Some Phase 05-07 audit items are quarantined where the safe fix would require broader API redesign. | No exact claim for those specific bug IDs until a later pass closes them. |
| P29-LIM-007 | Phase 08 vector/HNSW items `BUG-101`, `BUG-103`, `BUG-104`, `BUG-105`, `BUG-114`, `BUG-115`, and `BUG-119` remain quarantined. | No exact vector/HNSW format or concurrency claim for those items. |
| P29-LIM-008 | Phase 09 pool, baseline, and large-dataset behaviors remain quarantined where repair requires broader owner/API changes. | No broad concurrency or baseline-provenance release claim. |
| P29-LIM-009 | Phase 10 low and medium graph/query/API cleanup items remain quarantined outside the small local correctness repairs. | No claim that graph, chunker, or query APIs are fully closed beyond tested fixes. |
| P29-LIM-010 | Phase 11 baseline provenance, baggage deserialization validation, proof budget, and broad medium-risk contract items remain quarantined. | No release-candidate claim for those specific semantics. |
| P29-LIM-011 | v11B region, boundary, residual, syndrome, convergence, and subtraction surfaces are advisory executable seeds with canonical-owner backpointers. | No active v11B runtime, mutation, or cross-region admission claim. |
