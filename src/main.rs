use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryW};

fn main() {
    let dll_path = "hello_world.dll";

    let path: Vec<_> = OsStr::new(dll_path).encode_wide().chain(Some(0)).collect();
    let handle = unsafe { LoadLibraryW(path.as_ptr()) };

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

    let hello_world: fn() = unsafe { std::mem::transmute(addr) };
    hello_world();

    unsafe { FreeLibrary(handle) };
}
