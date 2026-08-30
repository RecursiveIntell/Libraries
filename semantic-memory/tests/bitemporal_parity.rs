//! TRUTH-001: Bitemporal reference parity suite.
//!
//! Locks named-fixture parity between the in-memory reference semantics
//! (`bitemporal-runtime`: append-supersede over a Vec, `as_of_query`
//! recorded-time-max-wins) and the SQLite temporal projection
//! (`semantic-memory` episodes: version rows chained by
//! `superseded_by` -> `fact_digest`, queried by `episode_as_of`).
//!
//! Storage-time ownership remains SQLite-only: the reference store is used
//! strictly as an in-memory oracle; no durable data passes through it.
//!
//! Claim boundary (per TRUTH-001): named fixture parity is verified, not all
//! temporal workloads.
#![allow(clippy::expect_used)]

use bitemporal_runtime::{append_supersede, as_of_query, BitemporalRecord};
use chrono::{DateTime, TimeZone, Utc};
use semantic_memory::{
    EpisodeAsOfReceiptV1, EpisodeMeta, EpisodeOutcome, MemoryConfig, MemoryStore, MockEmbedder,
    VerificationStatus,
};
use std::collections::BTreeMap;
use tempfile::TempDir;

// ─── Fixed timeline ────────────────────────────────────────────────────────
// Deterministic timestamps; EPS sits strictly between T0 and T1 to probe
// "before anything" cutoffs without colliding with T0.
const T0: i64 = 1_000;
const T1: i64 = 2_000;
const T2: i64 = 3_000;
const T3: i64 = 4_000;
const EPS: i64 = 500; // strictly BEFORE T0: "before anything" cutoffs
const MID: i64 = 1_500; // strictly between T0 and T1: between-recorded cutoffs

fn t(secs: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(secs, 0).unwrap()
}

// ─── Fixture model ─────────────────────────────────────────────────────────

struct Version {
    valid: i64,
    recorded: i64,
    value: &'static str,
}

struct Family {
    /// Record identity (reference `id` / SQLite document_id).
    id: &'static str,
    /// Appended in order; value is stored as the episode `effect_type`.
    versions: Vec<Version>,
}

struct Probe {
    valid: i64,
    recorded: i64,
    /// Expected winner value per family id (reference-derived, hand-pinned).
    /// Absent families must be absent in BOTH lanes.
    expected: &'static [(&'static str, &'static str)],
}

/// F1 basic supersede: same valid time, later recorded time.
fn f1() -> Family {
    Family {
        id: "doc-f1",
        versions: vec![
            Version {
                valid: T0,
                recorded: T0,
                value: "alpha",
            },
            Version {
                valid: T0,
                recorded: T1,
                value: "beta",
            },
        ],
    }
}

/// F2 valid-time progression: one version per valid time.
fn f2() -> Family {
    Family {
        id: "doc-f2",
        versions: vec![
            Version {
                valid: T0,
                recorded: T0,
                value: "a",
            },
            Version {
                valid: T1,
                recorded: T1,
                value: "b",
            },
            Version {
                valid: T2,
                recorded: T2,
                value: "c",
            },
        ],
    }
}

/// F3 valid-time reversal (retroactive correction):
/// the new version is valid EARLIER than the version it supersedes.
fn f3() -> Family {
    Family {
        id: "doc-f3",
        versions: vec![
            Version {
                valid: T2,
                recorded: T0,
                value: "x",
            },
            Version {
                valid: T0,
                recorded: T1,
                value: "y",
            },
        ],
    }
}

/// F4 recorded-time reversal (backdated insert):
/// the last append carries an earlier recorded_time than the prior append.
fn f4() -> Family {
    Family {
        id: "doc-f4",
        versions: vec![
            Version {
                valid: T0,
                recorded: T0,
                value: "a",
            },
            Version {
                valid: T1,
                recorded: T2,
                value: "b",
            },
            Version {
                valid: T1,
                recorded: T1,
                value: "c",
            },
        ],
    }
}

/// F5 supersession chain: three versions, strictly increasing times.
fn f5() -> Family {
    Family {
        id: "doc-f5",
        versions: vec![
            Version {
                valid: T0,
                recorded: T0,
                value: "a",
            },
            Version {
                valid: T1,
                recorded: T1,
                value: "b",
            },
            Version {
                valid: T2,
                recorded: T2,
                value: "c",
            },
        ],
    }
}

