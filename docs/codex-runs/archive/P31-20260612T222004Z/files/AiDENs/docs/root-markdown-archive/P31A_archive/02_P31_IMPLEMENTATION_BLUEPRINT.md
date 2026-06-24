# P31 Implementation Blueprint

## Intended crate/module shape

Preferred module layout:

```text
boundary-compiler-core/
  Cargo.toml
  src/
    lib.rs
    types.rs
    json_boundary.rs
    strict_json.rs
    canonical.rs
    treatment.rs
    digest.rs
    error.rs
  tests/
    json_boundary_fixtures.rs
```

If integrating into an existing crate, keep the same logical modules where possible.

## Public API sketch

```rust
pub fn compile_json_boundary(
    profile: &BoundaryCompilerProfileV1,
    raw: &[u8],
    schema: Option<&serde_json::Value>,
    treatment_critical_paths: &[JsonPointerLikePath],
) -> BoundaryCompileResultV1;
```

A smaller equivalent is acceptable if it preserves semantics.

## Core type sketch

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BoundaryCompilerProfileV1 {
    pub profile_id: String,
    pub language: BoundaryLanguage,
    pub dialect: String,
    pub schema_id: Option<String>,
    pub schema_version: Option<String>,
    pub canonicalization: CanonicalizationProfile,
    pub duplicate_key_policy: AmbiguityPolicy,
    pub unknown_field_policy: UnknownFieldPolicy,
    pub coercion_policy: CoercionPolicy,
    pub repair_policy: RepairPolicy,
    pub resource_ceilings: ResourceCeilingsV1,
    pub trust_boundary: TrustBoundary,
    pub treatment_critical_paths: Vec<JsonPointerLikePath>,
    pub allowed_degradation: Vec<String>,
    pub allowed_top_level_fields: Option<BTreeSet<String>>,
    pub expected_field_types: BTreeMap<String, ExpectedJsonType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParseReceiptV1 {
    pub receipt_id: String,
    pub raw_digest: DigestHex,
    pub parsed_digest: Option<DigestHex>,
    pub canonical_digest: Option<DigestHex>,
    pub parser: String,
    pub dialect: String,
    pub status: ParseStatus,
    pub errors: Vec<BoundaryErrorRecordV1>,
    pub ambiguity_detected: bool,
    pub resource_ceiling_triggered: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepairReceiptV1 {
    pub receipt_id: String,
    pub repair_operator: String,
    pub before_digest: DigestHex,
    pub after_digest: DigestHex,
    pub changed_paths: Vec<JsonPointerLikePath>,
    pub rationale: String,
    pub semantic_impact: SemanticImpact,
    pub allowed_changes: Vec<String>,
    pub disallowed_changes: Vec<String>,
    pub treatment_integrity_status: TreatmentIntegrityDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TreatmentIntegrityReceiptV1 {
    pub receipt_id: String,
    pub treatment_critical_paths: Vec<JsonPointerLikePath>,
    pub before_hashes: BTreeMap<String, Option<DigestHex>>,
    pub after_hashes: BTreeMap<String, Option<DigestHex>>,
    pub differences: Vec<TreatmentDifferenceV1>,
    pub decision: TreatmentIntegrityDecision,
    pub waiver: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum BoundaryDecisionV1 {
    Accept,
    Reject,
    Quarantine,
    RepairedAccept,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BoundaryCompileResultV1 {
    pub decision: BoundaryDecisionV1,
    pub value: Option<serde_json::Value>,
    pub canonical_bytes: Option<Vec<u8>>,
    pub raw_digest: DigestHex,
    pub canonical_digest: Option<DigestHex>,
    pub parse_receipt: ParseReceiptV1,
    pub repair_receipt: Option<RepairReceiptV1>,
    pub treatment_integrity_receipt: Option<TreatmentIntegrityReceiptV1>,
    pub errors: Vec<BoundaryErrorRecordV1>,
}
```

If `schemars` is not available or would cause dependency issues, omit `JsonSchema` for P31 and document it as P32/P33 debt.

## Strict duplicate-key parser approach

Use a custom deserializer target rather than `serde_json::Value`:

```rust
enum StrictJsonValue {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<StrictJsonValue>),
    Object(BTreeMap<String, StrictJsonValue>),
}
```

Implement `Deserialize` with a `Visitor` whose object path does:

```rust
while let Some(key) = map.next_key::<String>()? {
    if seen.contains(&key) {
        return Err(de::Error::custom(format!("duplicate key: {key}")));
    }
    seen.insert(key.clone());
    let value = map.next_value::<StrictJsonValue>()?;
    object.insert(key, value);
}
```

This ensures duplicate-key ambiguity is caught before conversion to `serde_json::Value`.

## Canonicalization profile

For P31, implement `StableSortedJsonV1`:

- object keys sorted;
- no insignificant whitespace;
- arrays preserve order;
- stable primitive serialization;
- digest computed over canonical bytes.

Do not claim full JCS/RFC 8785 unless fully implemented.

## Decision precedence

Recommended precedence:

1. max bytes failure → reject;
2. parse/malformed failure → reject;
3. duplicate key failure → reject/quarantine per profile;
4. resource depth/key failure → reject/quarantine per profile;
5. unknown field failure → reject/quarantine per profile;
6. coercion/type mismatch → reject/quarantine per profile;
7. treatment-critical missing/touched → emit treatment receipt; reject/quarantine only if profile says missing critical path is fatal;
8. accepted → canonical digest + parse receipt.

## P31 schema subset

Full JSON Schema validation is optional for P31. Minimum subset:

- allowed top-level field set;
- per-field expected primitive type;
- explicit policy for unknown fields;
- no automatic coercion.

P32 should replace or extend this with generated schema validation and compatibility gates.
