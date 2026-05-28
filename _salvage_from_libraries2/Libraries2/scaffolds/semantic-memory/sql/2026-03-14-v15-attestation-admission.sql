-- Draft additive v15 persistence scaffold.

CREATE TABLE IF NOT EXISTS attestation_envelopes (
    attestation_envelope_id   TEXT PRIMARY KEY,
    artifact_family           TEXT NOT NULL,
    content_digest            TEXT NOT NULL,
    trust_root_set_id         TEXT NOT NULL,
    disclosure_policy_id      TEXT,
    artifact_json             TEXT NOT NULL,
    recorded_at               TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS remote_slice_requests (
    remote_slice_request_id   TEXT PRIMARY KEY,
    remote_oracle_lease_id    TEXT NOT NULL,
    attestation_envelope_id   TEXT,
    artifact_json             TEXT NOT NULL,
    recorded_at               TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS remote_slice_results (
    remote_slice_result_id    TEXT PRIMARY KEY,
    remote_slice_request_id   TEXT NOT NULL,
    attestation_envelope_id   TEXT NOT NULL,
    replay_ticket_id          TEXT,
    dispute_bundle_id         TEXT,
    artifact_json             TEXT NOT NULL,
    recorded_at               TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS dispute_bundles (
    dispute_bundle_id         TEXT PRIMARY KEY,
    artifact_json             TEXT NOT NULL,
    recorded_at               TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_v15_request_lease ON remote_slice_requests(remote_oracle_lease_id);
CREATE INDEX IF NOT EXISTS idx_v15_result_request ON remote_slice_results(remote_slice_request_id);
CREATE INDEX IF NOT EXISTS idx_v15_result_dispute ON remote_slice_results(dispute_bundle_id);
