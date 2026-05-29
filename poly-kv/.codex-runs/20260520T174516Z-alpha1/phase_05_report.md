# Phase 05 Report - Raw Exact Fallback, q8 Keys, and Value Boundary

Status: passed.

Implemented:

- `ExactKvBlock`, `ExactFallback`, `RawExactCodec`
- `Q8KeyCodec` symmetric per-block reference implementation
- `ValueCodec` trait and `RawExactValueCodec`
- feature-gated `TurboQuantValueCodec` and `FibQuantValueCodec` unsupported stubs

Guardrail result:

- Exact fallback is required for pool builds.
- q8 key compression has eval output and bounded synthetic drift tests.
- TurboQuant/FibQuant math was not reimplemented.
- Optional adapters do not claim compatibility.

Blockers: none.
