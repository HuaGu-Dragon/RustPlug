use std::{ffi::CString, os::windows::ffi::OsStrExt, path::Path, ptr::NonNull};

use anyhow::Context;
use winapi::{
    shared::minwindef::HINSTANCE__,
    um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryW},
};

pub struct DllManager {
    handle: NonNull<HINSTANCE__>,
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

    /// # Safety
    ///
    /// We ensure that the function pointer is valid and can be safely transmuted.
    pub unsafe fn get_func(&self, name: impl AsRef<str>) -> anyhow::Result<extern "C" fn()> {
        let func = unsafe {
            GetProcAddress(
                self.handle.as_ptr(),
                CString::new(name.as_ref())
                    .context("Convert Rust str to Cstr")?
                    .as_ptr(),
            )
        };

        if func.is_null() {
            anyhow::bail!(
                "Failed to find function, error: {:?}",
                std::io::Error::last_os_error()
            )
        }

        Ok(unsafe {
            std::mem::transmute::<*mut winapi::shared::minwindef::__some_function, extern "C" fn()>(
                func,
            )
        })
    }
}

impl Drop for DllManager {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.handle.as_ptr()) };
    }
}
