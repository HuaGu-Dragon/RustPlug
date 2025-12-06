use std::{ffi::CString, path::PathBuf};

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
        let parsed_args = input.map(lexer).collect::<Vec<_>>();

        let args = parsed_args.iter().map(|arg| match arg {
            InputType::Interger(i) => (libffi::middle::Type::c_int(), libffi::middle::arg(i)),
            InputType::Text(_, ptr) => (libffi::middle::Type::pointer(), libffi::middle::arg(ptr)),
        });

        unsafe { manager.call_func::<()>(func, args, libffi::middle::Type::void())? };

        stdin.clear();
    }

    Ok(())
}

enum InputType {
    Interger(i32),
    Text(CString, *const i8),
}

fn lexer(input: &str) -> InputType {
    if input.starts_with('"') {
        let end = input.rfind('"').expect("");
        let s = CString::new(&input[1..end]).expect("");
        let ptr = s.as_ptr();
        InputType::Text(s, ptr)
    } else {
        InputType::Interger(input.parse().expect(""))
    }
}
