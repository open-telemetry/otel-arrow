# Compaction allocation measurements

Compaction now borrows retained records, calculates their encoded size without
allocating, and writes directly into one fixed-size output buffer. It keeps
both uniqueness checks and all three validation stages. The snapshot format
is unchanged; compatibility tests compare against existing golden vectors and
bytes captured with the old encoder.

## Results

Measured on testvm, Linux x86_64, Rust 1.98.1, kernel `7.0.0-1019-azure`, with
sources and temporary checkpoints on native ext4 at `/mnt/persist`.
The baseline is commit `655fa30d64b4e278cb12f6e7ca7a4adfdda009fa`.
The optimized version is the bounded, borrowed-record compaction in this PR.
Both executables used the workspace's release profile and the same harness.

Each fixture starts with a populated snapshot and one deferred progress update.
The measured call is ordinary `compact()`, with no removals. It includes sizing,
encoding, all validation, file I/O, and all five file/directory sync barriers.
Fixture creation, store opening, and result checks are outside the call.

Values below are **before -> after**. MB means 1,000,000 bytes.

| Fixture | Allocation count | Allocated MB | Peak live MB |
| --- | --- | --- | --- |
| 10,000 records, 32-byte fingerprints, 32-byte paths | 240,131 -> 60,102 | 93.427 -> 55.609 | 21.361 -> 21.116 |
| 10,000 records, 32-byte fingerprints, 4 KiB paths | 280,132 -> 60,102 | 935.162 -> 258.809 | 171.921 -> 143.036 |
| 1,000 records, 4 KiB fingerprints, 4 KiB paths | 26,106 -> 6,084 | 173.626 -> 47.196 | 27.143 -> 26.857 |

Allocation results were identical across three fresh-process runs per version
and fixture. DHAT counts allocation/reallocation requests and requested bytes.
These are Rust-heap measurements, not RSS, allocator metadata, kernel cache, or
physical storage use. Profiling begins before store opening so the peak includes
the old live table. Counters are sampled immediately around `compact()`. The
harness checks that the recorded peak increased during that call; otherwise it
rejects the measurement rather than reporting a setup peak as compaction cost.

| Fixture | Median ms before | Median ms after | Before range ms | After range ms |
| --- | --- | --- | --- | --- |
| 10,000 records, short paths | 73.194 | 59.465 | 72.261-76.453 | 58.007-74.318 |
| 10,000 records, long paths | 852.924 | 741.456 | 833.495-895.107 | 733.283-756.362 |
| 1,000 records, larger fingerprints | 152.001 | 134.994 | 148.842-158.013 | 130.959-139.270 |

Timing used a separate executable with the system allocator and no profiler.
Each fixture had one warm-up per version, followed by 11 measured calls per
version in fresh processes. Before/after order alternated between rounds.
No builds or tests ran alongside the comparison. These local results do not
establish production latency or receiver throughput, and no test asserts a
timing threshold.

Fewer allocations do not guarantee the same reduction in peak memory. The
unchanged validation stages still allocate replay tables; their peak largely
limits the improvement for short paths and larger fingerprints.

Raw observations: [allocations](compaction-allocations.csv) and
[latency](compaction-latency.csv).

## Reproduce

Use the same source for both measurement examples on each revision. For the
baseline, copy the `examples` directory from this PR into a clean checkout of
`655fa30d6`; leave the baseline library source unchanged. Build both versions
with Rust 1.98.1 and preserve each pair of executables before switching source.

From `rust/otap-dataflow`, with `TMPDIR` on native Linux storage:

```sh
cargo build --release -p otel-arrow-dfe-filelog-checkpoint --examples
cargo run --release -p otel-arrow-dfe-filelog-checkpoint \
  --example compaction_allocations -- 10000 32 4096
cargo run --release -p otel-arrow-dfe-filelog-checkpoint \
  --example compaction_latency -- 10000 32 4096
```

Repeat with `10000 32 32` and `1000 4096 4096`. Alternate versions for each
fixture. Allocation output is CSV: record count, fingerprint bytes, path bytes,
allocation count, allocated bytes, initial live bytes, and peak live bytes.
Timing output is CSV: record count, fingerprint bytes, path bytes, and elapsed
nanoseconds. The examples use the existing DHAT and tempfile dependencies and
introduce no unsafe code.
