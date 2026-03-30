use serde_json::Value;

#[test]
fn v25_fixture_manifest_matches_bundle_files() {
    let manifest_body = std::fs::read_to_string("../contracts/fixtures/v25/manifest.json")
        .expect("read fixture manifest");
    let manifest: Value = serde_json::from_str(&manifest_body).expect("parse fixture manifest");
    let fixtures = manifest["fixtures"].as_array().expect("fixtures array");

    for fixture in fixtures {
        let file = fixture["file"].as_str().expect("fixture file");
        let required = fixture["required_artifacts"]
            .as_array()
            .expect("required artifacts");
        let path = format!("../contracts/fixtures/v25/{file}");
        let body = std::fs::read_to_string(&path).expect("read fixture file");
        let bundle: Value = serde_json::from_str(&body).expect("parse fixture bundle");
        let artifacts = bundle["artifacts"].as_object().expect("artifacts object");
        for key in required {
            let key = key.as_str().expect("required artifact key");
            assert!(
                artifacts.contains_key(key),
                "{file} missing required artifact {key}"
            );
        }
    }
}
