#![no_main]

use std::path::PathBuf;

use libfuzzer_sys::fuzz_target;
use sandbox_workspace::{LocalPatchFs, PatchFs};

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let root = tempfile::tempdir().unwrap();
    let fs = LocalPatchFs::new(root.path());
    let relative = PathBuf::from(input.chars().take(48).collect::<String>());
    let lines = input
        .lines()
        .take(8)
        .map(|line| line.chars().take(64).collect::<String>())
        .collect::<Vec<_>>();

    let _ = fs.create_parent_dirs(&relative);
    let _ = fs.write_lines(&relative, &lines);
    let _ = fs.exists(&relative);
    let _ = fs.read_lines(&relative);
    let _ = fs.snapshot_lines(&relative);
    let _ = fs.remove_file(&relative);
});
