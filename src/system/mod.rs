// Export platform-specific memory allocation
#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::{allocate_mirror, deallocate_mirror};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{allocate_mirror, deallocate_mirror};

// helper to get the OS page size
pub fn get_page_size() -> usize {
    #[cfg(unix)]
    unsafe {
        // get page size for Unix
        libc::sysconf(libc::_SC_PAGESIZE) as usize
    }
    #[cfg(windows)]
    unsafe {
        // use GetSystemInfo for Windows
        let mut info = std::mem::zeroed();
        windows_sys::Win32::System::SystemInformation::GetSystemInfo(&mut info);
        info.dwPageSize as usize
    }
}
