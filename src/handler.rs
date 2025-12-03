use std::{os::windows::ffi::OsStrExt, path::Path, ptr::NonNull};

use anyhow::Context;
use winapi::{
    shared::minwindef::HINSTANCE__,
    um::libloaderapi::{FreeLibrary, LoadLibraryW},
};

pub struct DllManager {
    pub handle: NonNull<HINSTANCE__>,
}

impl DllManager {
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            anyhow::bail!("file did not exit");
        }

        let dll_path: Vec<_> = path.as_os_str().encode_wide().chain(Some(0)).collect();

        let handle = unsafe { LoadLibraryW(dll_path.as_ptr()) };
        let handle = NonNull::new(handle).with_context(|| {
            format!(
                "Failed to load DLL, error: {:?}",
                std::io::Error::last_os_error()
            )
        })?;

        Ok(Self { handle })
    }
}

impl Drop for DllManager {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.handle.as_ptr()) };
    }
}
