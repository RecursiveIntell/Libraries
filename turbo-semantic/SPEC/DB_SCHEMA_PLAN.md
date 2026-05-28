# DB / Persistence Plan

## Principle

Do not mutate existing raw embedding semantics. Add optional sidecar persistence.

## Preferred tables

### `vector_codec_profiles`

```sql
CREATE TABLE IF NOT EXISTS vector_codec_profiles (
    profile_digest TEXT PRIMARY KEY,
    codec_family TEXT NOT NULL,
    codec_version TEXT NOT NULL,
    dim INTEGER NOT NULL,
    bits INTEGER,
    projections INTEGER,
    seed TEXT,
    rotation_kind TEXT,
    radius_encoding TEXT,
    angle_encoding TEXT,
    qjl_encoding TEXT,
    distance_metric TEXT,
    canonical_profile_json TEXT NOT NULL,
    created_recorded_at TEXT NOT NULL
);
```

### `encoded_vector_artifacts`

```sql
CREATE TABLE IF NOT EXISTS encoded_vector_artifacts (
    entity_type TEXT NOT NULL,
    entity_key TEXT NOT NULL,
    codec_family TEXT NOT NULL,
    profile_digest TEXT NOT NULL,
    encoded_bytes BLOB NOT NULL,
    checksum TEXT NOT NULL,
    encoded_len INTEGER NOT NULL,
    created_recorded_at TEXT NOT NULL,
    encode_receipt_json TEXT NOT NULL,
    degradation_json TEXT,
    PRIMARY KEY (entity_type, entity_key, codec_family, profile_digest),
    FOREIGN KEY (profile_digest) REFERENCES vector_codec_profiles(profile_digest)
);
```

### `vector_codec_eval_runs`

```sql
CREATE TABLE IF NOT EXISTS vector_codec_eval_runs (
    run_id TEXT PRIMARY KEY,
    codec_family TEXT NOT NULL,
    profile_digest TEXT NOT NULL,
    corpus_snapshot TEXT NOT NULL,
    query_count INTEGER NOT NULL,
    recall_at_10 REAL,
    top_k_agreement REAL,
    score_correlation REAL,
    avg_encoded_bytes INTEGER,
    raw_f32_bytes INTEGER,
    sq8_bytes INTEGER,
    latency_json TEXT NOT NULL,
    degradation_count INTEGER NOT NULL,
    created_recorded_at TEXT NOT NULL,
    report_json TEXT NOT NULL
);
```

## Migration discipline

- Add migrations through existing semantic-memory migration machinery.
- Migration must be idempotent.
- Existing DB tests must pass.
- Downgrade is not required, but rollback instructions must say how to ignore/drop sidecar tables.

## Non-goal

Do not rewrite existing embedding storage in this pass.
