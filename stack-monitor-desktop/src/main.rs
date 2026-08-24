fn main() {
    let database = std::env::var_os("ARES_OBSERVATORY_DB")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("activity.db"));
    if let Err(error) = stack_monitor_desktop_lib::run(database) {
        eprintln!("Ares Observatory failed: {error}");
        std::process::exit(1);
    }
}