/// F6 multi-id interleave: two independent families written alternately.
fn f6() -> (Family, Family) {
    (
        Family {
            id: "doc-f6a",
            versions: vec![
                Version {
                    valid: T0,
                    recorded: T0,
                    value: "a1",
                },
                Version {
                    valid: T1,
                    recorded: T2,
                    value: "a2",
                },
            ],
        },
        Family {
            id: "doc-f6b",
            versions: vec![Version {
                valid: T1,
                recorded: T1,
                value: "b1",
            }],
        },
    )
}

/// F7 recorded-time tie: two versions with identical recorded_time.
/// Reference tie rule: first-inserted wins ("b").
fn f7() -> Family {
    Family {
        id: "doc-f7",
        versions: vec![
            Version {
                valid: T0,
                recorded: T0,
                value: "a",
            },
            Version {
                valid: T1,
                recorded: T1,
                value: "b",
            },
            Version {
                valid: T1,
                recorded: T1,
                value: "c",
            },
        ],
    }
}

/// F8 branching supersession: two versions supersede the same predecessor.
/// Both branch versions share recorded_time — also exercises the tie rule.
fn f8() -> Family {
    Family {
        id: "doc-f8",
        versions: vec![
            Version {
                valid: T0,
                recorded: T0,
                value: "root",
            },
            Version {
                valid: T1,
                recorded: T1,
                value: "left",
            },
            Version {
                valid: T1,
                recorded: T1,
                value: "right",
            },
        ],
    }
}

/// F13 three-way recorded-time tie with explicit insertion order.
fn f13() -> Family {
    Family {
        id: "doc-f13",
        versions: vec![
            Version {
                valid: T0,
                recorded: T0,
                value: "a",
            },
            Version {
                valid: T1,
                recorded: T1,
                value: "b",
            },
            Version {
                valid: T1,
                recorded: T1,
                value: "c",
            },
            Version {
                valid: T1,
                recorded: T1,
                value: "d",
            },
        ],
    }
}

// ─── Lanes ─────────────────────────────────────────────────────────────────

/// Reference lane: in-memory Vec + bitemporal-runtime.
fn reference_winners(families: &[&Family], probe: &Probe) -> BTreeMap<String, String> {
    let mut records: Vec<BitemporalRecord<String>> = Vec::new();
    for family in families {
        for v in &family.versions {
            let record = BitemporalRecord {
                id: family.id.to_string(),
                valid_time: t(v.valid),
                recorded_time: t(v.recorded),
                value: v.value.to_string(),
            };
            append_supersede(&mut records, record).expect("append");
        }
    }
    as_of_query(&records, t(probe.valid), t(probe.recorded))
        .into_iter()
        .map(|r| (r.id, r.value))
        .collect()
}

/// SQLite lane: episodes written through the canonical append path, queried
/// through `episode_as_of`. `doc_map` translates SQLite document ids back to
/// fixture family ids. Returns (family_id, effect_type) winners.
async fn sqlite_winners(
    store: &MemoryStore,
    probe: &Probe,
    doc_map: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let (rows, _receipt) = store
        .episode_as_of(t(probe.valid), t(probe.recorded))
        .await
        .expect("episode_as_of");
    let row_count = rows.len();
    let winners: BTreeMap<String, String> = rows
        .into_iter()
        .map(|row| {
            let family = doc_map
                .get(&row.document_id)
                .expect("winner document id must map to a fixture family")
                .clone();
            (family, row.meta.effect_type)
        })
        .collect();
    assert_eq!(
        winners.len(),
        row_count,
        "episode_as_of returned multiple winners for one fixture family"
    );
    winners
}

