//! CLI for llm-output-parser — reads raw LLM output on stdin, writes parsed result on stdout.
//!
//! Usage:
//!   llm-parse json     — extract JSON from raw LLM output
//!   llm-parse text     — clean text (strip think blocks, fences)
//!   llm-parse list     — extract string list
//!   llm-parse strip    — strip think blocks only, pass through rest
//!   llm-parse think-check — exit 0 if think blocks found, 1 if not

use std::io::{self, Read, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: llm-parse <command>");
        eprintln!("commands: json, text, list, strip, think-check");
        std::process::exit(1);
    }

    let command = &args[1];
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap_or_else(|e| {
        eprintln!("error reading stdin: {e}");
        std::process::exit(1);
    });

    match command.as_str() {
        "json" => match llm_output_parser::parse_json_value(&input) {
            Ok(value) => {
                let mut stdout = io::stdout();
                serde_json::to_writer_pretty(&mut stdout, &value).unwrap_or_default();
                stdout.write_all(b"\n").unwrap_or_default();
            }
            Err(e) => {
                eprintln!("parse error: {e}");
                std::process::exit(1);
            }
        },
        "text" => match llm_output_parser::parse_text(&input) {
            Ok(text) => print!("{text}"),
            Err(e) => {
                eprintln!("parse error: {e}");
                std::process::exit(1);
            }
        },
        "list" => match llm_output_parser::parse_string_list(&input) {
            Ok(items) => {
                let json = serde_json::to_string_pretty(&items).unwrap_or_default();
                println!("{json}");
            }
            Err(e) => {
                eprintln!("parse error: {e}");
                std::process::exit(1);
            }
        },
        "strip" => {
            let cleaned = llm_output_parser::strip_think_tags(&input);
            print!("{cleaned}");
        }
        "think-check" => {
            let has_think = input.contains("tidos") || input.contains("思考这个问题");
            std::process::exit(if has_think { 0 } else { 1 });
        }
        _ => {
            eprintln!("unknown command: {command}");
            eprintln!("commands: json, text, list, strip, think-check");
            std::process::exit(1);
        }
    }
}