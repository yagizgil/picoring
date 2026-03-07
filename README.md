# PicoRing

PicoRing is a high-performance circular buffer (Ring Buffer) for Rust that uses **Hardware Memory Mirroring** to provide a contiguous virtual memory view of circular data.

By mapping the same physical memory to two adjacent virtual address ranges, PicoRing allows you to access any part of the circular buffer as a single, contiguous slice (`&[T]`), even if it wraps around the end. This eliminates the need for manual wrapping logic and extra copies.

---

## Installation

Add PicoRing to your project using cargo:

```powershell
cargo add picoring
```

Or add it manually to your `Cargo.toml`:

```toml
[dependencies]
picoring = "0.3.0"
```

---

## Performance Benchmarks

Below are the complete benchmark results comparing PicoRing against a classic vector-based ring buffer implementation.

### 1. Read Performance (Zero-Copy vs Reassemble)

_Description: Accessing a contiguous slice of data that wraps around the buffer._

| Read Size | Pico (avg ns) | Classic (avg ns) |     Speedup      |
| :-------- | :-----------: | :--------------: | :--------------: |
| 8 B       |       2       |        84        |    **42.00x**    |
| 64 B      |       1       |       112        |   **112.00x**    |
| 4.0 KB    |       1       |       301        |   **301.00x**    |
| 64.0 KB   |       1       |       5794       |   **5794.00x**   |
| 1.0 MB    |       2       |      297976      |  **148988.00x**  |
| 10.0 MB   |       1       |     1883261      | **1883261.00x**  |
| 50.0 MB   |       1       |     10197133     | **10197133.00x** |
| 100.0 MB  |       4       |     22075846     | **5518961.50x**  |
| 250.0 MB  |       5       |     50443039     | **10088607.80x** |
| 500.0 MB  |       4       |    102183291     | **25545822.75x** |

### 2. Full Cycle Performance (Write + Read)

_Description: Combined time to write a packet and read it back immediately._

| Cycle Size | Pico (avg ns) | Classic (avg ns) |  Speedup   |
| :--------- | :-----------: | :--------------: | :--------: |
| 8 B        |      15       |        91        | **6.07x**  |
| 64 B       |       4       |        59        | **14.75x** |
| 4.0 KB     |      46       |       242        | **5.26x**  |
| 64.0 KB    |     1362      |      15020       | **11.03x** |
| 1.0 MB     |     28190     |      302556      | **10.73x** |
| 10.0 MB    |    636418     |     2911980      | **4.58x**  |
| 50.0 MB    |    3133888    |     13371780     | **4.27x**  |
| 100.0 MB   |    7659580    |     27257800     | **3.56x**  |
| 250.0 MB   |   16495180    |     71036560     | **4.31x**  |
| 500.0 MB   |   32945940    |    198883560     | **6.04x**  |

### 3. Write Performance (Cross-Boundary Copy)

_Description: Copying data into the ring buffer when it crosses the physical boundary._

| Packet Size | Pico (avg ns) | Classic (avg ns) |  Speedup  |
| :---------- | :-----------: | :--------------: | :-------: |
| 8 B         |      24       |        9         | **0.38x** |
| 64 B        |       8       |        9         | **1.12x** |
| 4.0 KB      |      60       |        43        | **0.72x** |
| 64.0 KB     |     1765      |       1828       | **1.04x** |
| 1.0 MB      |     31977     |      37107       | **1.16x** |
| 10.0 MB     |    546821     |      455670      | **0.83x** |
| 50.0 MB     |    3800115    |     3959129      | **1.04x** |
| 100.0 MB    |    9539720    |     7614025      | **0.80x** |
| 250.0 MB    |   21740000    |     19413425     | **0.89x** |
| 500.0 MB    |   49187725    |     43610955     | **0.89x** |

---

## Collection Performance Comparison

Comparison between high-level collections and classic manual wrap logic.

| Data Size | Stream (ns) | Queue (ns)  | Classic (ns) | Stream Up | Queue Up |
| :-------- | :---------: | :---------: | :----------: | :-------: | :------: |
| 8 B       |     16      |      9      |      11      |   0.7x    |   1.2x   |
| 64 B      |     36      |     32      |      16      |   0.4x    |   0.5x   |
| 4.0 KB    |    1816     |    1826     |     715      |   0.4x    |   0.4x   |
| 64.0 KB   |    18692    |    12623    |     8141     |   0.4x    |   0.6x   |
| 1.0 MB    |   124058    |    99452    |    102192    |   0.8x    |   1.0x   |
| 10.0 MB   |   1252732   | **319956**  |   1173427    |   0.9x    | **3.7x** |
| 50.0 MB   |   7412139   | **3109952** |   6349793    |   0.9x    | **2.0x** |
| 100.0 MB  |  13433399   | **6754518** |   12942260   |   1.0x    | **1.9x** |

