use picoring::PicoQueue;

fn main() {
    // Imagine this PicoQueue is shared between a "Network Thread" and a "Worker Thread"
    let mut channel = PicoQueue::<u64>::new(8192).unwrap();

    // PRODUCER SIDE (e.g., Network Thread)
    // Batch produce 100 values
    if let Some(slot) = channel.reserve(100) {
        for i in 0..100 {
            slot[i] = i as u64;
        }
        channel.commit(100);
    }

    // CONSUMER SIDE (e.g., Worker Thread)
    // Read all available messages at once as a single slice
    let messages = channel.peek();

    for msg in messages {
        // High-speed processing
        let _ = black_box_calc(*msg);
    }

    let processed_count = messages.len();
    channel.release(processed_count);

    println!(
        "Thread-like channel processed {} messages via zero-copy peek.",
        processed_count
    );
}

fn black_box_calc(v: u64) -> u64 {
    v.wrapping_mul(31)
}
