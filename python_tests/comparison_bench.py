import picoring
import time
import os
import psutil

def get_mem():
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / (1024 * 1024) # MB

def bench_case(title):
    print(f"\n{'='*60}")
    print(f"CASE: {title}")
    print(f"{'='*60}")

def scenario_memory_fragmentation():
    bench_case("MEMORY FRAGMENTATION & REALLOCATION (Large List Growth)")
    # Testing how collections handle growing to huge sizes
    # Python List: Reallocates and copies the entire array multiple times
    # PicoList: Allocates new 16KB chunks without touching old memory
    
    ITEMS = 50_000_000 # 50 Million items (u8)
    
    print(f"Growing to {ITEMS/1e6:.0f} Million items...")
    
    # 1. PicoList
    m_start = get_mem()
    l_pico = picoring.PicoListByte()
    start = time.perf_counter()
    # Using extend for realistic bulk growth
    chunk = bytes([0] * 1024 * 1024) # 1MB chunk
    for _ in range(ITEMS // (1024 * 1024)):
        l_pico.extend(chunk)
    t_pico = time.perf_counter() - start
    m_end = get_mem()
    print(f"PicoList Growth Time: {t_pico:.4f} s | RAM Impact: {m_end - m_start:.1f} MB")

    # 2. Python List
    m_start = get_mem()
    l_py = []
    start = time.perf_counter()
    for _ in range(ITEMS // (1024 * 1024)):
        l_py.extend(chunk)
    t_py = time.perf_counter() - start
    m_end = get_mem()
    print(f"Python List Growth Time: {t_py:.4f} s | RAM Impact: {m_end - m_start:.1f} MB")

def scenario_massive_wrap_around():
    bench_case("MASSIVE ZERO-COPY WRAP-AROUND (IO/Network Buffering)")
    # Scenario: Processing a continuous stream where data always wraps the buffer end
    # We want a 100MB contiguous view of a 100MB buffer that has wrapped.
    
    BUF_SIZE = 1024 * 1024 * 100 # 100 MB
    READ_SIZE = 1024 * 1024 * 90 # 90 MB read
    ITERATIONS = 500 # Total 45 GB of view extractions
    
    print(f"Processing {ITERATIONS} wraps of {READ_SIZE/1e6:.0f}MB in a {BUF_SIZE/1e6:.0f}MB ring...")

    # 1. PicoRing (Hardware Mirroring)
    ring = picoring.PicoRingByte(BUF_SIZE)
    # Force a wrap state: push 60MB, pop 50MB, then push 80MB
    ring.push_bytes(bytes([0] * (BUF_SIZE // 2 + 10_000_000)))
    ring.pop_bytes(BUF_SIZE // 2)
    ring.push_bytes(bytes([0] * (BUF_SIZE // 2 + 20_000_000)))
    
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        # Hardware magic: O(1) regardless of read size
        _ = ring.get_readable_view()
    t_pico = time.perf_counter() - start
    print(f"PicoRing View Extraction: {t_pico:.6f} s (Constant Time)")

    # 2. Python Manual Join (What you normally have to do)
    # Python CANNOT view wrapped data as contiguous. It MUST copy.
    py_buf = bytearray(BUF_SIZE)
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        # Simulated wrap: 10MB at end, 80MB at start
        view = py_buf[BUF_SIZE-10_000_000:] + py_buf[:80_000_000] # O(N) allocation + copy
    t_py = time.perf_counter() - start
    print(f"Python Re-allocation + Copy: {t_py:.6f} s (Linear Time O(N))")
    
    print(f"PICO ADVANTAGE: {t_py / t_pico:.1f}x reduction in CPU/Memory churn")

def scenario_io_streaming():
    bench_case("IO STREAMING (Constant Memory Footprint)")
    # Scenario: Reading 1GB of data through a stream
    # BytesIO grows its internal buffer. PicoByteStream stays fixed.
    
    TOTAL_DATA = 1024 * 1024 * 500 # 500 MB
    CHUNK = 1024 * 64 # 64 KB
    
    print(f"Streaming {TOTAL_DATA/1e6:.0f}MB through a fixed buffer...")

    # 1. PicoByteStream (Fixed Circular Buffer)
    m_start = get_mem()
    stream = picoring.PicoByteStreamPy(1024 * 512) # 512KB fixed buffer
    data = bytes([0] * CHUNK)
    start = time.perf_counter()
    for _ in range(TOTAL_DATA // CHUNK):
        stream.write(data)
        _ = stream.read(CHUNK)
    t_pico = time.perf_counter() - start
    m_end = get_mem()
    print(f"PicoByteStream: {t_pico:.4f} s | Buffer RAM: {m_end - m_start:.2f} MB")

    # 2. Python BytesIO (Standard behavior)
    m_start = get_mem()
    stream_py = io_bytes() # helper below
    start = time.perf_counter()
    for _ in range(TOTAL_DATA // CHUNK):
        stream_py.write(data)
        stream_py.seek(0)
        _ = stream_py.read(CHUNK)
        # Note: Purging BytesIO is complex/slow, usually people just let it grow or reset it
    t_py = time.perf_counter() - start
    m_end = get_mem()
    print(f"Python BytesIO:  {t_py:.4f} s | Buffer RAM: {m_end - m_start:.2f} MB (Unstable)")

import io
def io_bytes():
    return io.BytesIO()

if __name__ == "__main__":
    print("RUNNING ADVANCED SCENARIOS (DIVERGENCE TESTS)")
    scenario_memory_fragmentation()
    scenario_massive_wrap_around()
    scenario_io_streaming()
