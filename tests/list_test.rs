use picoring::PicoList;
use std::collections::{BTreeMap, HashMap, LinkedList, VecDeque};
use std::hint::black_box;
use std::time::Instant;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, get_current_pid};

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
fn rust_collections_ultimate_war() {
    let mut sys = System::new();
    let baseline_init = get_rss_bytes(&mut sys);

    // --- TABLE 1: MEGA WAR (500M items / 3.7 GB) ---
    let mega_items = 500_000_000;
    let expected_raw_mega = mega_items as u64 * 8;

    println!(
        "\n--- RUST COLLECTIONS MEGA WAR ({} per Collection) ---",
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
        [0u128; 3], [0u128; 3], [0u128; 3], [0u128; 3], [0u128; 3], [0u128; 3],
    );
    let (pico_mem, vec_mem, deque_mem, list_mem, btree_mem, hash_mem);

    // 1. PICOLIST
    {
        let start = Instant::now();
        let mut coll = PicoList::<u64, 131072>::new(); // 1MB chunks
        for i in 0..mega_items {
            coll.push(black_box(i as u64));
        }
        pico_res[0] = start.elapsed().as_millis();
        pico_mem = get_rss_bytes(&mut sys).saturating_sub(baseline_init);
        let start = Instant::now();
        for &idx in &indices {
            black_box(coll.get(idx));
        }
        pico_res[1] = start.elapsed().as_micros();
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
        vec_res[0] = start.elapsed().as_millis();
        vec_mem = get_rss_bytes(&mut sys).saturating_sub(baseline);
        let start = Instant::now();
        for &idx in &indices {
            black_box(coll.get(idx));
        }
        vec_res[1] = start.elapsed().as_micros();
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
        deque_res[0] = start.elapsed().as_millis();
        deque_mem = get_rss_bytes(&mut sys).saturating_sub(baseline);
        let start = Instant::now();
        for &idx in &indices {
            black_box(coll.get(idx));
        }
        deque_res[1] = start.elapsed().as_micros();
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
        list_res[0] = start.elapsed().as_millis() * (mega_items / 1_000_000) as u128;
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
        btree_res[0] = start.elapsed().as_millis() * (mega_items / 1_000_000) as u128;
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
        hash_res[0] = start.elapsed().as_millis() * (mega_items / 1_000_000) as u128;
        hash_mem =
            get_rss_bytes(&mut sys).saturating_sub(baseline) * (mega_items / 1_000_000) as u64;
    }

    println!(
        "{:<15} | {:>15} | {:>12} | {:>12} | {:>15} | {:>12} | {:>12}",
        "Pushing (ms)",
        pico_res[0],
        vec_res[0],
        deque_res[0],
        list_res[0],
        btree_res[0],
        hash_res[0]
    );
    println!(
        "{:<15} | {:>15} | {:>12} | {:>12} | {:>15} | {:>12} | {:>12}",
        "Access (us)", pico_res[1], vec_res[1], deque_res[1], "> 1 WEEK", "O(log N)", "O(1)"
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

    // --- SECTION: 1GB, 2GB, 3GB, 4GB SENSITIVITY ---
    let chunk_configs = [
        (128, "1 KB"),
        (1024, "8 KB"),
        (8192, "64 KB"),
        (32768, "256 KB"),
        (65536, "512 KB"),
        (131072, "1 MB"),
        (262144, "2 MB"),
        (655360, "5 MB"),
        (2097152, "16 MB"),
    ];

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
            "N (Chunk Size)", "Push (ms)", "Access (us)", "Update (us)", "RAM Usage"
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
            let push_ms = start.elapsed().as_millis();
            let mem = get_rss_bytes(&mut sys).saturating_sub(baseline);

            let start = Instant::now();
            for &idx in &indices {
                black_box(coll.get(idx));
            }
            let get_us = start.elapsed().as_micros();

            let start = Instant::now();
            for &idx in &indices {
                if let Some(v) = coll.get_mut(idx) {
                    *v = black_box(*v + 1);
                }
            }
            let mut_us = start.elapsed().as_micros();

            println!("{:-<100}", "");
            println!(
                "{:<20} | {:>15} | {:>15} | {:>15} | {:>15}",
                "Std Vec Ref",
                push_ms,
                get_us,
                mut_us,
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
    let push_ms = start.elapsed().as_millis();
    let mem = get_rss_bytes(sys).saturating_sub(baseline);

    let start = Instant::now();
    for &idx in indices {
        black_box(pico.get(idx));
    }
    let get_us = start.elapsed().as_micros();

    let start = Instant::now();
    for &idx in indices {
        if let Some(v) = pico.get_mut(idx) {
            *v = black_box(*v + 1);
        }
    }
    let mut_us = start.elapsed().as_micros();

    println!(
        "{:<20} | {:>15} | {:>15} | {:>15} | {:>15}",
        label,
        push_ms,
        get_us,
        mut_us,
        format_bytes(mem)
    );
    black_box(pico);
}
