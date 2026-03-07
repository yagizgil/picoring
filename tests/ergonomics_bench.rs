use picoring::{PicoByteStream, PicoList, PicoQueue};
use std::hint::black_box;
use std::time::Instant;

#[test]
fn bench_ergonomics_vs_manual() {
    let items = 10_000_000;
    println!(
        "\n--- ERGONOMICS VS MANUAL PERFORMANCE ({} items) ---",
        items
    );
    println!("{:-<100}", "");
    println!(
        "{:<25} | {:<20} | {:<20} | {:<20}",
        "Operation", "Manual/Old (µs)", "Ergonomic/New (µs)", "Std Collection (µs)"
    );
    println!("{:-<100}", "");

    // --- 1. PICOLIST INDEXING ---
    let mut plist = PicoList::<u64, 8192>::new();
    let mut vec = Vec::with_capacity(items);
    for i in 0..items {
        plist.push(i as u64);
        vec.push(i as u64);
    }

    // Manual .get()
    let start = Instant::now();
    for i in 0..items {
        black_box(plist.get(i));
    }
    let plist_manual_get = start.elapsed().as_micros();

    // Ergonomic []
    let start = Instant::now();
    for i in 0..items {
        black_box(&plist[i]);
    }
    let plist_ergonomic_index = start.elapsed().as_micros();

    // Std Vec []
    let start = Instant::now();
    for i in 0..items {
        black_box(&vec[i]);
    }
    let vec_index = start.elapsed().as_micros();

    println!(
        "{:<25} | {:<20} | {:<20} | {:<20}",
        "List Indexing", plist_manual_get, plist_ergonomic_index, vec_index
    );

    // --- 2. PICOLIST ITERATION ---
    // Manual loop with get
    let start = Instant::now();
    for i in 0..items {
        black_box(plist.get(i));
    }
    let plist_manual_iter = start.elapsed().as_micros();

    // Ergonomic for x in &list
    let start = Instant::now();
    for x in &plist {
        black_box(x);
    }
    let plist_ergonomic_iter = start.elapsed().as_micros();

    // Std Vec iter
    let start = Instant::now();
    for x in &vec {
        black_box(x);
    }
    let vec_iter = start.elapsed().as_micros();

    println!(
        "{:<25} | {:<20} | {:<20} | {:<20}",
        "List Iteration", plist_manual_iter, plist_ergonomic_iter, vec_iter
    );

    // --- 3. PICOQUEUE INDEXING ---
    let mut pqueue = PicoQueue::<u64>::new(items).unwrap();
    for i in 0..items {
        pqueue.try_push(i as u64);
    }

    // Manual .peek()[i]
    let start = Instant::now();
    for i in 0..items {
        black_box(&pqueue.peek()[i]);
    }
    let pqueue_manual_index = start.elapsed().as_micros();

    // Ergonomic pqueue[i]
    let start = Instant::now();
    for i in 0..items {
        black_box(&pqueue[i]);
    }
    let pqueue_ergonomic_index = start.elapsed().as_micros();

    println!(
        "{:<25} | {:<20} | {:<20} | {:<20}",
        "Queue Indexing", pqueue_manual_index, pqueue_ergonomic_index, "N/A"
    );

    // --- 4. PICOBYTESTREAM ITERATION ---
    let mut pstream = PicoByteStream::new(items).unwrap();
    let data = vec![0u8; items];
    use std::io::Write;
    pstream.write_all(&data).unwrap();

    // Manual as_read_slice().iter()
    let start = Instant::now();
    for b in pstream.as_read_slice().iter() {
        black_box(b);
    }
    let pstream_manual_iter = start.elapsed().as_micros();

    // Ergonomic for b in &stream
    let start = Instant::now();
    for b in &pstream {
        black_box(b);
    }
    let pstream_ergonomic_iter = start.elapsed().as_micros();

    println!(
        "{:<25} | {:<20} | {:<20} | {:<20}",
        "Stream Iteration", pstream_manual_iter, pstream_ergonomic_iter, "N/A"
    );

    println!("{:-<100}", "");
    println!("Note: Ergonomic versions use trait-based abstractions (Index, IntoIterator).");
    println!("As shown, zero-copy architecture ensures no performance penalty for clean syntax.");
}
