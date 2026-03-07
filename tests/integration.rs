use picoring::MirrorBuffer;
use picoring::ring::PicoRing;

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
    let mut ring = PicoRing::<u32>::new(10).expect("Failed to create PicoRing");

    // Test initial state
    assert!(ring.is_empty());
    assert!(!ring.is_full());

    // Test pushing items
    for i in 0..9 {
        assert!(ring.push(i), "Failed to push item {}", i);
    }

    // Ring should be full now (capacity - 1 for typical ring buffer implementation)
    // Note: PicoRing seems to use (head + 1) % capacity == tail as is_full
    assert!(ring.is_full());
    assert!(!ring.push(99));

    // Test popping items
    for i in 0..9 {
        assert_eq!(ring.pop(), Some(i));
    }

    assert!(ring.is_empty());
    assert_eq!(ring.pop(), None);
}

#[test]
fn test_pico_ring_wrap_around_with_mirroring() {
    let mut ring = PicoRing::<u8>::new(4096).expect("Failed to create PicoRing");

    // Fill up to the end
    for _ in 0..4095 {
        ring.push(0);
    }
    ring.pop(); // tail is now 1, head is 4095

    // Push across the boundary
    let data = [1, 2, 3, 4, 5];
    assert!(
        ring.push_slice(&data),
        "Failed to push slice across boundary"
    );

    // Read them back
    for _ in 0..4094 {
        ring.pop();
    }

    assert_eq!(ring.pop(), Some(1));
    assert_eq!(ring.pop(), Some(2));
    assert_eq!(ring.pop(), Some(3));
    assert_eq!(ring.pop(), Some(4));
    assert_eq!(ring.pop(), Some(5));
}
