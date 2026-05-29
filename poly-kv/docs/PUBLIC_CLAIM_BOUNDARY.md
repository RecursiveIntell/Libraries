# README / crates.io Claim Boundary

## Safe claims after alpha gates pass

- `poly-kv` is a Rust research-to-implementation crate for shared KV-cache pool experiments.
- It provides typed manifests and receipt types for building and attaching readers to a shared KV pool.
- It includes exact fallback and synthetic tests for shape, memory accounting, q8 key quantization, and deterministic receipts.
- It is intended to become one primitive under a future adaptive compression controller.

## Unsafe claims until reproduced locally

- production-ready
- universal runtime compatibility
- vLLM compatibility
- llama.cpp compatibility
- Candle/Burn/tch compatibility
- specific memory reduction percentages
- no quality loss
- benchmark superiority
- original PolyKV authorship or affiliation
- exact reproduction of PolyKV paper metrics
- adaptive compression completed

## Required attribution draft

> This crate is an independent Rust research-to-implementation effort inspired by PolyKV-style shared compressed KV-cache pool ideas. It is not the original authors' reference implementation and does not claim affiliation with the PolyKV paper authors.

## Mandatory README status table

| Capability | Status |
|---|---|
| shared pool manifests | implemented / tested |
| exact fallback | implemented / tested |
| q8 key reference path | implemented / tested |
| TurboQuant value adapter | optional / experimental / API-inspected |
| FibQuant value adapter | optional / experimental / API-inspected |
| real model benchmarks | not yet reproduced unless evidence added |
| serving runtime integration | not implemented |
| adaptive controller | deferred |