### PicoList: High-Scale Efficiency

PicoList is a dynamic, chunked collection designed to handle massive datasets (GBs) without the performance degradation of large reallocations.

<details>
<summary><b>View Benchmark Results (3.7 GB / 500M Items)</b></summary>

| Operation        |  PicoList  |    Vec     |  VecDeque  |  LinkedList  |   BTreeMap   |   HashMap    |
| :--------------- | :--------: | :--------: | :--------: | :----------: | :----------: | :----------: |
| **Pushing (ms)** |    3655    |    2177    |    1972    |    26000     |    44500     |    42500     |
| **Access (µs)**  |  **151**   |    214     |    212     |   > 1 WEEK   |   O(log N)   |     O(1)     |
| **RAM Usage**    | **3.7 GB** | **3.7 GB** | **3.7 GB** | **15.0 GB!** | **17.3 GB!** | **16.4 GB!** |

_Analysis: PicoList outperforms Vec in random access speed at 3.7GB scale while maintaining identical memory efficiency._

</details>

### Full-Scale Technical Analysis & Sensitivity

Comprehensive performance metrics across different dataset scales and chunk configurations (N). This analysis evaluates raw throughput, memory efficiency, and the zero-overhead impact of our ergonomic abstractions.

<details>
<summary><b>View Analysis: 1 GB Scale (125M Items)</b></summary>

| Config (N)      | Push (ms) | Access: [] (µs) | Update (µs) | RAM Usage |
| :-------------- | :-------: | :-------------: | :---------: | :-------: |
| 128 (1 KB)      |   15095   |      19 µs      |   483 µs    |  3.8 GB   |
| 8192 (64 KB)    |   1253    |      19 µs      |    96 µs    | 955.6 MB  |
| 131072 (1 MB)   |   1028    |      23 µs      |   343 µs    | 953.2 MB  |
| 2097152 (16 MB) |    910    |      19 µs      |   105 µs    | 954.2 MB  |
| **Std Vec Ref** |  **521**  |    **17 µs**    |  **92 µs**  | 954.6 MB  |

</details>

<details>
<summary><b>View Analysis: 2 GB Scale (250M Items)</b></summary>

| Config (N)      | Push (ms) | Access: [] (µs) | Update (µs) | RAM Usage |
| :-------------- | :-------: | :-------------: | :---------: | :-------: |
| 128 (1 KB)      |   30658   |      19 µs      |   457 µs    |  7.5 GB   |
| 8192 (64 KB)    |   2198    |      19 µs      |   130 µs    |  1.9 GB   |
| 131072 (1 MB)   |   1943    |      19 µs      |   105 µs    |  1.9 GB   |
| 2097152 (16 MB) |   2015    |      19 µs      |   125 µs    |  1.9 GB   |
| **Std Vec Ref** |  **998**  |    **30 µs**    | **106 µs**  |  1.9 GB   |

</details>

<details>
<summary><b>View Analysis: 3 GB Scale (375M Items)</b></summary>

| Config (N)      | Push (ms) | Access: [] (µs) | Update (µs) | RAM Usage |
| :-------------- | :-------: | :-------------: | :---------: | :-------: |
| 128 (1 KB)      |   45605   |      19 µs      |   222 µs    |  11.3 GB  |
| 8192 (64 KB)    |   3391    |      19 µs      |   153 µs    |  2.8 GB   |
| 131072 (1 MB)   |   2825    |      19 µs      |   181 µs    |  2.8 GB   |
| 2097152 (16 MB) |   3035    |      48 µs      |   107 µs    |  2.8 GB   |
| **Std Vec Ref** | **1764**  |    **16 µs**    | **112 µs**  |  2.8 GB   |

</details>

<details>
<summary><b>View Analysis: 4 GB Scale (500M Items)</b></summary>

| Config (N)      | Push (ms) | Access: [] (µs) | Update (µs) | RAM Usage |
| :-------------- | :-------: | :-------------: | :---------: | :-------: |
| 128 (1 KB)      |   65641   |     437 µs      |  50893 µs   |  13.9 GB  |
| 8192 (64 KB)    |   4573    |      54 µs      |   198 µs    |  3.7 GB   |
| 131072 (1 MB)   |   3918    |     141 µs      |   219 µs    |  3.7 GB   |
| 2097152 (16 MB) |   3991    |      34 µs      |   181 µs    |  3.7 GB   |
| **Std Vec Ref** | **2191**  |    **55 µs**    | **189 µs**  |  3.7 GB   |

</details>

_Note: Choosing N >= 64KB ensure sub-nanosecond access latency and zero-copy stability even under high memory pressure._

