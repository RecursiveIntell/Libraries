# DB_SCHEMA.md
# forge.db — Schema and Migration Rules

## Requirements
- Forge DB is entirely separate from semantic-memory DB.
- Forge refuses to open DBs lacking the schema signature.
- `PRAGMA user_version` is used for schema versioning.
- All migrations affect forge.db only, never memory.db.

---

## Schema constants (compile-time)
```rust
pub const FORGE_SCHEMA_HASH: &str = "<blake3 of CREATE TABLE statements, alphabetical order>";
pub const FORGE_MIN_USER_VERSION: u32 = 1;
pub const FORGE_MAX_USER_VERSION: u32 = 999;
pub const FORGE_CURRENT_USER_VERSION: u32 = 1;
```
`FORGE_SCHEMA_HASH` is computed as:
```
blake3(sorted CREATE TABLE statements, stripped of comments and extra whitespace)
```
Recompute whenever tables change. Store in `forge_meta` on DB creation.

---

## Tables

### forge_meta
```sql
CREATE TABLE forge_meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
-- Required rows on creation:
-- INSERT INTO forge_meta VALUES ('schema_hash',    '<FORGE_SCHEMA_HASH>');
-- INSERT INTO forge_meta VALUES ('schema_version', '1');
-- INSERT INTO forge_meta VALUES ('created_at',     '<ISO8601>');
```

### candidates
```sql
CREATE TABLE candidates (
  candidate_id  TEXT PRIMARY KEY,
  spec_json     TEXT NOT NULL,
  parents_json  TEXT NOT NULL DEFAULT '[]',
  created_at    TEXT NOT NULL,
  status        TEXT NOT NULL   -- 'active' | 'retired' | 'promoted'
);
```

### tasks
```sql
CREATE TABLE tasks (
  task_id          TEXT PRIMARY KEY,
  suite_name       TEXT NOT NULL,
  fixture_ref      TEXT NOT NULL,
  prompt           TEXT NOT NULL,
  constraints_json TEXT NOT NULL,
  weights_json     TEXT NOT NULL,
  created_at       TEXT NOT NULL
);
```

### eval_runs
```sql
CREATE TABLE eval_runs (
  eval_id          TEXT PRIMARY KEY,
  candidate_id     TEXT NOT NULL,
  task_id          TEXT NOT NULL,
  backend          TEXT NOT NULL,   -- 'host' | 'container'
  seed             INTEGER NOT NULL,
  mindstate_hash   TEXT NOT NULL,
  patch_hash       TEXT NOT NULL,
  diff_hash        TEXT NOT NULL,
  scores_json      TEXT NOT NULL,
  violations_json  TEXT NOT NULL,
  logs_ref         TEXT NOT NULL,   -- path to log file or 'inline:<base64>'
  cea_run_hash     TEXT,            -- NULL if CEA not enabled for this run
  created_at       TEXT NOT NULL
);
CREATE INDEX idx_eval_runs_candidate ON eval_runs(candidate_id);
CREATE INDEX idx_eval_runs_task      ON eval_runs(task_id);
```

### archive_cells
```sql
CREATE TABLE archive_cells (
  cell_key             TEXT PRIMARY KEY,
  candidate_id         TEXT NOT NULL,
  score_summary_json   TEXT NOT NULL,
  cea_fingerprint      TEXT,        -- NULL if CEA not yet available
  updated_at           TEXT NOT NULL
);
```

### promotions
```sql
CREATE TABLE promotions (
  version_id          TEXT PRIMARY KEY,   -- 'v0001', 'v0002', ...
  candidate_id        TEXT NOT NULL,
  frozen_spec_json    TEXT NOT NULL,
  bounds_json         TEXT NOT NULL,
  invariants_json     TEXT NOT NULL,
  checksum            TEXT NOT NULL,      -- blake3 of frozen_spec + bounds + invariants
  cea_fingerprint_json TEXT,              -- frozen CEA fingerprint for drift detection
  promoted_at         TEXT NOT NULL
);
```

### answer_traces
```sql
CREATE TABLE answer_traces (
  trace_id            TEXT PRIMARY KEY,
  question_sig        TEXT NOT NULL,
  version_id          TEXT NOT NULL,
  strategy_tags_json  TEXT NOT NULL,
  patch_hash          TEXT NOT NULL,
  diff_hash           TEXT NOT NULL,
  score_json          TEXT NOT NULL,
  created_at          TEXT NOT NULL
);
CREATE INDEX idx_answer_traces_question_sig ON answer_traces(question_sig);
```

### cea_nodes
```sql
CREATE TABLE cea_nodes (
  node_id    TEXT PRIMARY KEY,
  node_kind  TEXT NOT NULL,   -- 'cause' | 'effect'
  sig_json   TEXT NOT NULL,   -- EditOpSignature or EffectSignature (no raw source)
  first_seen TEXT NOT NULL,
  last_seen  TEXT NOT NULL
);
```

### cea_edges
```sql
CREATE TABLE cea_edges (
  edge_id        TEXT PRIMARY KEY,
  cause_node_id  TEXT NOT NULL REFERENCES cea_nodes(node_id),
  effect_node_id TEXT NOT NULL REFERENCES cea_nodes(node_id),
  weight         REAL NOT NULL DEFAULT 0.0,
  count          INTEGER NOT NULL DEFAULT 0,
  confidence     REAL NOT NULL DEFAULT 0.0,
  version_id     TEXT NOT NULL,
  last_seen      TEXT NOT NULL,
  UNIQUE(cause_node_id, effect_node_id, version_id)
);
CREATE INDEX idx_cea_edges_cause   ON cea_edges(cause_node_id);
CREATE INDEX idx_cea_edges_effect  ON cea_edges(effect_node_id);
CREATE INDEX idx_cea_edges_version ON cea_edges(version_id);
```

### cea_run_log
```sql
CREATE TABLE cea_run_log (
  run_hash      TEXT PRIMARY KEY,   -- observation idempotency key; legacy rows may use content hash
  eval_id       TEXT NOT NULL,
  edges_added   INTEGER NOT NULL,
  edges_updated INTEGER NOT NULL,
  processed_at  TEXT NOT NULL
);
```

---

## Migration policy
- Each migration file: `migrations/<N>_<description>.sql`
- Migrations run in ascending numeric order.
- Each migration must:
  1. Bump `PRAGMA user_version` to N.
  2. `UPDATE forge_meta SET value = '<new_hash>' WHERE key = 'schema_hash'`.
  3. `UPDATE forge_meta SET value = '<N>' WHERE key = 'schema_version'`.
- Migrations are one-way (no down migrations in v1).
- Never modify `memory.db` in a migration — any migration touching a non-forge path is a bug.

---

## Initial migration (migration 1)
`migrations/1_initial.sql`:
```sql
PRAGMA user_version = 1;

CREATE TABLE forge_meta ( ... );
CREATE TABLE candidates ( ... );
CREATE TABLE tasks ( ... );
CREATE TABLE eval_runs ( ... );
CREATE TABLE archive_cells ( ... );
CREATE TABLE promotions ( ... );
CREATE TABLE answer_traces ( ... );
CREATE TABLE cea_nodes ( ... );
CREATE TABLE cea_edges ( ... );
CREATE TABLE cea_run_log ( ... );

-- All indexes

INSERT INTO forge_meta VALUES ('schema_hash',    '<FORGE_SCHEMA_HASH>');
INSERT INTO forge_meta VALUES ('schema_version', '1');
INSERT INTO forge_meta VALUES ('created_at',     strftime('%Y-%m-%dT%H:%M:%SZ', 'now'));
```
