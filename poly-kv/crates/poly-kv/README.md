# poly-kv

`poly-kv` is a Rust research-to-implementation crate for shared KV-cache pool manifests, q8 key compression, exact fallback, realized accounting, and receipt-bearing compressed-pool experiments.

| Capability | Status |
|---|---|
| shared pool manifests | implemented / tested |
| exact fallback | implemented / tested |
| q8 key reference path | implemented / tested on synthetic fixtures |
| raw exact value codec | implemented / tested |
| persisted compression eval receipts | implemented / tested |
| full-block decode receipt disclosure | implemented / tested |
| mixed reader scratch accounting | implemented / tested |
| TurboQuant value adapter | optional / experimental / unsupported stub until API inspection |
| FibQuant value adapter | optional / experimental / unsupported stub until API inspection |
| real model benchmarks | not yet reproduced |
| serving runtime integration | not implemented |
| adaptive controller | deferred |

The crate owns shared pool semantics only. It does not implement adaptive routing, runtime permits, app truth stores, or TurboQuant/FibQuant math.

Every fallback decode is explicit in `DecodeReceiptV1` through `FallbackReceiptV1`. Shape and span mismatches return typed errors. Decode receipts also disclose full-block decode behavior, returned value count, scratch bytes, and owned-copy behavior.

This crate is independent and does not claim affiliation with or endorsement by the PolyKV paper authors.