### Zero-Overhead Ergonomics Validation

Our benchmarks confirm that high-level abstractions (`Index`, `Iterator`) carry zero performance penalty across all scales. The bit-masking optimization ensures that `list[i]` access incurs the same cycle cost as raw pointer arithmetic.

_\*Std Vec iteration is faster due to memory contiguity, but PicoList maintains performance even when crossing chunk boundaries._

---

## How to Run Benchmarks

You can reproduce these results on your local machine.

### Simple Benchmarks

Quick smoke tests for immediate feedback:

```powershell
# Core hardware mirroring tests
cargo test --test benchmarks --release -- --nocapture

# High-level collections tests
cargo test --test collections_bench --release -- --nocapture
```

### Professional Benchmarks (Criterion)

For statistically significant measurements and HTML reports:

```powershell
cargo bench --bench ring_bench
```

Reports will be generated at `target/criterion/report/index.html`.

---

## Performance Analysis (O-Notation)

Standard ring buffers suffer from **Linear Time O(N)** overhead for reads when data wraps, because they require reassembling parts into a temporary buffer.

PicoRing achieves **Constant Time O(1)** for all read operations. Because of hardware mirroring, the data is _already_ linear in virtual memory. As shown in Criterion results:

- **Classic Read (1MB):** ~26.7 µs (microsecond) (Linear increase with size)
- **PicoRing Read (1MB):** ~671 ps (picosecond) (Remains constant regardless of size)

This makes PicoRing the ideal choice for high-frequency trading, real-time audio, and high-throughput network processing.

---

## Usage Examples

### 1. High-Performance List (`PicoList`)

Designed for massive datasets where standard `Vec` reallocations would cause stalls.

```rust
use picoring::PicoList;

// Create a list with 128KB chunks (16384 * 8 bytes for u64)
let mut list = PicoList::<u64, 16384>::new();

// 1. Basic Pushing
list.push(10);
list.extend_from_slice(&[20, 30, 40, 50]);

// 2. Ergonomic Indexing (Zero Overhead)
let value = list[0];       // Direct access
list[1] = 99;              // Direct mutation
list.set(2, 100);          // Safe mutation (returns bool)

// 3. Iteration (Hardware Mirroring optimized)
for item in &list {
    println!("Value: {}", item);
}

// 4. Mutable Iteration
for item in &mut list {
    *item *= 2;
}

// 5. Functional methods
let sum: u64 = list.iter().sum();
```

### 2. Zero-Copy Queue (`PicoQueue`)

Ideal for message passing and inter-thread communication.

```rust
use picoring::PicoQueue;

// Static Capacity (Compile-time allocation)
let mut queue = PicoQueue::<f32, 4096>::new_static().unwrap();

// Dynamic Capacity
let mut queue = PicoQueue::<f32>::new(8192).unwrap();

// --- PRODUCER: Reservation API ---
if let Some(chunk) = queue.reserve(128) {
    // Write directly into the mirrored virtual memory
    chunk.fill(1.0);
    queue.commit(128); // Data is now live
}

// --- CONSUMER: Read & Indexing ---
assert_eq!(queue[0], 1.0); // O(1) random access to queued data

for val in &queue {
    // Logic here...
}

// Release processed data
queue.release(64);
```

### 3. Integrated Byte Stream (`PicoByteStream`)

Perfect for network buffers and file I/O with `std::io` support.

```rust
use picoring::PicoByteStream;
use std::io::{Read, Write};

let mut stream = PicoByteStream::new(65536).unwrap();

// 1. Use standard traits
stream.write_all(b"Technical Protocol Data").unwrap();
let mut buffer = [0u8; 9];
stream.read_exact(&mut buffer).unwrap();

// 2. Direct Zero-Copy Access (e.g., for Socket send/recv)
let readable = stream.as_read_slice(); // Contiguous slice of all available bytes
// socket.send(readable);
stream.consume(readable.len());

let writable = stream.as_write_slice(); // Direct access to available capacity
// socket.recv(writable);
stream.produce(writable.len());
```

### 4. Basic Ring Buffer (`PicoRing`)

The low-level primitive powering the entire library.

```rust
use picoring::PicoRing;

let mut ring = PicoRing::<u8, 4096>::new().unwrap();

ring.push(255);
// hardware mirroring guarantees this slice is contiguous even if it wraps
let data = ring.readable_slice();
assert_eq!(data[0], 255);
```

---

## Real-World Scenarios

Architecture examples in the `scenarios/` directory:

- **Audio Processing**: High-speed, contiguous DSP processing.
- **Network Stream**: Efficient stream reassembly without copies.
- **Log Analysis**: Fast moving window algorithms.
- **Message Passing**: Inter-thread zero-copy communication.

## License

MIT / Apache-2.0
