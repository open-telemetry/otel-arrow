# Filelog pattern benchmarks

Measures regex matching speed and memory use. It does not measure file reading,
decoding, multiline grouping, or the complete receiver.

## Run

From `rust/otap-dataflow`, run timings and allocation checks separately:

```bash
cargo bench --locked -p otel-arrow-dfe-core-nodes --features bench \
  --bench filelog_multiline
cargo bench --locked -p otel-arrow-dfe-core-nodes --features bench \
  --bench filelog_multiline_allocations
```

The first command prints timings. The second measures allocations with DHAT
and checks that matching memory stays within the worker estimate.

### Options

Set these environment variables before running either command:

| Variable | Purpose |
| --- | --- |
| `FILELOG_BENCH_BYTES` | Input size in bytes; default 1 MiB, minimum 128 |
| `FILELOG_BENCH_SAMPLES` | Timing samples per case; default 7 |
| `FILELOG_BENCH_SEARCHES` | Searches per allocation measurement; default 3 |
| `FILELOG_BENCH_VARIANTS` | Comma-separated engine variants |
| `FILELOG_BENCH_CASES` | Comma-separated test cases |
| `FILELOG_BENCH_LARGE` | Set to include the slower stress cases |

The default run covers all variants. `meta` uses the default matching policy;
`meta_64k` and `meta_256k` use smaller caches. `pike_prefilter` disables lazy
acceleration but keeps literal prefilters. Other variants and case names are
listed in [cases.rs](cases.rs).

Large cases can take seconds per search. Start with one sample and a selected
variant, using an external timeout. A timed-out run is not a completed result.

## Linux results (2026-10-09)

All inputs below are nonmatches. Cold uses a fresh matcher cache; warm reuses
one after an untimed search. Timings exclude input preparation, UTF-8 validation,
and cache creation. `us` means microseconds.

| Case | Variant | Input | Samples | Cold | Warm |
| --- | --- | ---: | ---: | ---: | ---: |
| Timestamp | `meta` | 128 B | 7 | 0.20 us | 0.037 us |
| Required literal absent | `meta` | 1 MiB | 7 | 34.3 us | 32.0 us |
| Plausible literal candidates | `meta` | 1 MiB | 7 | 1.66 ms | 1.61 ms |
| Cache churn (raw) | `meta` | 1 MiB | 7 | 167 ms | 141 ms |
| Large program | `meta` | 1 MiB | 1 | 32.7 ms | 31.5 ms |
| Repetition limit | `meta` | 1 MiB | 1 | 4.72 s | 4.67 s |
| Branching pattern | `meta` | 1 MiB | 1 | 14.7 ms | 1.79 ms |
| Branching pattern | `pike_prefilter` | 1 MiB | 1 | 19.09 s | 19.07 s |

Seven-sample rows show medians. The other rows are single observations.
The large-program case uses NFA fallback under the default policy. The final
row forces NFA execution; it is not the default policy's performance.

**Setup:** Azure Linux VM, Intel Xeon 6973P-C, pinned to one logical CPU.
Memory caps were 32 MiB for the first four rows and 128 MiB for the remaining
rows, with swap disabled. Rust 1.98.1, regex-automata 0.4.18, regex-syntax
0.8.11; isolated release build, optimization level 3, fat LTO, one codegen unit.
Matching code: [commit 727113330][measured-source].

A separate allocation run of the large-program case measured cache creation
and one search:

| Peak matching memory | Worker memory estimate |
| ---: | ---: |
| 9,600,496 bytes | 9,602,136 bytes |

This excludes the compiled pattern and other process memory.

These are measurements for specific inputs and hardware, not speed or memory
guarantees for the full receiver. Keep detailed output with the PR and update
this table when the matching code or benchmark changes.

[measured-source]: https://github.com/lalitb/otel-arrow/commit/727113330e4110a44d2ee7697d059430c03db887
