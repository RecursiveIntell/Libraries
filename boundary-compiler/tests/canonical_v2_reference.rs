use boundary_compiler::canonicalize_json_v2;
use serde_json::Value;
use sha2::{Digest, Sha256};

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn cyberphone_reference_vectors_match_byte_for_byte() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("manifest_dir={}", env!("CARGO_MANIFEST_DIR"));
    for name in [
        "arrays.json",
        "french.json",
        "structures.json",
        "unicode.json",
        "values.json",
        "weird.json",
    ] {
        let input_path = format!(
            "{}/tests/fixtures/rfc8785/input/{name}",
            env!("CARGO_MANIFEST_DIR")
        );
        eprintln!(
            "input_path={input_path} exists={}",
            std::path::Path::new(&input_path).exists()
        );
        let input = std::fs::read(input_path)?;
        let expected = std::fs::read(format!(
            "{}/tests/fixtures/rfc8785/output/{name}",
            env!("CARGO_MANIFEST_DIR")
        ))?;
        let actual = canonicalize_json_v2(&input)?;
        assert_eq!(actual.as_bytes(), expected.as_slice(), "fixture {name}");
    }
    Ok(())
}

#[test]
fn pinned_corpus_manifest_matches_fixture_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let manifest: Value = serde_json::from_slice(&std::fs::read(format!(
        "{}/tests/fixtures/rfc8785/manifest.json",
        env!("CARGO_MANIFEST_DIR")
    ))?)?;
    assert_eq!(
        manifest["source"]["commit"],
        "19d51d7fe467d4706a3ff08adf8a748f29fc21e0"
    );
    for kind in ["input", "output"] {
        let entries = manifest["files"][kind]
            .as_object()
            .ok_or("manifest entries must be an object")?;
        for (name, entry) in entries {
            let path = format!(
                "{}/tests/fixtures/rfc8785/{kind}/{name}",
                env!("CARGO_MANIFEST_DIR")
            );
            let bytes = std::fs::read(path)?;
            assert_eq!(
                entry["bytes"].as_u64(),
                Some(bytes.len() as u64),
                "{kind}/{name}"
            );
            assert_eq!(entry["sha256"], sha256_hex(&bytes), "{kind}/{name}");
        }
    }
    Ok(())
}
