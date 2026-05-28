#![cfg_attr(test, allow(clippy::expect_used))]

mod main_support;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(error) = main_support::run().await {
        eprintln!("forge-pilot: {error}");
        std::process::exit(1);
    }
}
