use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use rust_plug::handler::DllManager;

#[derive(Parser)]
struct Cli {
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let manager = DllManager::new(cli.path)?;

    let mut stdin = String::new();
    loop {
        std::io::stdin()
            .read_line(&mut stdin)
            .context("read input from stdin")?;

        let input = stdin.trim();
        if input == ":q" {
            break;
        }

        let mut input = input.split_whitespace();

        let func = input.next().context("")?;
        let parsed_args = input
            .map(|arg| arg.parse::<i32>().unwrap())
            .collect::<Vec<_>>();

        let args = parsed_args
            .iter()
            .map(|arg| (libffi::middle::Type::i32(), libffi::middle::arg(arg)));

        unsafe { manager.call_func::<()>(func, args, libffi::middle::Type::void())? };

        stdin.clear();
    }

    Ok(())
}
