# Seed risks

| Risk | Phase | Description | Mitigation |
|---|---:|---|---|
| R-001 | 1 | ID public wire/API break | Dual-read/single-write, versioned adapters, golden fixtures |
| R-002 | 2 | Digest V2 reidentifies history | Preserve V1, append supersession, never reinterpret |
| R-003 | 3 | Codec trait becomes lowest-common denominator | Typed capabilities/metrics and conformance |
| R-004 | 3 | Existing sidecars unreadable | Versioned readers or rebuild from raw authority |
| R-005 | 4 | Queue atomicity changes fairness/throughput | Race tests and benchmark receipts |
| R-006 | 5 | Full CI exposes native/platform gaps | Required-lane classification; blocked is not pass |
| R-007 | 5 | Evidence validates itself | Read-only verify, separate record, source/log digests |
| R-008 | all | Parallel agents overlap authorities | Path locks, frozen contracts, shared-file owner |
