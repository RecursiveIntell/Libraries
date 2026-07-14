use serde_json::{json, Value};

#[test]
fn canonical_json_digests_match_boundary_compiler_for_shared_golden_corpus() {
    let cases: [Value; 24] = [
        json!(null),
        json!(true),
        json!(false),
        json!(0),
        json!(-0.0),
        json!(1),
        json!(-1),
        json!(0.000001),
        json!(1e30),
        json!(""),
        json!("plain"),
        json!("quote\"slash\\"),
        json!("\u{0085}"),
        json!([]),
        json!([1, 2, 3]),
        json!([{"b": 2, "a": 1}]),
        json!({}),
        json!({"b": 2, "a": 1}),
        json!({"nested": {"z": null, "a": true}}),
        json!({"array": [{"z": 1, "a": 2}, false]}),
        json!({"\u{e000}": 1, "\u{10000}": 2}),
        json!({"unicode": "€😀דּ"}),
        json!({"numbers": [333333333.33333329, 4.50, 2e-3, 1e-27]}),
        json!({"literals": [null, true, false], "string": "€$\u{000f}\nA'B\"\\\\\"/"}),
    ];

    for value in cases {
        let boundary = boundary_compiler::ContentDigest::compute(&value).unwrap();
        let stack = stack_ids::ContentDigest::compute_json(&value).unwrap();
        assert_eq!(stack.as_bytes(), boundary.as_bytes(), "value: {value}");
        assert_eq!(stack.metadata().algorithm, boundary.metadata().algorithm);
        assert_eq!(
            stack.metadata().canonicalization_profile,
            boundary.metadata().canonicalization_profile
        );
        assert_eq!(stack.metadata().schema_id, boundary.metadata().schema_id);
        assert_eq!(
            stack.metadata().schema_version,
            boundary.metadata().schema_version
        );
        assert_eq!(
            stack.metadata().domain_separator,
            boundary.metadata().domain_separator
        );
    }
}
