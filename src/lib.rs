mod os {
    #[cfg(windows)]
    pub mod windows;

    #[cfg(unix)]
    pub mod unix;
}

#[cfg(windows)]
pub use os::windows as handler;

#[cfg(unix)]
pub use os::unix as handler;
