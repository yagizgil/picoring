# PicoRing

PicoRing is a high-performance circular buffer (Ring Buffer) for Rust that uses **Hardware Memory Mirroring** to provide a contiguous virtual memory view of circular data.

By mapping the same physical memory to two adjacent virtual address ranges, PicoRing allows you to access any part of the circular buffer as a single, contiguous slice (`&[T]`), even if it wraps around the end. This eliminates the need for manual wrapping logic and extra copies.

---

## Performance Benchmarks

Below are the complete benchmark results comparing PicoRing against a classic vector-based ring buffer implementation.

### 1. Read Performance (Zero-Copy vs Reassemble)

_Description: Accessing a contiguous slice of data that wraps around the buffer._

| Read Size | Pico (avg ns) | Classic (avg ns) |     Speedup      |
| :-------- | :-----------: | :--------------: | :--------------: |
| 8 B       |       1       |        49        |    **49.00x**    |
| 64 B      |       1       |        69        |    **69.00x**    |
| 4.0 KB    |       1       |       200        |   **200.00x**    |
| 64.0 KB   |       0       |       2630       |     **infx**     |
| 1.0 MB    |       2       |      240148      |  **120074.00x**  |
| 10.0 MB   |       1       |     1942358      | **1942358.00x**  |
| 50.0 MB   |       1       |     9809676      | **9809676.00x**  |
| 100.0 MB  |       4       |     19199420     | **4799855.00x**  |
| 250.0 MB  |       8       |     48513931     | **6064241.38x**  |
| 500.0 MB  |       4       |    103540057     | **25885014.25x** |

### 2. Full Cycle Performance (Write + Read)

_Description: Combined time to write a packet and read it back immediately._

| Cycle Size | Pico (avg ns) | Classic (avg ns) |  Speedup  |
| :--------- | :-----------: | :--------------: | :-------: |
| 8 B        |      15       |        75        | **5.00x** |
| 64 B       |       8       |        63        | **7.88x** |
| 4.0 KB     |      92       |       205        | **2.23x** |
| 64.0 KB    |     1684      |      15476       | **9.19x** |
| 1.0 MB     |     41084     |      352016      | **8.57x** |
| 10.0 MB    |    669244     |     2934064      | **4.38x** |
| 50.0 MB    |    3511548    |     15252756     | **4.34x** |
| 100.0 MB   |    6817460    |     24923890     | **3.66x** |
| 250.0 MB   |   16368680    |     65310000     | **3.99x** |
| 500.0 MB   |   33242780    |    137514910     | **4.14x** |

### 3. Write Performance (Cross-Boundary Copy)

_Description: Copying data into the ring buffer when it crosses the physical boundary._

| Packet Size | Pico (avg ns) | Classic (avg ns) |  Speedup  |
| :---------- | :-----------: | :--------------: | :-------: |
| 8 B         |      16       |        11        | **0.69x** |
| 64 B        |       8       |        10        | **1.25x** |
| 4.0 KB      |      44       |        42        | **0.95x** |
| 64.0 KB     |     1618      |       1561       | **0.96x** |
| 1.0 MB      |     46380     |      28452       | **0.61x** |
| 10.0 MB     |    509513     |      339644      | **0.67x** |
| 50.0 MB     |    3336179    |     3161439      | **0.95x** |
| 100.0 MB    |    8084115    |     7447950      | **0.92x** |
| 250.0 MB    |   19724620    |     17509530     | **0.89x** |
| 500.0 MB    |   42631735    |     37274320     | **0.87x** |

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

You can reproduce these results on your local machine:

```powershell
# Core performance tests
cargo test --test benchmarks --release -- --nocapture

# High-level collections tests
cargo test --test collections_bench --release -- --nocapture
```

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
