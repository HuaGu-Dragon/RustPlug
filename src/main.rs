use std::path::PathBuf;

use clap::Parser;
use winapi::um::libloaderapi::GetProcAddress;

use crate::handler::DllManager;

mod handler;

#[derive(Parser)]
struct Cli {
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let manager = DllManager::new(cli.path)?;

    let addr = unsafe { GetProcAddress(manager.handle.as_ptr(), c"hello_world".as_ptr()) };

    if addr.is_null() {
        eprintln!(
            "Failed to find function, error: {:?}",
            std::io::Error::last_os_error()
        );
    }

    let hello_world: extern "C" fn() = unsafe { std::mem::transmute(addr) };
    hello_world();

    Ok(())
}
