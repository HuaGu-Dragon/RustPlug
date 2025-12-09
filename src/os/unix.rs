use std::{
    ffi::{CString, c_void},
    path::Path,
    ptr::NonNull,
};

use anyhow::Context;
use libc::{RTLD_LAZY, dlopen, dlsym};
use libffi::{low::CodePtr, middle::Cif};

pub struct DllManager {
    handle: NonNull<c_void>,
}

impl DllManager {
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            anyhow::bail!("file did not exist");
        }

        let filename = CString::new(
            path.to_str()
                .context("Failed to convert path to string, check if the path is valid utf-8")?,
        )?;

        let handle = unsafe { dlopen(filename.as_ptr(), RTLD_LAZY) };

        let handle = NonNull::new(handle).with_context(|| {
            format!(
                "Failed to load Dynamic Lib, error: {:?}",
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
            dlsym(
                self.handle.as_ptr(),
                CString::new(name.as_ref())
                    .context("Convert Rust str to CStr")?
                    .as_ptr(),
            )
        };

        if func.is_null() {
            anyhow::bail!(
                "Failed to find function, error: {:?}",
                std::io::Error::last_os_error()
            )
        }

        Ok(unsafe { std::mem::transmute::<*mut c_void, extern "C" fn()>(func) })
    }

    /// Calls a function with the given arguments.
    ///
    /// In particular, this method invokes function `func` passing it
    /// arguments `args`, and returns the result.
    ///
    /// # Safety
    ///
    /// There is no checking that the calling convention and types
    /// in the `Cif` match the actual calling convention and types of
    /// `fun`, nor that they match the types of `args`.
    pub unsafe fn call_func<'arg, R>(
        &self,
        func: impl AsRef<str>,
        args: impl IntoIterator<Item = (libffi::middle::Type, libffi::middle::Arg<'arg>)>,
        ret: libffi::middle::Type,
    ) -> anyhow::Result<R> {
        let func = unsafe {
            self.get_func(func)
                .context("Get func addr from Dynamic Lib")?
        };

        let (types, values): (Vec<_>, Vec<_>) = args.into_iter().unzip();

        let cif = Cif::new(types, ret);

        Ok(unsafe { cif.call(CodePtr::from_fun(func), &values) })
    }
}
