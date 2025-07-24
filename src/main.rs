mod cli;
use cli::Cli;
use clap::Parser;
use env_logger;
use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    info!("Starting cargo-scrub with args: {:?}", cli);
    // TODO: Call library logic here
    println!("cargo-scrub: CLI parsed, core logic not yet implemented");
    Ok(())
}
