use core::ptr;
use libc::*;

pub unsafe fn allocate_mirror(size: usize) -> Result<*mut u8, String> {
    // create an anonymous shared memory object in RAM
    // we use O_CREAT | O_EXCL to ensure it's a new, unique object
    let fd = shm_open(ptr::null(), O_RDWR | O_CREAT | O_EXCL, 0600);
    if fd < 0 {
        return Err("shm_open failed".into());
    }

    // immediately unlink the object
    // this keeps the object private and ensures it's deleted when the process exits
    shm_unlink(ptr::null());

    // set the physical size of the shared memory block
    ftruncate(fd, size as off_t);

    // reserve a contiguous virtual address space that is 2 * size
    // PROT_NONE means we can't read or write to it yet
    // MAP_ANONYMOUS creates a simple reservation without physical backing
    let target = mmap(
        // let the kernel choose the address
        ptr::null_mut(),
        // reserve enough space for two mirrored views
        2 * size,
        // no access allowed initially
        PROT_NONE,
        // private reservation not backed by any file
        MAP_PRIVATE | MAP_ANONYMOUS,
        // -1 because it's anonymous
        -1,
        // no offset
        0,
    );

    if target == MAP_FAILED {
        return Err("mmap reserve failed".into());
    }

    // map the physical shared memory to the FIRST half of our reserved space
    let m1 = mmap(
        // start at the beginning of the reserved range
        target,
        // map one 'size' worth of memory
        size,
        // allow reading and writing
        PROT_READ | PROT_WRITE,
        // MAP_FIXED tells mmap to use the exact address we provided
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
        return Err("First mapping failed".into());
    }

    // map the SAME physical memory again to the SECOND half
    // this creates the mirror/repeating effect
    let m2 = mmap(
        // start immediately after the first view
        target.add(size),
        // same size
        size,
        // same permissions
        PROT_READ | PROT_WRITE,
        // again, replace the reserved space at this specific address
        MAP_FIXED | MAP_SHARED,
        // use the same physical memory source
        fd,
        // IMPORTANT: offset is 0, so it points to the SAME start of the buffer
        0,
    );

    if m2 == MAP_FAILED {
        munmap(target, size); // unmap the successful first part
        munmap(target, 2 * size); // cleanup the whole reservation
        close(fd);
        return Err("Second mapping failed".into());
    }

    // we can close the descriptor now
    // the memory mappings will stay alive until we manually unmap them
    close(fd);

    // return the start of our mirrored virtual buffer
    Ok(target as *mut u8)
}

pub unsafe fn deallocate_mirror(ptr: *mut u8, size: usize) {
    if !ptr.is_null() {
        munmap(ptr as *mut c_void, 2 * size);
    }
}