/// Seed the SQLite lane: one document per family, one `append_episode_version`
/// per version with explicit recorded_time. Returns SQLite doc id -> family id.
async fn seed_sqlite(store: &MemoryStore, families: &[&Family]) -> BTreeMap<String, String> {
    let mut doc_map = BTreeMap::new();
    for family in families {
        let doc_id = store
            .ingest_document(
                &format!("{} title", family.id),
                &format!("content for {}", family.id),
                "general",
                None,
                None,
            )
            .await
            .expect("ingest doc");
        doc_map.insert(doc_id.clone(), family.id.to_string());
        let mut predecessor_id: Option<String> = None;
        for (idx, v) in family.versions.iter().enumerate() {
            let episode_id = format!("{}-v{idx}", family.id);
            let meta = EpisodeMeta {
                cause_ids: vec![format!("cause-{idx}")],
                effect_type: v.value.to_string(),
                outcome: EpisodeOutcome::Pending,
                confidence: 0.5,
                verification_status: VerificationStatus::Unverified,
                experiment_id: None,
                valid_time: Some(t(v.valid)),
                fact_digest: None,
            };
            store
                .append_episode_version(
                    &episode_id,
                    predecessor_id.as_deref(),
                    &doc_id,
                    &meta,
                    Some(t(v.recorded)),
                )
                .await
                .expect("append episode version");
            predecessor_id = Some(episode_id);
        }
    }
    doc_map
}

fn test_store() -> (MemoryStore, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let config = MemoryConfig {
        base_dir: dir.path().to_path_buf(),
        ..Default::default()
    };
    let embedder = Box::new(MockEmbedder::new(config.embedding.dimensions));
    let store = MemoryStore::open_with_embedder(config, embedder).expect("open store");
    (store, dir)
}

// ─── Parity assertions ─────────────────────────────────────────────────────

