PHASE 1 GUARDRAIL — TYPE LAW

Verify:
1. Durable score types are fixed-point/integer only.
2. Public constructors reject invalid values.
3. External refs do not claim canonical ownership.
4. Rust types are source of truth for generated schemas.
5. No decision path returns a naked bool.
6. No f32/f64 appears in durable score artifacts.
