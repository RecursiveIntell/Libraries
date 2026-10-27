use constitutive_witness::{parse_request, solve, MAX_BYTES};
use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    if io::stdin()
        .take((MAX_BYTES + 1) as u64)
        .read_to_string(&mut input)
        .is_err()
    {
        eprintln!("invalid input encoding or read failure");
        std::process::exit(2);
    }
    match parse_request(&input).and_then(|(problem, budget)| solve(&problem, budget)) {
        Ok(result) => println!("{}", result.to_json()),
        Err(error) => {
            eprintln!("constitutive-witness rejected request: {error}");
            std::process::exit(2);
        }
    }
}
