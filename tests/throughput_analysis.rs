use picoring::PicoList;
use std::hint::black_box;
use std::time::Instant;

// Function removed as it was unused

#[test]
fn scale_parameter_ergonomics_bench() {
    let scales = [
        ("1 GB", 125_000_000),
        ("2 GB", 250_000_000),
        ("3 GB", 375_000_000),
    ];

    for (scale_label, items) in scales {
        println!(
            "\n--- THROUGHPUT & LATENCY SCALE ANALYSIS: {} ({} items) ---",
            scale_label, items
        );
        println!("{:-<110}", "");
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
            "{:<20} | {:>18} | {:>18} | {:>18} | {:>18}",
            "Config (N)", "Access: get()", "Access: []", "Iter: Manual", "Iter: Iterator"
        );
        println!("{:-<110}", "");

        let indices: Vec<usize> = (0..20_000).map(|i| (i * 13) % items).collect();
        let iter_limit = 500_000;

        // PicoList Sensitivity
        run_scale::<128>(items, &indices, iter_limit, "128 (1 KB)");
        run_scale::<8192>(items, &indices, iter_limit, "8192 (64 KB)");
        run_scale::<131072>(items, &indices, iter_limit, "131072 (1 MB)");
        run_scale::<2097152>(items, &indices, iter_limit, "2097152 (16 MB)");

        // Vec Reference
        {
            let mut vec = Vec::with_capacity(items);
            for i in 0..items {
                vec.push(i as u64);
            }

            let start = Instant::now();
            for &idx in &indices {
                black_box(&vec[idx]);
            }
            let vec_access = start.elapsed().as_nanos() as f64 / indices.len() as f64;

            let start = Instant::now();
            let mut _sum = 0u64;
            for x in vec.iter().take(iter_limit) {
                _sum += black_box(*x);
            }
            let vec_iter = start.elapsed().as_nanos() as f64 / iter_limit as f64;

            println!("{:-<110}", "");
            println!(
                "{:<20} | {:>18} | {:>18} | {:>18} | {:>18}",
                "Std Vec Ref",
                "N/A",
                format_time(vec_access),
                "N/A",
                format_time(vec_iter)
            );
        }
    }
}

fn run_scale<const N: usize>(items: usize, indices: &[usize], iter_limit: usize, label: &str) {
    let mut list = PicoList::<u64, N>::new();
    for i in 0..items {
        list.push(i as u64);
    }

    // Access benchmarks
    let start = Instant::now();
    for &idx in indices {
        black_box(list.get(idx));
    }
    let manual_get = start.elapsed().as_nanos() as f64 / indices.len() as f64;

    let start = Instant::now();
    for &idx in indices {
        black_box(&list[idx]);
    }
    let ergonomic_index = start.elapsed().as_nanos() as f64 / indices.len() as f64;

    // Iteration benchmarks
    let start = Instant::now();
    let mut _sum = 0u64;
    for i in 0..iter_limit {
        _sum += black_box(*list.get(i).unwrap());
    }
    let manual_iter = start.elapsed().as_nanos() as f64 / iter_limit as f64;

    let start = Instant::now();
    let mut _sum = 0u64;
    for x in list.iter().take(iter_limit) {
        _sum += black_box(*x);
    }
    let ergonomic_iter = start.elapsed().as_nanos() as f64 / iter_limit as f64;

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
        "{:<20} | {:>18} | {:>18} | {:>18} | {:>18}",
        label,
        format_time(manual_get),
        format_time(ergonomic_index),
        format_time(manual_iter),
        format_time(ergonomic_iter)
    );
}
