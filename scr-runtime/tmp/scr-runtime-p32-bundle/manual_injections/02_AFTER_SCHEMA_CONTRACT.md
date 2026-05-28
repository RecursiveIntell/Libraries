Manual invariant injection after Phase 02:

Show schema/Rust parity evidence:
1. every non-empty Rust field has schema `minLength` or documented alternative,
2. unknown fields are rejected,
3. recorded-time semantics are decided,
4. negative schema fixtures fail,
5. Rust tests or scripts cover parity.

If schema is weaker than Rust, stop and fix.