async fn assert_parity(families: &[&Family], probes: &[Probe]) {
    let (store, _dir) = test_store();
    let doc_map = seed_sqlite(&store, families).await;

    for probe in probes {
        let reference = reference_winners(families, probe);
        let sqlite = sqlite_winners(&store, probe, &doc_map).await;

        // Both lanes must match the hand-pinned expectation exactly.
        let mut expected: BTreeMap<String, String> = BTreeMap::new();
        for (id, value) in probe.expected {
            expected.insert(id.to_string(), value.to_string());
        }
        assert_eq!(
            reference, expected,
            "reference lane diverges from pinned expectation at valid={} recorded={}",
            probe.valid, probe.recorded
        );
        assert_eq!(
            sqlite, expected,
            "SQLite lane diverges from pinned expectation at valid={} recorded={}",
            probe.valid, probe.recorded
        );
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn parity_f1_basic_supersede() {
    let f = f1();
    assert_parity(
        &[&f],
        &[
            // Before anything: absent.
            Probe {
                valid: EPS,
                recorded: EPS,
                expected: &[],
            },
            // Recorded before supersession: alpha.
            Probe {
                valid: T0,
                recorded: T0,
                expected: &[("doc-f1", "alpha")],
            },
            Probe {
                valid: T0,
                recorded: EPS,
                expected: &[],
            },
            // After supersession: beta.
            Probe {
                valid: T0,
                recorded: T1,
                expected: &[("doc-f1", "beta")],
            },
        ],
    )
    .await;
}

#[tokio::test]
async fn parity_f2_valid_progression() {
    let f = f2();
    assert_parity(
        &[&f],
        &[
            Probe {
                valid: EPS,
                recorded: EPS,
                expected: &[],
            },
            Probe {
                valid: T0,
                recorded: T0,
                expected: &[("doc-f2", "a")],
            },
            Probe {
                valid: T1,
                recorded: T1,
                expected: &[("doc-f2", "b")],
            },
            Probe {
                valid: T2,
                recorded: T2,
                expected: &[("doc-f2", "c")],
            },
            // Recorded before a version was known: old winner persists.
            Probe {
                valid: T2,
                recorded: T1,
                expected: &[("doc-f2", "b")],
            },
            // Valid cutoff before a version's valid time: that version invisible.
            Probe {
                valid: T1,
                recorded: T2,
                expected: &[("doc-f2", "b")],
            },
        ],
    )
    .await;
}

#[tokio::test]
async fn parity_f3_valid_time_reversal() {
    let f = f3();
    assert_parity(
        &[&f],
        &[
            // Before correction recorded: original valid-only at T2.
            Probe {
                valid: T0,
                recorded: T0,
                expected: &[],
            },
            Probe {
                valid: T2,
                recorded: T0,
                expected: &[("doc-f3", "x")],
            },
            // Correction recorded: retroactively wins for ALL valid >= its valid_time.
            Probe {
                valid: T0,
                recorded: T1,
                expected: &[("doc-f3", "y")],
            },
            Probe {
                valid: T2,
                recorded: T1,
                expected: &[("doc-f3", "y")],
            },
        ],
    )
    .await;
}

#[tokio::test]
async fn parity_f4_recorded_time_reversal() {
    let f = f4();
    assert_parity(
        &[&f],
        &[
            Probe {
                valid: T0,
                recorded: T1,
                expected: &[("doc-f4", "a")],
            },
            // Backdated insert (c, recorded T1) beats a (recorded T0) at T1...
            Probe {
                valid: T1,
                recorded: T1,
                expected: &[("doc-f4", "c")],
            },
            // ...but b (recorded T2) is current once its recorded time arrives.
            Probe {
                valid: T1,
                recorded: T2,
                expected: &[("doc-f4", "b")],
            },
            // Recorded cutoff between c and b: a still wins.
            Probe {
                valid: T1,
                recorded: MID,
                expected: &[("doc-f4", "a")],
            },
        ],
    )
    .await;
}

#[tokio::test]
async fn parity_f5_chain() {
    let f = f5();
    assert_parity(
        &[&f],
        &[
            Probe {
                valid: T0,
                recorded: T0,
                expected: &[("doc-f5", "a")],
            },
            Probe {
                valid: T1,
                recorded: T1,
                expected: &[("doc-f5", "b")],
            },
            Probe {
                valid: T2,
                recorded: T2,
                expected: &[("doc-f5", "c")],
            },
            // Historic recorded cutoffs still return the then-current version.
            Probe {
                valid: T2,
                recorded: T0,
                expected: &[("doc-f5", "a")],
            },
            Probe {
                valid: T2,
                recorded: T1,
                expected: &[("doc-f5", "b")],
            },
        ],
    )
    .await;
}

#[tokio::test]
async fn parity_f6_multi_id_interleave() {
    let (fa, fb) = f6();
    assert_parity(
        &[&fa, &fb],
        &[
            Probe {
                valid: T0,
                recorded: T0,
                expected: &[("doc-f6a", "a1")],
            },
            Probe {
                valid: T1,
                recorded: T1,
                expected: &[("doc-f6a", "a1"), ("doc-f6b", "b1")],
            },
            Probe {
                valid: T1,
                recorded: T2,
                expected: &[("doc-f6a", "a2"), ("doc-f6b", "b1")],
            },
        ],
    )
    .await;
}

#[tokio::test]
async fn parity_f7_recorded_time_tie_first_inserted_wins() {
    let f = f7();
    assert_parity(
        &[&f],
        &[
            Probe {
                valid: T0,
                recorded: T1,
                expected: &[("doc-f7", "a")],
            },
            // Tie between "b" and "c" at recorded T1: reference keeps first-inserted "b".
            Probe {
                valid: T1,
                recorded: T1,
                expected: &[("doc-f7", "b")],
            },
        ],
    )
    .await;
}

#[tokio::test]
async fn parity_f8_branching_supersession() {
    let f = f8();
    assert_parity(
        &[&f],
        &[
            Probe {
                valid: T0,
                recorded: T0,
                expected: &[("doc-f8", "root")],
            },
            // Both branches eligible at T1/T1: recorded-time tie, first-inserted "left".
            Probe {
                valid: T1,
                recorded: T1,
                expected: &[("doc-f8", "left")],
            },
            // Later recorded cutoff still picks the first-inserted branch winner.
            Probe {
                valid: T1,
                recorded: T2,
                expected: &[("doc-f8", "left")],
            },
            // Valid cutoff before the branches were valid: root remains.
            Probe {
                valid: T0,
                recorded: T2,
                expected: &[("doc-f8", "root")],
            },
        ],
    )
    .await;
}

#[tokio::test]
async fn parity_f13_three_way_tie() {
    let f = f13();
    assert_parity(
        &[&f],
        &[
            Probe {
                valid: T0,
                recorded: T1,
                expected: &[("doc-f13", "a")],
            },
            // Three-way tie at recorded T1: first-inserted "b" wins.
            Probe {
                valid: T1,
                recorded: T1,
                expected: &[("doc-f13", "b")],
            },
        ],
    )
    .await;
}

/// Empty history: no versions means an empty winner set in BOTH lanes.
#[tokio::test]
async fn parity_empty_history() {
    assert_parity(
        &[],
        &[
            Probe {
                valid: T0,
                recorded: T0,
                expected: &[],
            },
            Probe {
                valid: T3,
                recorded: T3,
                expected: &[],
            },
        ],
    )
    .await;
}

/// The as-of receipt must report the same winner set and a truthful
/// excluded-superseded count on a supersession edge.
#[tokio::test]
async fn receipt_reports_winners_and_excluded_superseded() {
    let (store, _dir) = test_store();
    let f = f5();
    let doc_map = seed_sqlite(&store, &[&f]).await;

    let (rows, receipt) = store
        .episode_as_of(t(T2), t(T2))
        .await
        .expect("episode_as_of");
    assert_eq!(rows.len(), 1);
    assert_eq!(doc_map.get(&rows[0].document_id).unwrap(), "doc-f5");
    assert_eq!(rows[0].meta.effect_type, "c");
    assert_eq!(receipt.as_of_valid, t(T2));
    assert_eq!(receipt.as_of_recorded, t(T2));
    assert_eq!(receipt.episode_count, 1);
    assert_eq!(receipt.excluded_superseded, 2);
    assert!(!receipt.query_id.is_empty());
    assert_eq!(receipt.episode_ids, vec![rows[0].episode_id.clone()]);

    let (rows, receipt) = store
        .episode_as_of(t(T2), t(T1))
        .await
        .expect("episode_as_of");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].meta.effect_type, "b");
    assert_eq!(receipt.excluded_superseded, 1);

    // Repeated calls must be deterministic and side-effect free.
    let (rows_again, receipt_again) = store
        .episode_as_of(t(T2), t(T1))
        .await
        .expect("episode_as_of repeat");
    assert_eq!(rows_again, rows);
    assert_eq!(receipt_again.episode_ids, receipt.episode_ids);
    assert_eq!(
        receipt_again.excluded_superseded,
        receipt.excluded_superseded
    );
    assert_eq!(receipt_again.episode_count, receipt.episode_count);

    let _: EpisodeAsOfReceiptV1 = receipt;
}

