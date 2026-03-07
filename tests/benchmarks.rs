use picoring::ring::PicoRing;
use std::hint::black_box;
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

// Helper to print a benchmark row with speedup status
fn print_row(label: &str, pico_ns: u128, classic_ns: u128) {
    let multiplier = classic_ns as f64 / pico_ns as f64;

    println!(
        "{:<15} | {:>15} | {:>15} | {:>9.2}x",
        label, pico_ns, classic_ns, multiplier
    );
}

#[test]
fn ultimate_performance_showdown() {
    // 600 MB physical capacity to accommodate 500 MB tests
    let capacity_bytes = 600 * 1024 * 1024;

    println!(
        "\nInitializing PicoRing and Classic buffers ({})...",
        format_size(capacity_bytes)
    );
    let mut ring =
        PicoRing::<u8>::with_capacity(capacity_bytes).expect("Failed to create PicoRing");
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

    // --- SCENARIO 1: WRITING (Zero-Wrap Copy) ---
    println!("\n[1/3] WRITE PERFORMANCE (Cross-Boundary Copy)");
    println!(
        "Description: Copying data into the ring buffer when it crosses the physical boundary."
    );
    println!("{:-<75}", "");
    println!(
        "{:<15} | {:>15} | {:>15} | {:>11}",
        "Packet Size", "Pico (avg ns)", "Classic (avg ns)", "Speedup"
    );
    println!("{:-<75}", "");

    for &size in &test_sizes {
        // Adjust iterations for large sizes to keep test time reasonable
        let iterations = if size < 1_000_000 {
            10_000
        } else if size < 100_000_000 {
            100
        } else {
            20
        };

        let data_to_push = vec![42u8; size];
        let split_pos = capacity_bytes - (size / 2);

        // PicoRing Write
        let start_pico = Instant::now();
        for _ in 0..iterations {
            unsafe {
                let dest = ring.as_mut_slice().as_mut_ptr().add(split_pos);
                core::ptr::copy_nonoverlapping(data_to_push.as_ptr(), dest, size);
                black_box(core::ptr::read_volatile(dest));
            }
        }
        let pico_avg = start_pico.elapsed().as_nanos() / iterations as u128;

        // Classic Split-Copy
        let start_classic = Instant::now();
        for _ in 0..iterations {
            let split_at = size / 2;
            let (first, second) = data_to_push.split_at(split_at);
            classic_vec[split_pos..capacity_bytes].copy_from_slice(first);
            classic_vec[0..(size - split_at)].copy_from_slice(second);
            black_box(classic_vec[0]);
        }
        let classic_avg = start_classic.elapsed().as_nanos() / iterations as u128;

        print_row(&format_size(size), pico_avg, classic_avg);
    }

    // --- SCENARIO 2: READING (Zero-Copy) ---
    println!("\n[2/3] READ PERFORMANCE (Zero-Copy vs Reassemble)");
    println!("Description: Accessing a contiguous slice of data that wraps around the buffer.");
    println!("{:-<75}", "");
    println!(
        "{:<15} | {:>15} | {:>15} | {:>11}",
        "Read Size", "Pico (avg ns)", "Classic (avg ns)", "Speedup"
    );
    println!("{:-<75}", "");

    for &size in &test_sizes {
        let iterations = if size < 1_000_000 {
            10_000
        } else if size < 100_000_000 {
            500
        } else {
            100
        };

        let split_pos = capacity_bytes - (size / 2);

        // PicoRing Read (Zero-Copy)
        let start_pico = Instant::now();
        for _ in 0..iterations {
            let slice = &ring.as_mut_slice()[split_pos..(split_pos + size)];
            black_box(slice[0]);
            black_box(slice[size - 1]);
        }
        let pico_avg = start_pico.elapsed().as_nanos() / iterations as u128;

        // Classic Read (Must Copy to Reassemble)
        let start_classic = Instant::now();
        for _ in 0..iterations {
            let mut temp_buf = vec![0u8; size]; // Allocation + Zeroing + Copy
            let first_part_len = capacity_bytes - split_pos;
            temp_buf[0..first_part_len].copy_from_slice(&classic_vec[split_pos..capacity_bytes]);
            temp_buf[first_part_len..size]
                .copy_from_slice(&classic_vec[0..(size - first_part_len)]);
            black_box(temp_buf[0]);
            black_box(temp_buf[size - 1]);
        }
        let classic_avg = start_classic.elapsed().as_nanos() / iterations as u128;

        print_row(&format_size(size), pico_avg, classic_avg);
    }

    // --- SCENARIO 3: FULL CYCLE (Write then Read) ---
    println!("\n[3/3] FULL CYCLE PERFORMANCE (Write + Read)");
    println!("Description: Combined time to write a packet and read it back immediately.");
    println!("{:-<75}", "");
    println!(
        "{:<15} | {:>15} | {:>15} | {:>11}",
        "Cycle Size", "Pico (avg ns)", "Classic (avg ns)", "Speedup"
    );
    println!("{:-<75}", "");

    for &size in &test_sizes {
        let iterations = if size < 1_000_000 {
            5_000
        } else if size < 100_000_000 {
            50
        } else {
            10
        };

        let data_to_write = vec![77u8; size];
        let split_pos = capacity_bytes - (size / 2);

        // PicoRing Full Cycle
        let start_pico = Instant::now();
        for _ in 0..iterations {
            // Write (Zero-wrap copy using pretty view API)
            if let Some(buf) = ring.view_mut(split_pos, size) {
                buf.copy_from_slice(&data_to_write);
            }

            // Read (Zero-copy slice using pretty view API)
            if let Some(slice) = ring.view(split_pos, size) {
                black_box(slice[size / 2]);
            }
        }
        let pico_avg = start_pico.elapsed().as_nanos() / iterations as u128;

        // Classic Full Cycle
        let start_classic = Instant::now();
        for _ in 0..iterations {
            // Write
            let split = size / 2;
            let (f, s) = data_to_write.split_at(split);
            classic_vec[split_pos..capacity_bytes].copy_from_slice(f);
            classic_vec[0..(size - split)].copy_from_slice(s);
            // Read
            let mut temp = vec![0u8; size];
            let flen = capacity_bytes - split_pos;
            temp[0..flen].copy_from_slice(&classic_vec[split_pos..capacity_bytes]);
            temp[flen..size].copy_from_slice(&classic_vec[0..(size - flen)]);
            black_box(temp[size / 2]);
        }
        let classic_avg = start_classic.elapsed().as_nanos() / iterations as u128;

        print_row(&format_size(size), pico_avg, classic_avg);
    }
}
