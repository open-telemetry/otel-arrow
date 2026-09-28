# Filelog framing benchmarks

These benchmarks measure source decoding plus physical-line framing. They exclude
file I/O, discovery, multiline grouping, OTAP construction, downstream delivery,
and checkpoints. They establish an initial baseline, not a comparison with an
older framer or an end-to-end receiver throughput claim.

## Reproduce

Run from `rust/otap-dataflow` with the repository-pinned toolchain:

```sh
cargo bench --locked -p otel-arrow-dfe-core-nodes --bench filelog_framing
cargo bench --locked -p otel-arrow-dfe-core-nodes --bench filelog_framing_allocations
```

The first command uses Criterion with the normal allocator. The second uses
DHAT in a separate executable and prints CSV allocation measurements. Do not
use the instrumented allocation run for timing comparisons.

For a three-run summary (Linux CPU pinning is optional):

```sh
for run in run1 run2 run3; do
  taskset -c 0 cargo bench --locked -p otel-arrow-dfe-core-nodes \
    --bench filelog_framing -- --noplot --save-baseline "$run"
done
cargo bench --locked -p otel-arrow-dfe-core-nodes \
  --bench filelog_framing_allocations > allocations.csv
python3 crates/core-nodes/benches/filelog_framing/summarize.py \
  target/criterion allocations.csv > summary.csv
```

The summary reports the median of three run means and their minimum/maximum.
Heap columns apply only to the measured 128-KiB chunk size. Keep the Criterion
samples, allocation CSV, source revision and machine details with the results.
Build before timing, and avoid running other builds or tests during measurement.

## Workloads and method

| Case | Source body | Policy | Body limit | Source lines |
| --- | --- | --- | --- | --- |
| `utf8_short_preserve` | 128 ASCII bytes | Preserve raw, split | 1 MiB | 2,048 |
| `utf8_short_replace` | 128 ASCII bytes | Replace, split | 1 MiB | 2,048 |
| `utf8_multibyte_preserve` | 16 repetitions of three 2/3/4-byte scalars | Preserve raw, split | 1 MiB | 2,048 |
| `utf16le_preserve` | 16 repetitions of ASCII, Greek and supplementary scalars | Preserve raw, split | 1 MiB | 2,048 |
| `utf8_malformed_preserve` | 128 bytes with one invalid byte | Preserve raw, split | 1 MiB | 2,048 |
| `utf8_oversize_split` | 16 KiB of ASCII | Preserve raw, split | 1 KiB | 16 |
| `utf8_oversize_truncate` | 16 KiB of ASCII | Preserve raw, truncate | 1 KiB | 16 |

All synthetic source lines end in LF. Fixture creation is outside the measured
interval. Each scan constructs a fresh framer, feeds the complete corpus, and
consumes and drops each output immediately, including any exact-source shadow.
The consumer uses `black_box` without hashing or copying output. Expected frame
counts and total body lengths are checked before timing each case.

Timing uses 17-byte and 128-KiB input chunks, 20 samples, a 200-ms warm-up and a
one-second measurement target. Small chunks exercise boundaries; 128 KiB
represents a source turn. Criterion can extend the measurement time to collect
enough samples. Framer construction and output allocation/deallocation are
included. No timing-based pass/fail thresholds are imposed.

Throughput counts original source bytes, including LF. Records/s means emitted
frames/s, so a split line contributes multiple frames. Allocation events include
reallocations; they are reported per emitted frame. DHAT peak live bytes covers
framer and briefly live output allocations, excludes the prebuilt corpus, and is
not process RSS or a complete receiver memory bound. Allocation measurement uses
128-KiB chunks. UTF-16 decoded text and source shadows have different lengths.

The allocation harness uses the same fixtures and consumer as the timing harness.
The existing `filelog_framer_allocations` test separately checks that scanning a
long truncate tail does not allocate after its prefix buffers have filled.

## Recorded results

See [RESULTS.md](RESULTS.md) for the testvm measurements, validation limits,
and a results section suitable for the PR description.
