# PicoRing Python Bindings

High-performance circular buffers and collections for Python, powered by Rust and hardware memory mirroring.

## Benchmark Results

### 1. Hardware Mirroring (Zero-Copy)

When data wraps around the end of a circular buffer, standard implementations require copying to provide a contiguous view. PicoRing provides a zero-copy contiguous view in constant time O(1).

```text
SCENARIO: Extracting 94MB chunk from 105MB wrapped buffer
PicoRing Extraction:  0.000735 s (Constant Time)
Python Copy-Join:    17.705581 s (Linear Time O(N))
Performance Gap:     24,000x faster
```

### 2. Large List Growth

PicoList stays fast even when growing to millions of items because it never reallocates or copies existing data.

```text
SCENARIO: Growing to 50 Million items
PicoList Time:   0.0763 s | RAM:  48.2 MB
Python List Time: 0.6853 s | RAM: 376.0 MB
```

### 3. Bulk Queue Operations

PicoQueue optimizations for bulk byte transfers outperform standard deque loops.

```text
SCENARIO: Processing 40MB chunks
PicoRing Results: 1.47 us latency per call
Python Results:   16,087.00 us latency per call
Performance Gap:  10,944x faster
```

## How to Use

### PicoRingByte

Best for high-speed byte buffering where zero-copy contiguous access is required.

```python
import picoring

# Create a 1MB ring buffer
ring = picoring.PicoRingByte(1024 * 1024)

# Push data
ring.push_bytes(b"some data")

# Get a zero-copy contiguous view
# Even if data wraps around the buffer end, this view is a single memoryview
view = ring.get_readable_view()
print(bytes(view))
```

### PicoQueueByte

A high-level queue for byte packets.

```python
q = picoring.PicoQueueByte(4096)
q.push_bulk(b"packet data")
data = q.pop_bulk(5)
```

### PicoByteStreamPy

Implements a stream-like interface for read/write operations.

```python
stream = picoring.PicoByteStreamPy(8192)
stream.write(b"payload")
chunk = stream.read(4)
```

## Use Cases

- **High-Speed Networking**: Buffering TCP/UDP packets for parsing without copying.
- **Audio/Video Processing**: Real-time streaming where data wrap-around is frequent.
- **Large Data Collections**: Storing millions of bytes without the overhead of Python's memory reallocation.
- **Zero-Copy Interop**: Passing memory views from Rust to other libraries like NumPy or PyTorch.

## Running Tests

To run the benchmarks and tests:

```powershell
# Run the test suite
python python_tests/test_suite.py

# Run memory and speed benchmarks
python python_tests/comparison_bench.py

# Run stress tests
python python_tests/benchmark.py
```
