# Types and Schema Law

## Rust is canonical

Rust types are the source of truth for P0A.

Generated JSON schemas are derived from Rust types and checked into `schemas/generated/`.

CI/check scripts must fail if schema regeneration changes checked-in schema files.

## Durable scores

Use:

```rust
pub struct ScoreBps(u16);  // 0..=10000
pub struct WeightBps(u16); // 0..=10000
```

Do not use `f32` or `f64` in durable score artifacts.

## No naked bool decisions

Forbidden:

```rust
pub fn should_allow(...) -> bool
pub fn can_apply(...) -> bool
pub fn is_safe(...) -> bool
```

Required:

```rust
pub fn evaluate(...) -> Result<ControlDecisionReceiptV1, ScrError>
```

Every decision path must produce a receipt or explicit error.
