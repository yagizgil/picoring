use picoring::PicoQueue;

#[derive(Debug, Clone, Copy)]
struct LogEvent {
    timestamp: u64,
    level: u8,
    error_code: u32,
}

fn main() {
    let mut window = PicoQueue::<LogEvent>::new(1000).unwrap();

    // Fill the window with some events
    for i in 0..1000 {
        let event = LogEvent {
            timestamp: i,
            level: 1,
            error_code: 0,
        };
        window.try_push(event);
    }

    // Now, let's analyze the entire window as a single contiguous slice
    let events = window.peek();

    // We can use standard slice methods like .iter(), .chunks(), .window()
    // even though this is a circular buffer!
    let average_error =
        events.iter().map(|e| e.error_code as f64).sum::<f64>() / events.len() as f64;

    // Slide the window: remove oldest 10, add 10 new
    window.release(10);
    for i in 1000..1010 {
        window.try_push(LogEvent {
            timestamp: i,
            level: 2,
            error_code: 10,
        });
    }

    println!(
        "Analysis complete. Avg error: {}. Moving window works seamlessly.",
        average_error
    );
}
