-- Additive v13 storage scaffold.
-- This is a design starter, not a production-reviewed migration.

CREATE TABLE IF NOT EXISTS support_sets (
    support_set_id           TEXT PRIMARY KEY,
    claim_id                 TEXT NOT NULL,
    semantics_profile_id     TEXT NOT NULL,
    support_expr_json        TEXT NOT NULL,
    support_tokens_json      TEXT NOT NULL,
    content_digest           TEXT NOT NULL,
    recorded_at              TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS contradiction_witnesses (
    contradiction_witness_id TEXT PRIMARY KEY,
    claim_id                 TEXT NOT NULL,
    conflicting_token_ids_json TEXT NOT NULL,
    summary                  TEXT,
    recorded_at              TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS retraction_records (
    retraction_record_id         TEXT PRIMARY KEY,
    claim_id                     TEXT NOT NULL,
    retracted_claim_version_id   TEXT NOT NULL,
    superseded_by_claim_version_id TEXT,
    effective_recorded_at        TEXT NOT NULL,
    reason                       TEXT NOT NULL,
    cascade_required             INTEGER NOT NULL DEFAULT 0,
    delta_summary                TEXT,
    recorded_at                  TEXT NOT NULL DEFAULT (datetime('now'))
);

ALTER TABLE claim_versions ADD COLUMN bilattice_truth TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE claim_versions ADD COLUMN support_set_id TEXT;
ALTER TABLE claim_versions ADD COLUMN support_set_digest TEXT;
ALTER TABLE claim_versions ADD COLUMN contradiction_witness_id TEXT;
ALTER TABLE claim_versions ADD COLUMN tx_from TEXT;
ALTER TABLE claim_versions ADD COLUMN tx_to TEXT;
ALTER TABLE claim_versions ADD COLUMN quality_vector_json TEXT;

UPDATE claim_versions
   SET tx_from = recorded_at
 WHERE tx_from IS NULL;

CREATE INDEX IF NOT EXISTS idx_claim_versions_tx_interval
    ON claim_versions(tx_from, tx_to);

CREATE INDEX IF NOT EXISTS idx_claim_versions_support_set
    ON claim_versions(support_set_id);

CREATE INDEX IF NOT EXISTS idx_claim_versions_contradiction_witness
    ON claim_versions(contradiction_witness_id);
