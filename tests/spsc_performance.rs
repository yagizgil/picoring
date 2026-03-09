use picoring::{create_spsc, PicoRing};
use std::collections::VecDeque;
use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

#[test]
fn test_spsc_basic() {
    let (producer, consumer) = create_spsc::<u32>(4096).unwrap();

    let t1 = thread::spawn(move || {
        for i in 0..1000 {
            while !producer.push(i) {
                thread::yield_now();
            }
        }
    });

    let t2 = thread::spawn(move || {
        for i in 0..1000 {
            let mut val = consumer.pop();
            while val.is_none() {
                thread::yield_now();
                val = consumer.pop();
            }
            assert_eq!(val.unwrap(), i);
        }
    });

    t1.join().unwrap();
    t2.join().unwrap();
    println!("SPSC basic test passed!");
}

#[test]
fn test_spsc_into_from_ring() {
    let ring = PicoRing::<u32, 4096>::new().unwrap();
    let (producer, consumer) = ring.into_spsc();

    producer.push(100);
    assert_eq!(consumer.pop(), Some(100));
}

#[test]
fn compare_performance() {
    const ITEMS: u32 = 2_000_000; // Increased sample size
    const CAP: usize = 65536;

    // 1. Arc<Mutex<VecDeque>> - The "Standard" Alternative
    let queue = Arc::new(Mutex::new(VecDeque::with_capacity(CAP)));
    let start = Instant::now();
    {
        let q_prod = queue.clone();
        let t_prod = thread::spawn(move || {
            for i in 0..ITEMS {
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
        });

        let q_cons = queue.clone();
        let t_cons = thread::spawn(move || {
            let mut count = 0;
            while count < ITEMS {
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
        t_prod.join().unwrap();
        t_cons.join().unwrap();
    }
    let mutex_duration = start.elapsed();
    println!("Arc<Mutex<VecDeque>>: {:?}", mutex_duration);

    // 2. Classic PicoRing (Single Threaded - Baseline)
    let mut ring = PicoRing::<u32, CAP>::new().unwrap();
    let start = Instant::now();
    for i in 0..ITEMS {
        if ring.is_full() {
            black_box(ring.pop());
        }
        ring.push(i);
    }
    let classic_duration = start.elapsed();
    println!("Classic PicoRing (Single Thread): {:?}", classic_duration);

    // 3. SPSC PicoRing (Single Item)
    let (producer, consumer) = create_spsc::<u32>(CAP).unwrap();
    let start = Instant::now();
    {
        let t_prod = thread::spawn(move || {
            for i in 0..ITEMS {
                while !producer.push(i) {
                    // spin
                }
            }
        });

        let t_cons = thread::spawn(move || {
            let mut count = 0;
            while count < ITEMS {
                if let Some(val) = consumer.pop() {
                    black_box(val);
                    count += 1;
                }
            }
        });
        t_prod.join().unwrap();
        t_cons.join().unwrap();
    }
    let spsc_duration = start.elapsed();
    println!("SPSC PicoRing (Multi Thread): {:?}", spsc_duration);

    // 4. SPSC PicoRing (Batching - 64 items)
    let (producer, consumer) = create_spsc::<u32>(CAP).unwrap();
    let batch_data = [42u32; 64];
    let start = Instant::now();
    {
        let t_prod = thread::spawn(move || {
            let mut sent = 0;
            while sent < ITEMS {
                if producer.push_slice(&batch_data) {
                    sent += 64;
                }
            }
        });

        let t_cons = thread::spawn(move || {
            let mut received = 0;
            while received < ITEMS {
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
        t_prod.join().unwrap();
        t_cons.join().unwrap();
    }
    let batch_duration = start.elapsed();
    println!("SPSC Batching (64-item chunks): {:?}", batch_duration);

    println!("\nSummary ({} items):", ITEMS);
    println!(
        "Standard Mutex: {:>15} items/sec",
        (ITEMS as f64 / mutex_duration.as_secs_f64()) as u64
    );
    println!(
        "Classic Ring:   {:>15} items/sec",
        (ITEMS as f64 / classic_duration.as_secs_f64()) as u64
    );
    println!(
        "SPSC Single:    {:>15} items/sec",
        (ITEMS as f64 / spsc_duration.as_secs_f64()) as u64
    );
    println!(
        "SPSC Batching:  {:>15} items/sec",
        (ITEMS as f64 / batch_duration.as_secs_f64()) as u64
    );
}
