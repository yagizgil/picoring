use core::ptr;
use libc::*;

pub unsafe fn allocate_mirror(size: usize) -> Result<*mut u8, String> {
    // Generate a unique name for the shared memory object
    let name = format!(
        "/picoring_{}_{}_{}",
        getpid(),
        ptr::null::<u8>() as usize,
        size
    );
    let name_cstr = std::ffi::CString::new(name).map_err(|_| "invalid name")?;
    let name_ptr = name_cstr.as_ptr();

    // create a shared memory object in ram,
    // we use O_CREAT | O_EXCL to ensure it's a new, unique object
    let fd = shm_open(name_ptr, O_RDWR | O_CREAT | O_EXCL, 0600);
    if fd < 0 {
        return Err("shm_open failed".into());
    }

    // immediately unlink the object,
    // this keeps the object private and ensures it's deleted when the process exits
    shm_unlink(name_ptr);

    // set the physical size of the shared memory block
    if ftruncate(fd, size as off_t) != 0 {
        close(fd);
        return Err("ftruncate failed".into());
    }

    // reserve a contiguous virtual address space that is 2 * size,
    // PROT_NONE means we can't read or write to it yet
    let target = mmap(
        // let the kernel choose the address
        ptr::null_mut(),
        // reserve enough space for two mirrored views
        2 * size,
        // no access allowed initially
        PROT_NONE,
        // private reservation not backed by any file, using MAP_ANON for macos support
        MAP_PRIVATE | MAP_ANON,
        // -1 because it's anonymous
        -1,
        // no offset
        0,
    );

    if target == MAP_FAILED {
        close(fd);
        return Err("mmap reservation failed".into());
    }

    // map the physical shared memory to the first half of our reserved space,
    // allow reading and writing
    let m1 = mmap(
        // start at the beginning of the reserved range
        target,
        // map one 'size' worth of memory
        size,
        // allow reading and writing
        PROT_READ | PROT_WRITE,
        // MAP_FIXED tells mmap to use the exact address we provided,
        // MAP_SHARED makes changes visible across mappings
        MAP_FIXED | MAP_SHARED,
        // use our shared memory file descriptor
        fd,
        // start from the beginning of the physical memory (offset 0)
        0,
    );

    if m1 == MAP_FAILED {
        munmap(target, 2 * size);
        close(fd);
        return Err("mirror mapping 1 failed".into());
    }

    // map the same physical memory again to the second half,
    // this creates the mirror/repeating effect
    let m2 = mmap(
        // start immediately after the first view
        (target as *mut u8).add(size) as *mut c_void,
        // same size
        size,
        // same permissions
        PROT_READ | PROT_WRITE,
        // again, replace the reserved space at this specific address
        MAP_FIXED | MAP_SHARED,
        // use the same physical memory source
        fd,
        // important: offset is 0, so it points to the same start of the buffer
        0,
    );

    if m2 == MAP_FAILED {
        munmap(target, 2 * size);
        close(fd);
        return Err("mirror mapping 2 failed".into());
    }

    // we can close the descriptor now,
    // the memory mappings will stay alive until we manually unmap them
    close(fd);

    // return the start of our mirrored virtual buffer
    Ok(target as *mut u8)
}

pub unsafe fn deallocate_mirror(ptr: *mut u8, size: usize) {
    if !ptr.is_null() {
        // cleanup the whole 2 * size reservation
        munmap(ptr as *mut c_void, 2 * size);
    }
}
