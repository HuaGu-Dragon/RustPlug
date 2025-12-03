use std::{os::windows::ffi::OsStrExt, path::PathBuf};

use clap::Parser;
use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryW};

#[derive(Parser)]
struct Cli {
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let dll_path: Vec<_> = cli.path.as_os_str().encode_wide().chain(Some(0)).collect();

    let handle = unsafe { LoadLibraryW(dll_path.as_ptr()) };

    if handle.is_null() {
        eprintln!(
            "Failed to load DLL, error: {:?}",
            std::io::Error::last_os_error()
        );
        return;
    }
    let addr = unsafe { GetProcAddress(handle, c"hello_world".as_ptr()) };

    if addr.is_null() {
        eprintln!(
            "Failed to find function, error: {:?}",
            std::io::Error::last_os_error()
        );
    }

    let hello_world: extern "C" fn() = unsafe { std::mem::transmute(addr) };
    hello_world();

    unsafe { FreeLibrary(handle) };
}
