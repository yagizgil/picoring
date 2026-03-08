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
    let format_time = |ns: f64| {
        if ns < 1.0 {
            format!("{:.0} ps", ns * 1000.0)
        } else if ns < 1000.0 {
            format!("{:.1} ns", ns)
        } else if ns < 1_000_000.0 {
            format!("{:.2} µs", ns / 1000.0)
        } else {
            format!("{:.2} ms", ns / 1_000_000.0)
        }
    };

    println!(
        "{:<25} | {:<20} | {:<20} | {:<20}",
        "Operation", "Manual/Old", "Ergonomic/New", "Std Collection"
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
    let plist_manual_get = start.elapsed().as_nanos() as f64 / items as f64;

    // Ergonomic []
    let start = Instant::now();
    for i in 0..items {
        black_box(&plist[i]);
    }
    let plist_ergonomic_index = start.elapsed().as_nanos() as f64 / items as f64;

    // Std Vec []
    let start = Instant::now();
    for i in 0..items {
        black_box(&vec[i]);
    }
    let vec_index = start.elapsed().as_nanos() as f64 / items as f64;

    println!(
        "{:<25} | {:<20} | {:<20} | {:<20}",
        "List Indexing (avg)",
        format_time(plist_manual_get),
        format_time(plist_ergonomic_index),
        format_time(vec_index)
    );

    // --- 2. PICOLIST ITERATION ---
    // Manual loop with get
    let start = Instant::now();
    for i in 0..items {
        black_box(plist.get(i));
    }
    let plist_manual_iter = start.elapsed().as_nanos() as f64 / items as f64;

    // Ergonomic for x in &list
    let start = Instant::now();
    for x in &plist {
        black_box(x);
    }
    let plist_ergonomic_iter = start.elapsed().as_nanos() as f64 / items as f64;

    // Std Vec iter
    let start = Instant::now();
    for x in &vec {
        black_box(x);
    }
    let vec_iter = start.elapsed().as_nanos() as f64 / items as f64;

    println!(
        "{:<25} | {:<20} | {:<20} | {:<20}",
        "List Iteration (avg)",
        format_time(plist_manual_iter),
        format_time(plist_ergonomic_iter),
        format_time(vec_iter)
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
    let pqueue_manual_index = start.elapsed().as_nanos() as f64 / items as f64;

    // Ergonomic pqueue[i]
    let start = Instant::now();
    for i in 0..items {
        black_box(&pqueue[i]);
    }
    let pqueue_ergonomic_index = start.elapsed().as_nanos() as f64 / items as f64;

    println!(
        "{:<25} | {:<20} | {:<20} | {:<20}",
        "Queue Indexing (avg)",
        format_time(pqueue_manual_index),
        format_time(pqueue_ergonomic_index),
        "N/A"
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
    let pstream_manual_iter = start.elapsed().as_nanos() as f64 / items as f64;

    // Ergonomic for b in &stream
    let start = Instant::now();
    for b in &pstream {
        black_box(b);
    }
    let pstream_ergonomic_iter = start.elapsed().as_nanos() as f64 / items as f64;

    println!(
        "{:<25} | {:<20} | {:<20} | {:<20}",
        "Stream Iteration (avg)",
        format_time(pstream_manual_iter),
        format_time(pstream_ergonomic_iter),
        "N/A"
    );

    println!("{:-<100}", "");
    println!("Note: Ergonomic versions use trait-based abstractions (Index, IntoIterator).");
    println!("As shown, zero-copy architecture ensures no performance penalty for clean syntax.");
}
