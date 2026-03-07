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
picoring = "0.1.0"
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

let mut ring = PicoRing::<u8>::new(1024).unwrap();
ring.push(42);
let slice = ring.readable_slice(); // Always a contiguous slice
assert_eq!(slice[0], 42);
```

### 2. Zero-Copy Queue (PicoQueue)

Reservation-based API for maximum performance.

```rust
use picoring::PicoQueue;

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

---

## Real-World Scenarios

Architecture examples in the `scenarios/` directory:

- **Audio Processing**: High-speed, contiguous DSP processing.
- **Network Stream**: Efficient stream reassembly without copies.
- **Log Analysis**: Fast moving window algorithms.
- **Message Passing**: Inter-thread zero-copy communication.

## License

MIT / Apache-2.0
