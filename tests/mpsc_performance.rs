use picoring::{create_mpsc, PicoRing};
use std::collections::VecDeque;
use std::hint::black_box;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Instant;

#[test]
fn test_mpsc_basic() {
    let (producer, consumer) = create_mpsc::<u32>(4096).unwrap();
    let num_producers = 4;
    let items_per_producer = 1000;

    let mut producers = Vec::new();
    for p_id in 0..num_producers {
        let p = producer.clone();
        producers.push(thread::spawn(move || {
            for i in 0..items_per_producer {
                while !p.push(p_id * 1000 + i) {
                    thread::yield_now();
                }
            }
        }));
    }

    let t_cons = thread::spawn(move || {
        let mut count = 0;
        let total_items = num_producers * items_per_producer;
        while count < total_items {
            if let Some(_val) = consumer.pop() {
                count += 1;
            } else {
                thread::yield_now();
            }
        }
        count
    });

    for p in producers {
        p.join().unwrap();
    }
    let count = t_cons.join().unwrap();
    assert_eq!(count, num_producers * items_per_producer);
    println!("MPSC basic test passed!");
}

#[test]
fn compare_mpsc_performance() {
    const TOTAL_ITEMS: u32 = 2_000_000;
    const NUM_PRODUCERS: u32 = 4;
    const ITEMS_PER_PROD: u32 = TOTAL_ITEMS / NUM_PRODUCERS;
    const CAP: usize = 65536;

    println!("\n--- MPSC Performance Comparison ({} producers, {} items) ---", NUM_PRODUCERS, TOTAL_ITEMS);

    // 1. Arc<Mutex<VecDeque>>
    let queue = Arc::new(Mutex::new(VecDeque::with_capacity(CAP)));
    let start = Instant::now();
    {
        let mut producers = Vec::new();
        for _ in 0..NUM_PRODUCERS {
            let q_prod = queue.clone();
            producers.push(thread::spawn(move || {
                for i in 0..ITEMS_PER_PROD {
                    loop {
                        let mut q = q_prod.lock().unwrap();
                        if q.len() < CAP {
                            q.push_back(i);
                            break;
                        }
                        drop(q);
                        thread::yield_now();
                    }
                }
            }));
        }

        let q_cons = queue.clone();
        let t_cons = thread::spawn(move || {
            let mut count = 0;
            while count < TOTAL_ITEMS {
                let mut q = q_cons.lock().unwrap();
                if let Some(val) = q.pop_front() {
                    black_box(val);
                    count += 1;
                } else {
                    drop(q);
                    thread::yield_now();
                }
            }
        });

        for p in producers { p.join().unwrap(); }
        t_cons.join().unwrap();
    }
    let mutex_duration = start.elapsed();
    println!("Arc<Mutex<VecDeque>>: {:?}", mutex_duration);

    // 2. std::sync::mpsc (Bound)
    let (tx, rx) = mpsc::sync_channel(CAP);
    let start = Instant::now();
    {
        let mut producers = Vec::new();
        for _ in 0..NUM_PRODUCERS {
            let tx_clone = tx.clone();
            producers.push(thread::spawn(move || {
                for i in 0..ITEMS_PER_PROD {
                    tx_clone.send(i).unwrap();
                }
            }));
        }
        drop(tx); // Close original tx

        let t_cons = thread::spawn(move || {
            let mut count = 0;
            while let Ok(val) = rx.recv() {
                black_box(val);
                count += 1;
            }
            count
        });

        for p in producers { p.join().unwrap(); }
        t_cons.join().unwrap();
    }
    let std_mpsc_duration = start.elapsed();
    println!("std::sync::mpsc:      {:?}", std_mpsc_duration);

    // 3. PicoMPSC (Single Item)
    let (producer, consumer) = create_mpsc::<u32>(CAP).unwrap();
    let start = Instant::now();
    {
        let mut producers = Vec::new();
        for _ in 0..NUM_PRODUCERS {
            let p = producer.clone();
            producers.push(thread::spawn(move || {
                for i in 0..ITEMS_PER_PROD {
                    while !p.push(i) {
                        core::hint::spin_loop();
                    }
                }
            }));
        }

        let t_cons = thread::spawn(move || {
            let mut count = 0;
            while count < TOTAL_ITEMS {
                if let Some(val) = consumer.pop() {
                    black_box(val);
                    count += 1;
                }
            }
        });

        for p in producers { p.join().unwrap(); }
        t_cons.join().unwrap();
    }
    let pico_mpsc_duration = start.elapsed();
    println!("PicoMPSC Single:      {:?}", pico_mpsc_duration);

    // 4. PicoMPSC (Batching - 64 items)
    let (producer, consumer) = create_mpsc::<u32>(CAP).unwrap();
    let batch_data = [42u32; 64];
    let start = Instant::now();
    {
        let mut producers = Vec::new();
        for _ in 0..NUM_PRODUCERS {
            let p = producer.clone();
            producers.push(thread::spawn(move || {
                let mut sent = 0;
                while sent < ITEMS_PER_PROD {
                    if p.push_slice(&batch_data) {
                        sent += 64;
                    }
                }
            }));
        }

        let t_cons = thread::spawn(move || {
            let mut received = 0;
            while received < TOTAL_ITEMS {
                let slice = consumer.readable_slice();
                if !slice.is_empty() {
                    let n = slice.len();
                    for i in 0..n {
                        black_box(slice[i]);
                    }
                    consumer.advance_tail(n);
                    received += n as u32;
                }
            }
        });

        for p in producers { p.join().unwrap(); }
        t_cons.join().unwrap();
    }
    let pico_batch_duration = start.elapsed();
    println!("PicoMPSC Batching:    {:?}", pico_batch_duration);

    println!("\nSummary ({} items):", TOTAL_ITEMS);
    println!(
        "Mutex:           {:>15} items/sec",
        (TOTAL_ITEMS as f64 / mutex_duration.as_secs_f64()) as u64
    );
    println!(
        "std::sync::mpsc: {:>15} items/sec",
        (TOTAL_ITEMS as f64 / std_mpsc_duration.as_secs_f64()) as u64
    );
    println!(
        "PicoMPSC Single: {:>15} items/sec",
        (TOTAL_ITEMS as f64 / pico_mpsc_duration.as_secs_f64()) as u64
    );
    println!(
        "PicoMPSC Batch:  {:>15} items/sec",
        (TOTAL_ITEMS as f64 / pico_batch_duration.as_secs_f64()) as u64
    );
}
