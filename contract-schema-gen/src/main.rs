use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("--check") => {
            let dir = PathBuf::from(args.next().unwrap_or_else(|| "schemas".to_string()));
            contract_schema_gen::check_against_dir(&dir)
        }
        Some(path) => contract_schema_gen::generate_schemas(&PathBuf::from(path)).map(|_| ()),
        None => contract_schema_gen::generate_schemas(&PathBuf::from("schemas")).map(|_| ()),
    }
}
