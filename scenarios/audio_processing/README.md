# Scenario: Real-Time Audio Processing

In professional audio software (DAW, DSP), latency is the enemy. Standard ring buffers require copying data into a temporary buffer whenever the audio frame wraps around the end of the memory.

### How PicoRing solves it:

With hardware mirroring, an audio effect (like a Reverb or EQ) can always access a **contiguous slice** of the last N samples, even if they physically cross the boundary.

- **Benefit**: Zero-copy means no CPU cycles wasted on `memcpy`.
- **Latency**: Sub-microsecond access to circular data.
