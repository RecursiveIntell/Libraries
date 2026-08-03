use context_governor::{evaluate_replay_fixture, CompactRequest};
use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("read request JSON from stdin");
    let request: CompactRequest = serde_json::from_str(&input).expect("parse CompactRequest JSON");
    let fixture_id = request.session_id.clone();
    let report = evaluate_replay_fixture(fixture_id, request, 32).expect("evaluate replay fixture");
    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("serialize report")
    );
}
