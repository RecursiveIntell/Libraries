mod common;

use common::*;
use poly_kv::*;

#[test]
fn memory_accounting_reader_attach_does_not_duplicate_encoded_pool_bytes() {
    let pool = build_pool(shape_mha());
    let encoded = pool.encoded_bytes();
    assert!(encoded > 0);
    assert_eq!(pool.reader_count(), 0);

    let reader_a = pool.attach_reader(ReaderConfig::default()).unwrap();
    let reader_b = pool.attach_reader(ReaderConfig::default()).unwrap();

    assert_eq!(pool.encoded_bytes(), encoded);
    assert_eq!(reader_a.injection_receipt().encoded_shared_bytes, encoded);
    assert_eq!(reader_b.injection_receipt().encoded_shared_bytes, encoded);
    assert_eq!(pool.reader_count(), 2);

    let memory = pool.memory_accounting();
    assert_eq!(memory.encoded_shared_bytes, encoded);
    assert_eq!(memory.reader_count, 2);
    assert_eq!(
        memory.per_reader_scratch_bytes,
        2 * ReaderConfig::default().scratch_bytes()
    );

    drop(reader_a);
    drop(reader_b);
    assert_eq!(pool.reader_count(), 0);
}

#[test]
fn memory_accounting_tracks_mixed_reader_scratch_budgets() {
    let pool = build_pool(shape_mha());
    let reader_a = pool
        .attach_reader(ReaderConfig {
            reader_label: Some("small".to_string()),
            scratch_budget_bytes: 1024,
        })
        .unwrap();
    let reader_b = pool
        .attach_reader(ReaderConfig {
            reader_label: Some("large".to_string()),
            scratch_budget_bytes: 4096,
        })
        .unwrap();

    let memory = pool.memory_accounting();
    assert_eq!(memory.reader_count, 2);
    assert_eq!(memory.per_reader_scratch_bytes, 5120);

    drop(reader_a);
    let memory = pool.memory_accounting();
    assert_eq!(memory.reader_count, 1);
    assert_eq!(memory.per_reader_scratch_bytes, 4096);

    drop(reader_b);
    let memory = pool.memory_accounting();
    assert_eq!(memory.reader_count, 0);
    assert_eq!(memory.per_reader_scratch_bytes, 0);
}
