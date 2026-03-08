use picoring::PicoList;
use std::collections::{BTreeMap, HashMap, LinkedList, VecDeque};
use std::hint::black_box;
use std::time::Instant;
use sysinfo::{get_current_pid, ProcessRefreshKind, ProcessesToUpdate, System};

fn format_time(ns: f64) -> String {
    if ns < 1.0 {
        format!("{:.0} ps", ns * 1000.0)
    } else if ns < 1000.0 {
        format!("{:.1} ns", ns)
    } else if ns < 1_000_000.0 {
        format!("{:.2} µs", ns / 1000.0)
    } else {
        format!("{:.2} ms", ns / 1_000_000.0)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn get_rss_bytes(sys: &mut System) -> u64 {
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::everything(),
    );
    sys.process(get_current_pid().expect("failed to get pid"))
        .map(|p| p.memory())
        .unwrap_or(0)
}

#[test]
fn rust_collections_scale_analysis() {
    let mut sys = System::new();
    let baseline_init = get_rss_bytes(&mut sys);

    // --- TABLE 1: HIGH-CAPACITY BENCHMARK (500M items / 3.7 GB) ---
    let mega_items = 500_000_000;
    let expected_raw_mega = mega_items as u64 * 8;

    println!(
        "\n--- HIGH-CAPACITY COLLECTION BENCHMARK ({} per Collection) ---",
        format_bytes(expected_raw_mega)
    );
    println!("{:-<145}", "");
    println!(
        "{:<15} | {:>15} | {:>12} | {:>12} | {:>15} | {:>12} | {:>12}",
        "Operation", "PicoList", "Vec", "VecDeque", "LinkedList", "BTreeMap", "HashMap"
    );
    println!("{:-<145}", "");

    let access_count = 100_000;
    let indices: Vec<usize> = (0..access_count).map(|i| (i * 13) % mega_items).collect();

    let (mut pico_res, mut vec_res, mut deque_res, mut list_res, mut btree_res, mut hash_res) = (
        [0.0f64; 3],
        [0.0f64; 3],
        [0.0f64; 3],
        [0.0f64; 3],
        [0.0f64; 3],
        [0.0f64; 3],
    );
    let (pico_mem, vec_mem, deque_mem, list_mem, btree_mem, hash_mem);

    // 1. PICOLIST
    {
        let start = Instant::now();
        let mut coll = PicoList::<u64, 131072>::new(); // 1MB chunks
        for i in 0..mega_items {
            coll.push(black_box(i as u64));
        }
        pico_res[0] = start.elapsed().as_nanos() as f64;
        pico_mem = get_rss_bytes(&mut sys).saturating_sub(baseline_init);
        let start = Instant::now();
        for &idx in &indices {
            black_box(coll.get(idx));
        }
        pico_res[1] = start.elapsed().as_nanos() as f64 / indices.len() as f64;
        black_box(coll); // ensure it's not dropped early
    }
    let baseline = get_rss_bytes(&mut sys);

    // 2. VEC
    {
        let start = Instant::now();
        let mut coll = Vec::new();
        for i in 0..mega_items {
            coll.push(black_box(i as u64));
        }
        vec_res[0] = start.elapsed().as_nanos() as f64;
        vec_mem = get_rss_bytes(&mut sys).saturating_sub(baseline);
        let start = Instant::now();
        for &idx in &indices {
            black_box(coll.get(idx));
        }
        vec_res[1] = start.elapsed().as_nanos() as f64 / indices.len() as f64;
        black_box(coll);
    }
    let baseline = get_rss_bytes(&mut sys);

    // 3. VECDEQUE
    {
        let start = Instant::now();
        let mut coll = VecDeque::new();
        for i in 0..mega_items {
            coll.push_back(black_box(i as u64));
        }
        deque_res[0] = start.elapsed().as_nanos() as f64;
        deque_mem = get_rss_bytes(&mut sys).saturating_sub(baseline);
        let start = Instant::now();
        for &idx in &indices {
            black_box(coll.get(idx));
        }
        deque_res[1] = start.elapsed().as_nanos() as f64 / indices.len() as f64;
        black_box(coll);
    }
    let baseline = get_rss_bytes(&mut sys);

    // 4. LINKEDLIST (Sampled)
    {
        let start = Instant::now();
        let mut coll = LinkedList::new();
        let items = 1_000_000;
        for i in 0..items {
            coll.push_back(i as u64);
        }
        list_res[0] = start.elapsed().as_nanos() as f64 * (mega_items / 1_000_000) as f64;
        list_mem =
            get_rss_bytes(&mut sys).saturating_sub(baseline) * (mega_items / 1_000_000) as u64;
    }
    let baseline = get_rss_bytes(&mut sys);

    // 5. BTREEMAP (Sampled)
    {
        let start = Instant::now();
        let mut coll = BTreeMap::new();
        let items = 1_000_000;
        for i in 0..items {
            coll.insert(i as u64, i as u64);
        }
        btree_res[0] = start.elapsed().as_nanos() as f64 * (mega_items / 1_000_000) as f64;
        btree_mem =
            get_rss_bytes(&mut sys).saturating_sub(baseline) * (mega_items / 1_000_000) as u64;
    }
    let baseline = get_rss_bytes(&mut sys);

    // 6. HASHMAP (Sampled)
    {
        let start = Instant::now();
        let mut coll = HashMap::new();
        let items = 1_000_000;
        for i in 0..items {
            coll.insert(i as u64, i as u64);
        }
        hash_res[0] = start.elapsed().as_nanos() as f64 * (mega_items / 1_000_000) as f64;
        hash_mem =
            get_rss_bytes(&mut sys).saturating_sub(baseline) * (mega_items / 1_000_000) as u64;
    }

    println!(
        "{:<15} | {:>15} | {:>12} | {:>12} | {:>15} | {:>12} | {:>12}",
        "Pushing",
        format_time(pico_res[0]),
        format_time(vec_res[0]),
        format_time(deque_res[0]),
        format_time(list_res[0]),
        format_time(btree_res[0]),
        format_time(hash_res[0])
    );
    println!(
        "{:<15} | {:>15} | {:>12} | {:>12} | {:>15} | {:>12} | {:>12}",
        "Access (avg)",
        format_time(pico_res[1]),
        format_time(vec_res[1]),
        format_time(deque_res[1]),
        "> 1 WEEK",
        "O(log N)",
        "O(1)"
    );
    println!(
        "{:<15} | {:>15} | {:>12} | {:>12} | {:>15} | {:>12} | {:>12}",
        "RAM Usage",
        format_bytes(pico_mem),
        format_bytes(vec_mem),
        format_bytes(deque_mem),
        format_bytes(list_mem),
        format_bytes(btree_mem),
        format_bytes(hash_mem)
    );
    println!("{:-<145}", "");

    // --- SECTION: PARAMETER SENSITIVITY ANALYSIS (1GB TO 4GB) ---

    let scales = [
        ("1 GB", 125_000_000),
        ("2 GB", 250_000_000),
        ("3 GB", 375_000_000),
        ("4 GB", 500_000_000),
    ];

    for (scale_label, items) in scales {
        println!(
            "\n--- SENSITIVITY TEST FOR {} ({} Items) ---",
            scale_label, items
        );
        println!("{:-<100}", "");
        println!(
            "{:<20} | {:>15} | {:>15} | {:>15} | {:>15}",
            "N (Chunk Size)", "Push", "Access", "Update", "RAM Usage"
        );
        println!("{:-<100}", "");

        let indices: Vec<usize> = (0..20_000).map(|i| (i * 13) % items).collect();

        // PicoList runs
        run_sens::<128>(&mut sys, items, &indices, "128 (1 KB)");
        run_sens::<1024>(&mut sys, items, &indices, "1024 (8 KB)");
        run_sens::<8192>(&mut sys, items, &indices, "8192 (64 KB)");
        run_sens::<32768>(&mut sys, items, &indices, "32768 (256 KB)");
        run_sens::<65536>(&mut sys, items, &indices, "65536 (512 KB)");
        run_sens::<131072>(&mut sys, items, &indices, "131072 (1 MB)");
        run_sens::<262144>(&mut sys, items, &indices, "262144 (2 MB)");
        run_sens::<655360>(&mut sys, items, &indices, "655360 (5 MB)");
        run_sens::<2097152>(&mut sys, items, &indices, "2097152 (16 MB)");

        // Vec Reference for this scale
        {
            let baseline = get_rss_bytes(&mut sys);
            let start = Instant::now();
            let mut coll = Vec::new();
            for j in 0..items {
                coll.push(black_box(j as u64));
            }
            let push_time = start.elapsed().as_nanos() as f64;
            let mem = get_rss_bytes(&mut sys).saturating_sub(baseline);

            let start = Instant::now();
            for &idx in &indices {
                black_box(coll.get(idx));
            }
            let get_time = start.elapsed().as_nanos() as f64 / indices.len() as f64;

            let start = Instant::now();
            for &idx in &indices {
                if let Some(v) = coll.get_mut(idx) {
                    *v = black_box(*v + 1);
                }
            }
            let mut_time = start.elapsed().as_nanos() as f64 / indices.len() as f64;

            println!("{:-<100}", "");
            println!(
                "{:<20} | {:>15} | {:>15} | {:>15} | {:>15}",
                "Std Vec Ref",
                format_time(push_time),
                format_time(get_time),
                format_time(mut_time),
                format_bytes(mem)
            );
            black_box(coll);
        }
        println!("{:-<100}", "");
    }
}

fn run_sens<const N: usize>(sys: &mut System, items: usize, indices: &[usize], label: &str) {
    let baseline = get_rss_bytes(sys);

    let start = Instant::now();
    let mut pico = PicoList::<u64, N>::new();
    for i in 0..items {
        pico.push(black_box(i as u64));
    }
    let push_time = start.elapsed().as_nanos() as f64;
    let mem = get_rss_bytes(sys).saturating_sub(baseline);

    let start = Instant::now();
    for &idx in indices {
        black_box(pico.get(idx));
    }
    let get_time = start.elapsed().as_nanos() as f64 / indices.len() as f64;

    let start = Instant::now();
    for &idx in indices {
        if let Some(v) = pico.get_mut(idx) {
            *v = black_box(*v + 1);
        }
    }
    let mut_time = start.elapsed().as_nanos() as f64 / indices.len() as f64;

    println!(
        "{:<20} | {:>15} | {:>15} | {:>15} | {:>15}",
        label,
        format_time(push_time),
        format_time(get_time),
        format_time(mut_time),
        format_bytes(mem)
    );
    black_box(pico);
}