/// Sibling episodes may share one document but must remain distinct version
/// families. Linking by latest document row would collapse these two chains.
#[tokio::test]
async fn sibling_episode_families_do_not_merge() {
    let (store, _dir) = test_store();
    let doc_id = store
        .ingest_document(
            "siblings",
            "independent sibling families",
            "general",
            None,
            None,
        )
        .await
        .expect("ingest doc");

    let meta = |effect: &str, valid: i64| EpisodeMeta {
        cause_ids: vec![format!("cause-{effect}")],
        effect_type: effect.to_string(),
        outcome: EpisodeOutcome::Pending,
        confidence: 0.5,
        verification_status: VerificationStatus::Unverified,
        experiment_id: None,
        valid_time: Some(t(valid)),
        fact_digest: None,
    };

    store
        .append_episode_version("a0", None, &doc_id, &meta("a0", T0), Some(t(T0)))
        .await
        .expect("append a root");
    store
        .append_episode_version("b0", None, &doc_id, &meta("b0", T0), Some(t(T0)))
        .await
        .expect("append b root");
    store
        .append_episode_version("a1", Some("a0"), &doc_id, &meta("a1", T1), Some(t(T1)))
        .await
        .expect("append a successor");
    store
        .append_episode_version("b1", Some("b0"), &doc_id, &meta("b1", T1), Some(t(T2)))
        .await
        .expect("append b successor");

    let (at_t1, receipt_t1) = store.episode_as_of(t(T1), t(T1)).await.expect("as of t1");
    let t1_winners: BTreeMap<_, _> = at_t1
        .iter()
        .map(|row| (row.episode_id.as_str(), row.meta.effect_type.as_str()))
        .collect();
    assert_eq!(t1_winners, BTreeMap::from([("a1", "a1"), ("b0", "b0")]));
    assert_eq!(receipt_t1.episode_count, 2);
    assert_eq!(receipt_t1.excluded_superseded, 1);

    let (at_t2, receipt_t2) = store.episode_as_of(t(T1), t(T2)).await.expect("as of t2");
    let t2_winners: BTreeMap<_, _> = at_t2
        .iter()
        .map(|row| (row.episode_id.as_str(), row.meta.effect_type.as_str()))
        .collect();
    assert_eq!(t2_winners, BTreeMap::from([("a1", "a1"), ("b1", "b1")]));
    assert_eq!(receipt_t2.episode_count, 2);
    assert_eq!(receipt_t2.excluded_superseded, 2);
}
