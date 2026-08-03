#![allow(deprecated)]

use std::io::{self, Read};

use forge_memory_bridge::transform_envelope_v2;
use semantic_memory_forge::ExportEnvelopeV2;

fn usage() {
    eprintln!("usage: forge-memory-bridge transform --input-type v2");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 || args[1] != "transform" || args[2] != "--input-type" {
        usage();
        std::process::exit(2);
    }
    if args[3] != "v2" {
        eprintln!("unsupported input type: {}", args[3]);
        std::process::exit(2);
    }

    let mut input = String::new();
    if let Err(error) = io::stdin().read_to_string(&mut input) {
        eprintln!("failed to read stdin: {error}");
        std::process::exit(1);
    }
    let envelope: ExportEnvelopeV2 = match serde_json::from_str(&input) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid forge export JSON: {error}");
            std::process::exit(1);
        }
    };
    let batch = match transform_envelope_v2(&envelope) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("forge export transform failed: {error}");
            std::process::exit(1);
        }
    };
    match serde_json::to_writer(io::stdout(), &batch) {
        Ok(()) => println!(),
        Err(error) => {
            eprintln!("failed to write import batch JSON: {error}");
            std::process::exit(1);
        }
    }
}
