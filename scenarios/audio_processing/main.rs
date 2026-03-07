use picoring::PicoRing;

fn main() {
    // Simulate a 48kHz audio buffer (1 second of mono audio)
    let mut audio_buffer = PicoRing::<f32>::new(48000).unwrap();

    // 1. Simulate incoming audio frames
    let incoming_frame = vec![0.5f32; 1024];
    audio_buffer.push_slice(&incoming_frame);

    // 2. High-performance DSP processing (e.g., Applying Gain)
    // Hardware mirroring allows us to get a contiguous slice of ALL samples
    // currently in the buffer, NO MATTER WHERE they are physically.
    let samples = audio_buffer.readable_slice();

    // Process samples directly in place without any temporary copies
    // High-performance loops love contiguous memory for SIMD optimizations.
    for sample in samples.iter() {
        black_box_process(sample);
    }

    println!(
        "Processed {} audio samples with zero copies.",
        samples.len()
    );
}

fn black_box_process(s: &f32) {
    // Simulate DSP logic
    let _ = s * 2.0;
}
