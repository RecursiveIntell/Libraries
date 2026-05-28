# Manual Injection 00 — Prestart Authority and Drift Guard

Before you begin, restate the hard constraints:

- TurboQuant math belongs only in the `turbo-quant` crate.
- semantic-memory may only adapt/call the real crate.
- TurboQuant must be optional and non-default.
- Raw/f32 and current SQ8 behavior remain intact.
- Approximate results must disclose approximation/degradation.
- No absolute path dependencies.
- No local shadow codec.
- Stop if source layout makes clean integration impossible.

Produce a source-basis report before implementation. Do not start coding until Phase 0 is complete.
