use picoring::{PicoByteStream, PicoList, PicoQueue, PicoRing};
use std::io::{Read, Write};

#[test]
fn pico_ring_correctness() {
    // 4KB physical (usually 1 page)
    let mut ring = PicoRing::<u8>::with_capacity(4096).unwrap();
    let cap = ring.capacity();

    // 1. Fill almost to full
    for i in 0..cap - 1 {
        ring.push((i % 256) as u8);
    }
    assert!(ring.is_full());

    // 2. Pop half
    for i in 0..cap / 2 {
        assert_eq!(ring.pop(), Some((i % 256) as u8));
    }

    // 3. Push across boundary to verify mirroring
    let data = vec![7u8; 100];
    assert!(ring.push_slice(&data));

    let readable = ring.readable_slice();
    assert!(readable.contains(&7));
    assert_eq!(ring.len(), (cap - 1) - (cap / 2) + 100);
}

#[test]
fn pico_list_correctness() {
    // Small chunks for testing
    let mut list = PicoList::<u64, 128>::new();

    // Push across multiple chunks
    for i in 0..1000 {
        list.push(i as u64);
    }

    assert_eq!(list.len(), 1000);
    assert_eq!(list[0], 0);
    assert_eq!(list[999], 999);

    // Iteration
    let sum: u64 = list.iter().sum();
    assert_eq!(sum, (0..1000).sum());

    // Mutation via index
    list[500] = 999999;
    assert_eq!(list[500], 999999);
}

#[test]
fn pico_queue_correctness() {
    let mut queue = PicoQueue::<f32>::new(1024).unwrap();

    // Reservation API
    if let Some(buf) = queue.reserve(10) {
        buf.fill(1.5);
        queue.commit(10);
    }

    assert_eq!(queue.len(), 10);
    assert_eq!(queue[0], 1.5);
    assert_eq!(queue[9], 1.5);

    // Peek and release
    {
        let peeked = queue.peek();
        assert_eq!(peeked.len(), 10);
        assert_eq!(peeked[5], 1.5);
    }
    queue.release(5);
    assert_eq!(queue.len(), 5);
}

#[test]
fn pico_byte_stream_correctness() {
    let mut stream = PicoByteStream::new(4096).unwrap();

    // std::io::Write
    stream.write_all(b"hello pico").unwrap();

    // std::io::Read
    let mut buf = [0u8; 5];
    stream.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"hello");

    // Integration with slices
    let remaining = stream.as_read_slice();
    assert_eq!(remaining, b" pico");

    let len = remaining.len();
    stream.consume(len);
    assert_eq!(stream.available_to_read(), 0);
}

#[test]
fn cross_boundary_slice_integrity() {
    // Force a wrap-around scenario
    let mut ring = PicoRing::<u8>::with_capacity(4096).unwrap();
    let cap = ring.capacity();

    // Move pointers near the end
    for _ in 0..cap - 10 {
        ring.push(0);
    }
    for _ in 0..cap - 10 {
        ring.pop();
    }
    // head and tail are now at cap-10

    let test_data = (0..20).collect::<Vec<u8>>();
    assert!(ring.push_slice(&test_data)); // This MUST wrap around the physical boundary

    let readable = ring.readable_slice();
    assert_eq!(readable.len(), 20);
    assert_eq!(readable, test_data.as_slice());

    // Random access through mirroring view
    assert_eq!(ring.view(ring.tail(), 20).unwrap(), test_data.as_slice());
}
