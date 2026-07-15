# Phase 3 — Interchangeable codec contract

Issue: `INT-001`, final semantic closure of `CMP-001`.

Freeze in core: canonical CodecId/ProfileId, envelope magic/schema, profile digest law, typed score
semantics, normalization, capabilities, resource limits, errors, exact-authority rule.

After freeze, independent Turbo and Fib agents implement the contract; conformance agent owns shared
fixtures. Backend agents may not fork the contract.

Migrate semantic-memory to the canonical registry. Reduce scr-runtime-compression to routing and
validation. Poly-KV keeps raw exact values authoritative.

Exit: backend substitution requires no domain-code branch; all capabilities/errors explicit; common
conformance passes; sidecars migrated or rebuilt.
