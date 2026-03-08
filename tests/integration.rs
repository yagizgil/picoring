use picoring::ring::PicoRing;
use picoring::MirrorBuffer;

#[test]
fn test_hardware_mirroring() {
    // Create a buffer with one page size (usually 4KB)
    let mut buffer = MirrorBuffer::new(4096)
        .expect("Failed to create MirrorBuffer. Ensure you have necessary OS permissions.");

    let slice = buffer.as_mut_slice();
    assert_eq!(
        slice.len(),
        8192,
        "Virtual address space should be twice the physical size"
    );

    // Writing to the first half
    slice[0] = 42;
    // Should be reflected in the second half
    assert_eq!(
        slice[4096], 42,
        "Mirroring failed: Index 4096 does not match index 0"
    );

    // Writing to the second half
    slice[4097] = 99;
    // Should be reflected in the first half
    assert_eq!(
        slice[1], 99,
        "Mirroring failed: Write to mirrored area did not update original area"
    );
}

#[test]
fn test_pico_ring_basic_logic() {
    let mut ring = PicoRing::<u32>::with_capacity(10).expect("Failed to create PicoRing");

    // Test initial state
    assert!(ring.is_empty());
    assert!(!ring.is_full());

    // Test pushing items up to the actual capacity (might be larger than 10 due to page alignment)
    let cap = ring.capacity();
    for i in 0..cap - 1 {
        assert!(ring.push(i as u32), "Failed to push item {}", i);
    }

    // Ring should be full now
    assert!(ring.is_full());
    assert!(!ring.push(99));

    // Test popping items
    for i in 0..cap - 1 {
        assert_eq!(ring.pop(), Some(i as u32));
    }

    assert!(ring.is_empty());
    assert_eq!(ring.pop(), None);
}

#[test]
fn test_pico_ring_wrap_around_with_mirroring() {
    let mut ring = PicoRing::<u8>::with_capacity(4096).expect("Failed to create PicoRing");

    // Fill up to the end
    for _ in 0..4095 {
        ring.push(0);
    }
    // Need to pop at least 5 items to make space for 5 bytes
    for _ in 0..5 {
        ring.pop();
    }
    // tail is now 6, head is 4095. available_space = 4096 - (4095 - 6) - 1 = 6.

    // Push across the boundary
    let data = [1, 2, 3, 4, 5];
    assert!(
        ring.push_slice(&data),
        "Failed to push slice across boundary"
    );

    // Read them back (process skip the initial zeros we pushed)
    for _ in 0..4090 {
        ring.pop();
    }

    assert_eq!(ring.pop(), Some(1));
    assert_eq!(ring.pop(), Some(2));
    assert_eq!(ring.pop(), Some(3));
    assert_eq!(ring.pop(), Some(4));
    assert_eq!(ring.pop(), Some(5));
}

#[test]
fn test_pico_ring_static_capacity() {
    // Test the new PicoRing::<u8, 8192>::new() syntax
    let mut ring = PicoRing::<u8, 4096>::new().expect("Failed to create static PicoRing");
    assert_eq!(ring.capacity(), 4096);

    ring.push(10);
    assert_eq!(ring.pop(), Some(10));
}
