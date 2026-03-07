use core::hint::black_box;
use picoring::{PicoByteStream, PicoQueue};
use std::io::{Read, Write};
use std::time::Instant;

// Helper to format bytes into a human-readable string
fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

// Helper to print a benchmark row
fn print_row(label: &str, stream_ns: u128, queue_ns: u128, classic_ns: u128) {
    let stream_speedup = classic_ns as f64 / stream_ns as f64;
    let queue_speedup = classic_ns as f64 / queue_ns as f64;
    println!(
        "{:<12} | {:>12} | {:>12} | {:>12} | {:>10.1}x | {:>10.1}x",
        label, stream_ns, queue_ns, classic_ns, stream_speedup, queue_speedup
    );
}

#[test]
fn collection_performance_showdown() {
    let capacity_bytes = 600 * 1024 * 1024; // 600 MB Capacity (to handle 500MB tests)

    println!("\n--- COLLECTION PERFORMANCE SHOWDOWN ---");
    println!("Stream: std::io::Read/Write (Safe but Copying)");
    println!("Queue:  reserve/commit (Ultra Fast Zero-Copy)");
    println!("Classic: Vec with manual wrap logic");
    println!("{:-<100}", "");
    println!(
        "{:<12} | {:>12} | {:>12} | {:>12} | {:>11} | {:>11}",
        "Data Size", "Stream (ns)", "Queue (ns)", "Classic (ns)", "Stream Up", "Queue Up"
    );
    println!("{:-<100}", "");

    let mut stream = PicoByteStream::new(capacity_bytes).expect("Failed to create PicoByteStream");
    let mut queue = PicoQueue::<u8>::new(capacity_bytes).expect("Failed to create PicoQueue");
    let mut classic_vec = vec![0u8; capacity_bytes];

    let test_sizes = [
        8,                 // Tiny
        64,                // L1 Cache size
        4096,              // 4 KB Page
        65536,             // 64 KB L2
        1048576,           // 1 MB
        10 * 1024 * 1024,  // 10 MB
        50 * 1024 * 1024,  // 50 MB
        100 * 1024 * 1024, // 100 MB
        250 * 1024 * 1024, // 250 MB
        500 * 1024 * 1024, // 500 MB
    ];

    for &size in &test_sizes {
        let iterations = if size < 1_000_000 { 10_000 } else { 100 };
        let data = vec![42u8; size];
        let mut read_buffer = vec![0u8; size];

        // --- 1. PicoByteStream (std::io) ---
        let start_stream = Instant::now();
        for _ in 0..iterations {
            let _ = stream.write(&data).unwrap();
            let _ = stream.read(&mut read_buffer).unwrap();
            black_box(&read_buffer);
        }
        let stream_avg = start_stream.elapsed().as_nanos() / iterations as u128;

        // --- 2. PicoQueue (Reservation Based) ---
        let start_queue = Instant::now();
        for _ in 0..iterations {
            // Write (Zero-Copy Reservation)
            if let Some(buf) = queue.reserve(size) {
                buf.copy_from_slice(&data); // Directly into ring memory
                queue.commit(size);
            }

            // Read (Zero-Copy Peek)
            let readable = queue.peek();
            if readable.len() >= size {
                black_box(&readable[..size]);
                queue.release(size);
            }
        }
        let queue_avg = start_queue.elapsed().as_nanos() / iterations as u128;

        // --- 3. Classic Vec Wrap Cycle ---
        let mut classic_head = capacity_bytes - (size / 2);
        let mut classic_tail = classic_head;

        let start_classic = Instant::now();
        for _ in 0..iterations {
            // WRITE with manual wrap check
            if classic_head + size <= capacity_bytes {
                classic_vec[classic_head..classic_head + size].copy_from_slice(&data);
            } else {
                let first_part = capacity_bytes - classic_head;
                classic_vec[classic_head..capacity_bytes].copy_from_slice(&data[..first_part]);
                classic_vec[0..size - first_part].copy_from_slice(&data[first_part..]);
            }
            classic_head = (classic_head + size) % capacity_bytes;

            // READ with manual wrap check
            if classic_tail + size <= capacity_bytes {
                read_buffer.copy_from_slice(&classic_vec[classic_tail..classic_tail + size]);
            } else {
                let first_part = capacity_bytes - classic_tail;
                read_buffer[..first_part]
                    .copy_from_slice(&classic_vec[classic_tail..capacity_bytes]);
                read_buffer[first_part..].copy_from_slice(&classic_vec[..size - first_part]);
            }
            classic_tail = (classic_tail + size) % capacity_bytes;

            black_box(&read_buffer);
        }
        let classic_avg = start_classic.elapsed().as_nanos() / iterations as u128;

        print_row(&format_size(size), stream_avg, queue_avg, classic_avg);
    }
}
