use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use picoring::{PicoByteStream, PicoQueue, PicoRing};
use std::io::{Read, Write};

fn bench_read_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("Read Performance (Zero-Copy)");
    let capacity = 100 * 1024 * 1024; // 100MB

    let mut ring = PicoRing::<u8>::with_capacity(capacity).unwrap();
    let mut classic_vec = vec![0u8; capacity];

    let test_sizes = [64, 4096, 65536, 1048576]; // 64B, 4KB, 64KB, 1MB

    for size in test_sizes {
        let data = vec![42u8; size];
        ring.push_slice(&data);

        // Pico Performance
        group.bench_with_input(BenchmarkId::new("PicoRing", size), &size, |b, &s| {
            b.iter(|| {
                let slice = ring.readable_slice();
                black_box(&slice[..s]);
            });
        });

        // Classic Performance (Simulating reassemble/copy costs)
        group.bench_with_input(BenchmarkId::new("Classic (Copy)", size), &size, |b, &s| {
            let mut dest = vec![0u8; s];
            b.iter(|| {
                dest.copy_from_slice(&classic_vec[..s]);
                black_box(&dest);
            });
        });
    }
    group.finish();
}

fn bench_full_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("Full Cycle (Write+Read)");
    let capacity = 100 * 1024 * 1024;

    let mut pico_queue = PicoQueue::<u8>::new(capacity).unwrap();
    let mut classic_vec = vec![0u8; capacity];
    let mut classic_head = 0;
    let mut classic_tail = 0;

    let test_sizes = [64, 4096, 65536, 1048576];

    for size in test_sizes {
        let data = vec![7u8; size];
        let mut read_buf = vec![0u8; size];

        group.bench_with_input(BenchmarkId::new("PicoQueue", size), &size, |b, &s| {
            b.iter(|| {
                // Write
                if let Some(buf) = pico_queue.reserve(s) {
                    buf.copy_from_slice(&data);
                    pico_queue.commit(s);
                }
                // Read
                let readable = pico_queue.peek();
                black_box(&readable[..s]);
                pico_queue.release(s);
            });
        });

        group.bench_with_input(BenchmarkId::new("Classic", size), &size, |b, &s| {
            b.iter(|| {
                // Write
                classic_vec[classic_head..classic_head + s].copy_from_slice(&data);
                classic_head = (classic_head + s) % (capacity - s);
                // Read
                read_buf.copy_from_slice(&classic_vec[classic_tail..classic_tail + s]);
                classic_tail = (classic_tail + s) % (capacity - s);
                black_box(&read_buf);
            });
        });
    }
    group.finish();
}

fn bench_collections(c: &mut Criterion) {
    let mut group = c.benchmark_group("Collections (1MB)");
    let size = 1024 * 1024;
    let capacity = size * 2;
    let data = vec![0u8; size];
    let mut read_buf = vec![0u8; size];

    let mut stream = PicoByteStream::new(capacity).unwrap();
    let mut queue = PicoQueue::<u8>::new(capacity).unwrap();

    group.bench_function("ByteStream (std::io)", |b| {
        b.iter(|| {
            stream.write_all(&data).unwrap();
            stream.read_exact(&mut read_buf).unwrap();
            black_box(&read_buf);
        });
    });

    group.bench_function("PicoQueue (Zero-Copy)", |b| {
        b.iter(|| {
            if let Some(buf) = queue.reserve(size) {
                buf.copy_from_slice(&data);
                queue.commit(size);
            }
            let readable = queue.peek();
            black_box(&readable[..size]);
            queue.release(size);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_read_performance,
    bench_full_cycle,
    bench_collections
);
criterion_main!(benches);
