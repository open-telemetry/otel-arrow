# Filelog framing benchmark results

Measured on testvm on 2026-09-28 against framer/decoder commit
`cfa9342d8dc5922cd1ca9c1e48fe5470ab3ac154`, with the benchmark additions in this
branch. The framer and decoder implementation were not changed for measurement.

## PR description

Added decoder-plus-physical-line-framer benchmarks covering short lines,
multibyte input, UTF-16, malformed input, and oversized split/truncate paths.

Results below use 128-KiB input chunks and the median of three Criterion run
means (20 samples per run, 200-ms warm-up, one-second measurement target).
Linux x86-64 testvm has four logical CPUs on an AMD EPYC 7763 host; timing was
pinned to guest CPU 0. Rust 1.98.1 / LLVM 22.1.8, Criterion 0.8.2, and the normal
workspace bench profile (opt-level 3, fat LTO, one codegen unit) were used.
The VM was not running builds or tests during timing. Host isolation and CPU
frequency were not controlled.

| Workload | Source MiB/s | Frames/s | Allocation events/frame | Peak live heap (bytes) |
| --- | ---: | ---: | ---: | ---: |
| Short UTF-8, preserve raw | 51.0 | 414,536 | 16.000 | 256 |
| Short UTF-8, replace | 68.6 | 557,666 | 8.000 | 128 |
| Multibyte UTF-8, preserve raw | 47.3 | 341,931 | 14.000 | 320 |
| UTF-16LE, preserve raw | 80.8 | 651,338 | 14.000 | 240 |
| Malformed UTF-8, preserve raw | 55.1 | 447,975 | 8.004 | 384 |
| Oversized UTF-8, split | 59.1 | 60,470 | 11.043 | 2,048 |
| Oversized UTF-8, truncate | 110.4 | 7,064 | 22.000 | 2,048 |

Allocation events include reallocations. Heap measurements use a separate DHAT
0.3.3 executable, so profiler overhead does not affect the throughput figures.
Each emitted frame and its source shadow are consumed and dropped immediately.
Peak live heap excludes the prebuilt input corpus and profiler bookkeeping; it
is not process RSS, a worst-case configured memory bound, or a full receiver
memory measurement.

Short lines have 128-byte bodies; the default preserve-raw path retains decoded
text and exact source bytes. Oversized cases use 16-KiB lines with a 1-KiB body
limit. Split throughput counts emitted fragments, while truncate intentionally
discards the tail. MiB/s counts source bytes processed, not bytes delivered.
See the [workload definitions](README.md#workloads-and-method).

This is an initial baseline. It excludes file I/O, discovery, multiline grouping,
OTAP batching, delivery and checkpointing, and does not claim an improvement over
an earlier implementation. The default short-line case's 16 allocation events
per frame identifies a useful target for later buffer-reuse work.

Validation on testvm:

- `cargo check --locked -p otel-arrow-dfe-core-nodes`: passed.
- Filelog decoder/framer unit tests: 59 passed.
- Framer allocation regression test: passed.
- Full core-node unit-test group during the workspace run: 1,208 passed,
  one ignored, zero failed.
- Both new benchmark targets: Clippy passed with warnings denied; optimized
  build passed; all three timing passes and the separate heap run passed their
  workload checks.
- `cargo xtask check`: structure, inventory, formatting, workspace Clippy and
  no_std checks passed. The workspace test run aborted in `otel-arrow-dfe-pdata`
  while requesting 16 GiB. The existing max-ID delta-encoding test supplies
  `u32::MAX`, and its remapping vector has 2^32 `u32` entries. This code is
  unchanged by the branch. The full workspace suite therefore did not pass.

## Reproduction and evidence

The [README](README.md#reproduce) contains the exact benchmark and summary
commands. [results.csv](results.csv) includes both 17-byte and 128-KiB chunks,
plus the minimum and maximum run means. [allocations.csv](allocations.csv)
contains the separate allocation measurements.

The raw Criterion samples, validation logs, machine details and source hashes
were retained at `/mnt/preserve/filelog-framing-bench-20260928` on testvm and
copied to `/tmp/filelog-framing-evidence-20260928` on the development machine.
The hashes below identify the measured benchmark sources, before the recorded
results were added to the module-level comment in `main.rs`. That later
documentation-only change did not alter executable benchmark code.

| Source | SHA-256 |
| --- | --- |
| `main.rs` | `0f5eb1844b7da2fddadc4849920ecf39462cf3ba46cf3ccd92bb095725ead5af` |
| `cases.rs` | `2acbada52a7281fa508d5e0c0d58fcf3da9ac6eee3a567a3b37d21bc8f8fbddc` |
| `allocations.rs` | `ef0f857a7ac8840c367817acbd41c41dc8bab0ce1db6670b5fbfb1984ec6f39e` |
