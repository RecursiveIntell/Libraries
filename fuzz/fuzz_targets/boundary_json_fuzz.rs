#![no_main]

use libfuzzer_sys::fuzz_target;
use boundary_compiler::parse_with_dup_check;

fuzz_target!(|data: &[u8]| {
    let text = std::str::from_utf8_lossy(data);
    let _ = parse_with_dup_check(&text);
});

