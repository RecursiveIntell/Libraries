CREATE TABLE IF NOT EXISTS continuity_artifacts_v1 (
    artifact_id TEXT PRIMARY KEY,
    artifact_family TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    valid_time_start TEXT,
    valid_time_end TEXT,
    recorded_at TEXT NOT NULL,
    owner_ref TEXT,
    advisory_state TEXT NOT NULL,
    backpointer_refs_json TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_continuity_artifacts_v1_family
    ON continuity_artifacts_v1(artifact_family);

CREATE INDEX IF NOT EXISTS idx_continuity_artifacts_v1_recorded_at
    ON continuity_artifacts_v1(recorded_at);
