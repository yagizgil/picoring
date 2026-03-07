use std::ptr;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::Memory::*;

// we linking "CreateFileMappingW" function as manual
#[link(name = "kernel32")]
unsafe extern "system" {
    unsafe fn CreateFileMappingW(
        hfile: HANDLE,
        lpfilemappingattributes: *const std::ffi::c_void,
        flprotect: u32,
        dwmaximumsizehigh: u32,
        dwmaximumsizelow: u32,
        lpname: *const u16,
    ) -> HANDLE;
}

pub unsafe fn allocate_mirror(size: usize) -> Result<*mut u8, String> {
    // INVALID_HANDLE_VALUE: -1
    let h_file = -1 as isize as HANDLE;

    unsafe {
        // we create a physical memory block that can be mapped to multiple virtual addresses
        let section = CreateFileMappingW(
            // use system RAM (paging file) instead of a disk file
            h_file,
            // use null_mut() for default security descriptors
            // no need for child processes to inherit the handle
            ptr::null(),
            // PAGE_READWRITE allows reading and writing in this memory area
            PAGE_READWRITE,
            // high 32-bits of the 64-bit size
            // set to 0 as our buffer won't exceed 4GB
            0,
            // low 32-bits of the physical memory size
            // this is where the aligned 'size' variable is passed
            size as u32,
            // no name assigned (null)
            // the object remains anonymous and accessible only within this process
            ptr::null(),
        );

        // if section is 0, it means we failed to create a file mapping
        if section == 0 as HANDLE {
            return Err("CreateFileMappingW failed".into());
        }

        // VirtualAlloc2 reserves a contiguous range of virtual address space without allocating physical memory
        let placeholder = VirtualAlloc2(
            // current process handle. null_mut() as HANDLE indicates the current process
            ptr::null_mut() as HANDLE,
            // preferred starting address. null() let Windows choose a suitable location
            ptr::null(),
            // total size to reserve. we need 2 * size for our mirrored (back-to-back) views
            2 * size,
            // MEM_RESERVE reserves the space, MEM_RESERVE_PLACEHOLDER allows splitting this range later
            MEM_RESERVE | MEM_RESERVE_PLACEHOLDER,
            // initially no access is allowed to this reserved range to prevent accidental use
            PAGE_NOACCESS,
            // advanced allocation options. not needed for our standard ring buffer
            ptr::null_mut(),
            // number of extended parameters passed. Set to 0
            0,
        );

        // check if allocate virtual address space were successful
        if placeholder.is_null() {
            let _ = CloseHandle(section);
            return Err("VirtualAlloc2 failed".into());
        }

        /*
         * VirtualFree with MEM_PRESERVE_PLACEHOLDER splits a single reserved range
         * into smaller, independent placeholders for mirrored mapping
         */
        VirtualFree(
            // the starting address of the virtual range to be split
            placeholder,
            // the size of the first segment to split off
            // this defines the boundary where the two mirrored views will meet
            size,
            // MEM_RELEASE combined with PRESERVE_PLACEHOLDER tells windows
            // "dont delete this, just split it into two separate address placeholders"
            MEM_RELEASE | MEM_PRESERVE_PLACEHOLDER,
        );

        // MapViewOfFile3 maps the physical memory to the first reserved virtual address
        let view1 = MapViewOfFile3(
            // handle to the memory section we created earlier
            section,
            // current process handle
            ptr::null_mut() as HANDLE,
            // start mapping at the beginning of our reserved placeholder
            placeholder,
            // offset into the file mapping. we start from the beginning (0)
            0,
            // size of the memory to map
            size,
            // this flag tells Windows to replace the reserved placeholder with actual memory
            MEM_REPLACE_PLACEHOLDER,
            // give the view both read and write permissions
            PAGE_READWRITE,
            // no special extended parameters needed
            ptr::null_mut(),
            // number of extended parameters is 0
            0,
        );

        // MapViewOfFile3 maps the SAME physical memory to the second reserved virtual address
        // this creates the "mirror" effect where the buffer repeats itself
        let view2 = MapViewOfFile3(
            // use the same physical memory section
            section,
            // current process handle
            ptr::null_mut() as HANDLE,
            // start mapping immediately after the first view
            placeholder.add(size),
            // offset is still 0 because both views point to the SAME start of the buffer
            0,
            // map the same amount of memory
            size,
            // again, replace the second reserved placeholder
            MEM_REPLACE_PLACEHOLDER,
            // same permissions as the first view
            PAGE_READWRITE,
            // no extended parameters
            ptr::null_mut(),
            // param count is 0
            0,
        );

        // we can close the section handle now
        // the memory views will stay active until they are unmapped later
        let _ = CloseHandle(section);

        // check if both mapping operations were successful
        if view1.Value.is_null() || view2.Value.is_null() {
            // cleanup: if one view succeeded, unmap it before returning error
            if !view1.Value.is_null() {
                let _ = UnmapViewOfFile(view1);
            }
            if !view2.Value.is_null() {
                let _ = UnmapViewOfFile(view2);
            }
            return Err("MapViewOfFile3 failed".into());
        }

        // return the start address of our contiguous mirrored buffer
        Ok(placeholder as *mut u8)
    }
}

pub unsafe fn deallocate_mirror(ptr: *mut u8, size: usize) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        // unmap both views
        // windows-sys 0.61+ uses the MEMORY_MAPPED_VIEW_ADDRESS union for UnmapViewOfFile
        UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
            Value: ptr as *mut std::ffi::c_void,
        });

        UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
            Value: ptr.add(size) as *mut std::ffi::c_void,
        });
    }
}
