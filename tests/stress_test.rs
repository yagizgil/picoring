use picoring::ring::PicoRing;
use std::time::Instant;

#[test]
fn pico_ring_data_integrity_stress_test() {
    // 1 MB physical capacity
    let capacity = 1024 * 1024;
    // Total 1 GB of data to process through this 1MB buffer
    let total_data_size = 1024 * 1024 * 1024;
    let chunk_size = 65536; // 64 KB chunks

    let mut ring = PicoRing::<u8>::with_capacity(capacity).expect("Failed to create PicoRing");

    println!("\n--- PICO RING STRESS TEST ---");
    println!("Buffer Size: {} MB", capacity / (1024 * 1024));
    println!("Total Data:  1 GB");
    println!("Chunk Size:  {} KB", chunk_size / 1024);
    println!("{:-<40}", "");

    let mut total_processed = 0;
    let mut next_byte_to_write: u8 = 0;
    let mut next_byte_to_read: u8 = 0;

    let start_time = Instant::now();

    while total_processed < total_data_size {
        // 1. FILL: Fill the ring as much as possible
        while ring.available_space() >= chunk_size && total_processed < total_data_size {
            let mut chunk = vec![0u8; chunk_size];
            for i in 0..chunk_size {
                chunk[i] = next_byte_to_write;
                next_byte_to_write = next_byte_to_write.wrapping_add(1);
            }

            assert!(ring.push_slice(&chunk), "FAILED TO PUSH SLICE");
            total_processed += chunk_size;
        }

        // 2. DRAIN & VERIFY: Read everything currently in the ring
        let readable = ring.readable_slice();
        let len_to_read = readable.len();

        for i in 0..len_to_read {
            if readable[i] != next_byte_to_read {
                panic!(
                    "DATA CORRUPTION AT BYTE {}! Expected {}, got {}",
                    total_processed - len_to_read + i,
                    next_byte_to_read,
                    readable[i]
                );
            }
            next_byte_to_read = next_byte_to_read.wrapping_add(1);
        }

        // Mark as read
        ring.advance_tail(len_to_read);

        // Progress report every 128MB
        if (total_processed % (128 * 1024 * 1024)) == 0 {
            println!("Processed: {} MB...", total_processed / (1024 * 1024));
        }
    }

    let duration = start_time.elapsed();
    let throughput = (total_data_size as f64 / (1024.0 * 1024.0)) / duration.as_secs_f64();

    println!("{:-<40}", "");
    println!("SUCCESS! All 1 GB verified without corruption.");
    println!("Time taken: {:.2?}", duration);
    println!("Throughput: {:.2} MB/s", throughput);
    println!("{:-<40}\n", "");
}
