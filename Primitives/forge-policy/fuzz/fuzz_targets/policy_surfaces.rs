#![no_main]

use std::path::Path;

use forge_policy::{ensure_relative_path, is_env_allowed, resolve_workspace_path, validate_forbidden_paths};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let candidate = input.chars().take(64).collect::<String>();
    let root = tempfile::tempdir().unwrap();

    let _ = ensure_relative_path(Path::new(&candidate));
    let _ = resolve_workspace_path(root.path(), Path::new(&candidate));
    let _ = is_env_allowed(&candidate);
    let _ = validate_forbidden_paths(
        &[candidate.clone()],
        &[String::from("target/**"), String::from("**/*.snap")],
        true,
    );
});
