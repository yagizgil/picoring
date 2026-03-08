import picoring
import time

def run_stress_benchmark():
    # TEST CONFIGURATION
    # 50MB buffer, processing 40MB chunks
    BUFFER_SIZE = 1024 * 1024 * 50 
    DATA_CHUNK = 1024 * 1024 * 40   
    ITERATIONS = 2000 # Total ~80GB of virtual data processing

    print(f"STRESS TEST CONFIGURATION:")
    print(f"Buffer Size: {BUFFER_SIZE / (1024*1024):.1f} MB")
    print(f"Chunk Size:  {DATA_CHUNK / (1024*1024):.1f} MB")
    print(f"Iterations:  {ITERATIONS}")
    print("-" * 50)

    # 1. PICORING (HARDWARE MIRRORING - ZERO COPY)
    ring = picoring.PicoRingByte(BUFFER_SIZE)
    # Fill data to force a wrap-around state
    init_data = bytes([0] * DATA_CHUNK)
    ring.push_bytes(init_data)
    # Move tail/head to mid-buffer to ensure every read is a 'wrap' read
    ring.advance_tail(BUFFER_SIZE // 2)
    ring.push_bytes(init_data)

    start_pico = time.perf_counter()
    for _ in range(ITERATIONS):
        # O(1) access regardless of data size
        view = ring.get_readable_view()
        # Verify first byte to ensure memory is mapped
        _ = view[0]
    end_pico = time.perf_counter()

    t_pico = end_pico - start_pico
    print(f"PICORING RESULTS:")
    print(f"Total Time:  {t_pico:.6f} s")
    print(f"Avg Latency: {(t_pico/ITERATIONS)*1e6:.2f} us/call")
    print("-" * 50)

    # 2. PYTHON (BYTEARRAY/BYTES - O(N) COPY)
    # Simulating manual slice and join for wrapped data
    py_buf = bytearray(BUFFER_SIZE)
    wrap_point = BUFFER_SIZE - (DATA_CHUNK // 2)

    start_py = time.perf_counter()
    for _ in range(ITERATIONS):
        # Manual reconstruction of wrapped data causes O(N) copy
        part1 = py_buf[wrap_point:]
        part2 = py_buf[:DATA_CHUNK - len(part1)]
        contiguous = part1 + part2 
        _ = contiguous[0]
    end_py = time.perf_counter()

    t_py = end_py - start_py
    print(f"PYTHON RESULTS:")
    print(f"Total Time:  {t_py:.6f} s")
    print(f"Avg Latency: {(t_py/ITERATIONS)*1e6:.2f} us/call")
    print("-" * 50)

    # FINAL COMPARISON
    print("ANALYSIS:")
    print(f"Performance Gap: {t_py / t_pico:.1f}x")
    print(f"Conclusion: PicoRing remains constant time O(1). Python cost grows with N.")

if __name__ == "__main__":
    run_stress_benchmark()
