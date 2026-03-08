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
    use std::sync::OnceLock;
    static PAGE_SIZE: OnceLock<usize> = OnceLock::new();

    *PAGE_SIZE.get_or_init(|| {
        #[cfg(unix)]
        unsafe {
            // get page size for Unix
            let res = libc::sysconf(libc::_SC_PAGESIZE);
            if res <= 0 {
                4096 // Fallback to 4KB if sysconf fails
            } else {
                res as usize
            }
        }
        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::System::SystemInformation::*;
            // use GetSystemInfo for Windows
            let mut info: SYSTEM_INFO = std::mem::zeroed();
            GetSystemInfo(&mut info);
            info.dwPageSize as usize
        }
        #[cfg(not(any(unix, windows)))]
        {
            4096 // Default fallback
        }
    })
}
