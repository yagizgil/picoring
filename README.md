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
picoring = "0.2.0"
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

## Collection Performance Showdown

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

### PicoList: Extreme Scale Performance

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

<details>
<summary><b>View 1 GB Sensitivity Data (125M Items)</b></summary>

| N (Chunk Size)  | Push (ms) | Access (µs) | Update (µs) | RAM Usage |
| :-------------- | :-------: | :---------: | :---------: | :-------: |
| 128 (1 KB)      |   15095   |     55      |     483     |  3.8 GB   |
| 1024 (8 KB)     |   2559    |     34      |     151     | 959.3 MB  |
| 8192 (64 KB)    |   1253    |     34      |     96      | 955.6 MB  |
| 32768 (256 KB)  |    941    |     33      |     111     | 954.5 MB  |
| 65536 (512 KB)  |    989    |     54      |     118     | 953.8 MB  |
| 131072 (1 MB)   |   1028    |     32      |     343     | 953.2 MB  |
| 262144 (2 MB)   |    970    |     35      |     107     | 953.7 MB  |
| 655360 (5 MB)   |    919    |     45      |     110     | 954.2 MB  |
| 2097152 (16 MB) |    910    |     34      |     105     | 954.2 MB  |
| **Std Vec Ref** |    521    |   **30**    |   **92**    | 954.6 MB  |

</details>

<details>
<summary><b>View 2 GB Sensitivity Data (250M Items)</b></summary>

| N (Chunk Size)  | Push (ms) | Access (µs) | Update (µs) | RAM Usage |
| :-------------- | :-------: | :---------: | :---------: | :-------: |
| 128 (1 KB)      |   30658   |     52      |     457     |  7.5 GB   |
| 1024 (8 KB)     |   5236    |     53      |     204     |  1.9 GB   |
| 8192 (64 KB)    |   2198    |     40      |     130     |  1.9 GB   |
| 32768 (256 KB)  |   1957    |     37      |     153     |  1.9 GB   |
| 65536 (512 KB)  |   1904    |     24      |     112     |  1.9 GB   |
| 131072 (1 MB)   |   1943    |     32      |     105     |  1.9 GB   |
| 262144 (2 MB)   |   1948    |     53      |     180     |  1.9 GB   |
| 655360 (5 MB)   |   1744    |     45      |     106     |  1.9 GB   |
| 2097152 (16 MB) |   2015    |     35      |     125     |  1.9 GB   |
| **Std Vec Ref** |    998    |     30      |     106     |  1.9 GB   |

</details>

<details>
<summary><b>View 3 GB Sensitivity Data (375M Items)</b></summary>

| N (Chunk Size)  | Push (ms) | Access (µs) | Update (µs) | RAM Usage |
| :-------------- | :-------: | :---------: | :---------: | :-------: |
| 128 (1 KB)      |   45605   |     20      |     222     |  11.3 GB  |
| 1024 (8 KB)     |   8134    |     51      |     218     |  2.8 GB   |
| 8192 (64 KB)    |   3391    |     36      |     153     |  2.8 GB   |
| 32768 (256 KB)  |   2927    |     56      |     168     |  2.8 GB   |
| 65536 (512 KB)  |   2869    |     61      |     504     |  2.8 GB   |
| 131072 (1 MB)   |   2825    |     32      |     107     |  2.8 GB   |
| 262144 (2 MB)   |   2979    |     51      |     204     |  2.8 GB   |
| 655360 (5 MB)   |   2752    |     77      |     209     |  2.8 GB   |
| 2097152 (16 MB) |   3035    |     34      |     107     |  2.8 GB   |
| **Std Vec Ref** |   1764    |     80      |     112     |  2.8 GB   |

</details>

<details>
<summary><b>View 4 GB Sensitivity Data (500M Items)</b></summary>

| N (Chunk Size)  | Push (ms) | Access (µs) | Update (µs) | RAM Usage |
| :-------------- | :-------: | :---------: | :---------: | :-------: |
| 128 (1 KB)      |   65641   |     437     |    50893    |  13.9 GB  |
| 1024 (8 KB)     |   10337   |     36      |     183     |  3.7 GB   |
| 8192 (64 KB)    |   4573    |     54      |     198     |  3.7 GB   |
| 32768 (256 KB)  |   4346    |     48      |     154     |  3.7 GB   |
| 65536 (512 KB)  |   3845    |     52      |     157     |  3.7 GB   |
| 131072 (1 MB)   |   3918    |     141     |     219     |  3.7 GB   |
| 262144 (2 MB)   |   4047    |     56      |     198     |  3.7 GB   |
| 655360 (5 MB)   |   3970    |     83      |     220     |  3.7 GB   |
| 2097152 (16 MB) |   3991    |     34      |     181     |  3.7 GB   |
| **Std Vec Ref** |   2191    |     55      |     189     |  3.7 GB   |

</details>

_Note: Choosing N >= 64KB ensures peak hardware synergy and memory stability._

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

### 1. Basic Ring Buffer (PicoRing)

Fundamental access to the mirrored buffer.

```rust
use picoring::PicoRing;

// Option A: Static capacity (via const generics)
let mut ring = PicoRing::<u8, 4096>::new().unwrap();

// Option B: Dynamic capacity
let mut ring = PicoRing::<u8>::with_capacity(1024).unwrap();

ring.push(42);
let slice = ring.readable_slice(); // Always a contiguous slice
assert_eq!(slice[0], 42);
```

### 2. Zero-Copy Queue (PicoQueue)

Reservation-based API for maximum performance.

```rust
use picoring::PicoQueue;

// Static version
let mut queue = PicoQueue::<u32, 1024>::new_static().unwrap();

// Dynamic version
let mut queue = PicoQueue::<u32>::new(1024).unwrap();
// Write directly into reserved memory
if let Some(buf) = queue.reserve(2) {
    buf[0] = 10;
    buf[1] = 20;
    queue.commit(2);
}

// Read without copying
let data = queue.peek();
queue.release(data.len());
```

### 3. Byte Stream (PicoByteStream)

Implementation of standard Read and Write traits.

```rust
use picoring::PicoByteStream;
use std::io::{Read, Write};

let mut stream = PicoByteStream::new(4096).unwrap();
stream.write_all(b"Hello Pico").unwrap();
```

### 4. Dynamic Chunked List (PicoList)

Optimized for massive datasets, avoids large reallocations.

```rust
use picoring::PicoList;

// 1MB chunks (131072 * 8 bytes for u64)
let mut list = PicoList::<u64, 131072>::new();

list.push(100);
list.extend_from_slice(&[200, 300, 400]);

assert_eq!(*list.get(0).unwrap(), 100);
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
