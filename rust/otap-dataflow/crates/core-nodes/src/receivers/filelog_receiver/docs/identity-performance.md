# Identity admission measurements

These are standalone component measurements, not receiver performance
qualification. The receiver worker adapter, real discovery preflight, downstream
Ack latency, production policy-aware opener and scheduling are not included.

## Work limits

An association step visits at most 64 records and compares at most 64 KiB of
prefix bytes. It can pause within a comparison. Waiting scans hold no source
handle or shared probe reservation. The caller must service other work between
steps. The limits bound algorithmic work, not syscall or wall-clock duration.

Total worst-case association work remains quadratic when admitting a population
of unrelated files. This change provides scheduling opportunities; it does not
add a fingerprint index or change short-prefix matching semantics.

## Reproduce

Run on Linux, from `rust/otap-dataflow`:

```sh
cargo test -p otel-arrow-dfe-core-nodes --bench filelog_identity
FILELOG_IDENTITY_MODE=bulk cargo test -p otel-arrow-dfe-core-nodes --bench filelog_identity
FILELOG_IDENTITY_HEAP=1 cargo test -p otel-arrow-dfe-core-nodes --bench filelog_identity
FILELOG_IDENTITY_MODE=bulk FILELOG_IDENTITY_HEAP=1 cargo test -p otel-arrow-dfe-core-nodes --bench filelog_identity
```

Use `cargo bench` instead of `cargo test` for optimized measurements. Do not
compare results from different build profiles as a before/after comparison.
`FILELOG_IDENTITY_FILES` and `FILELOG_IDENTITY_WIDTH` override the fixture size;
configuration validation still applies.

Fixtures use 1,000-byte prefixes differing in their final eight bytes. Bulk
mode starts with an empty checkpoint and admits 10,000 distinct files in groups,
retaining their reader seeds. Churn mode makes 100 unmatched attempts against
10,000 saved records and discards each prepared plan without committing it.
Both modes reopen real files to validate evidence after association completes.
Sources are just-written local files with warm page-cache contents; this does
not measure cold-storage latency.

Heap profiling runs separately from timing. Fixture construction precedes the
profiler; measured allocations include store recovery, the authoritative table,
leases, scans, plans, and bulk reader seeds. The recovery input and temporary
allocations contribute to peak memory. This is component heap, not process RSS
or whole-receiver memory.

## Local Linux sample, 2026-10-28

Environment: Colima Linux ARM64, 4 CPUs, 10 GiB RAM, Linux-native Docker volume,
Rust 1.98.1. Commands above used the unoptimized test profile, one build job,
no incremental compilation and disabled debug symbols. Each row is one run;
maxima can include host scheduling delays and are not latency promises.

| Timing measurement | Churn | Bulk admission |
| --- | ---: | ---: |
| Candidate attempts | 100 | 10,000 |
| Total admission time | 75 ms | 6,894 ms |
| Record visits | 1,000,000 | 49,790,076 |
| Charged prefix bytes | 1,000,000,000 | 49,790,076,000 |
| Association steps | 15,700 | 782,752 |
| Longest association step | 39 us | 879 us |
| Longest initial preparation | 219 us | 421 us |
| Longest finishing preparation | 184 us | 346 us |
| Longest group commit | Not performed | 8,624 us |

| Separate heap measurement | Churn | Bulk admission |
| --- | ---: | ---: |
| Retained bytes before owner/reader teardown | 15,648,590 | 28,907,094 |
| Peak bytes, including recovery | 30,525,903 | 29,104,962 |
| Total allocated bytes | 37,537,936 | 249,873,126 |
| Allocation events | 20,968 | 211,588 |

The bulk result makes the remaining total work visible: roughly 50 billion
charged prefix bytes. Grouping reduces transaction count but does not remove
that association cost. The longest group commit also illustrates why the
worker's eventual scheduling qualification must cover encoding, replay and
fsync in addition to association steps.

Deterministic unit tests verify work limits, checkpoint progress between steps,
probe availability, partial comparisons, short-prefix association, cancellation,
source revalidation, and stale cursors after metadata changes or recovery. They
do not use timing thresholds. Integrated worker/control latency and memory
accounting remain separate qualification work.
