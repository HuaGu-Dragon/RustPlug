use std::path::PathBuf;

use clap::Parser;
use rust_plug::handler::DllManager;

#[derive(Parser)]
struct Cli {
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let manager = DllManager::new(cli.path)?;

    let hello_world = unsafe { manager.get_func("hello_world") }?;

    hello_world();

    Ok(())
}
