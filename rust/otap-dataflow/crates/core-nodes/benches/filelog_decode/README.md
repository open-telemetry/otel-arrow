# Filelog decoder qualification

These are **decoder/event-consumer microbenchmarks**, not file I/O, framing,
OTAP construction, checkpoint or production receiver throughput measurements.
The [module contract][module] describes the source ownership and stopping rules.

## Reproduce

Commands assume `rust/otap-dataflow` as the working directory.

Run the normal workspace benchmark:

```sh
cargo bench -p otel-arrow-dfe-core-nodes --bench filelog_decode
```

For an isolated, inexpensive comparison with the unchanged prototype:

```sh
git fetch https://github.com/lalitb/otel-arrow.git feat/filelog-receiver-phase1
python3 crates/core-nodes/benches/filelog_decode/compare.py \
  /tmp/filelog-decoder-evidence --runs 3
```

The output directory must not exist and must be outside the repository.
Python 3.11+, Git and the workspace's Rust toolchain are required.
`compare.py` does not fetch, modify the implementation, create a workspace
member, or delete its results. It:

1. Reads prototype commit
   `86b4fb2e08cec44c3798241d440323d9ab949d22` through Git objects.
2. Preserves the prototype decoder source unchanged, supplying only its two
   option enums in a small benchmark-only wrapper.
3. Copies the current decoder and the same `cases.rs` consumer into that
   isolated directory.
4. Pins the direct dependency versions from the workspace lockfile, copies the
   pinned toolchain, and builds both variants with identical optimized settings.
5. Alternates prototype and candidate runs, retaining source snapshots,
   checksums, generated lockfile, metadata, logs and Criterion samples.
6. Writes `summary.csv` with all cases and median/min/max of the run means.

The isolated profile uses optimization level 3, thin LTO, one codegen unit and
no debug assertions. The normal workspace benchmark defaults to fat LTO, so
its results must not be mixed into this comparison without matching profiles.
No receiver or checkpoint crate is pulled into the isolated comparison.

For a different locally available baseline, pass `--prototype <commit>`.
Record that commit and rerun rather than relabeling existing measurements.

## Method and environment

Recorded comparison: three alternating run pairs, 89 cases per variant.
Each case uses Criterion 0.8.2 with 10 samples, 100 ms warm-up and a 300 ms
measurement target. Criterion may extend sampling for slower cases.
Both variants consume identical events, update the same checksum/counters,
and use `std::hint::black_box`. Corpus construction is outside timed work.

| Item | Recorded value |
| --- | --- |
| Rust | 1.98.0, `88d9e12ae`, LLVM 22.1.8 |
| Cargo | 1.98.0, `797e8a9bc` |
| Target / host | `x86_64-unknown-linux-gnu`, AMD EPYC 7763 virtual host, 16 logical CPUs |
| OS | Linux / WSL2, kernel 6.6.114.1, glibc 2.43 |
| Runtime helper | `thiserror` 2.0.20 |
| Allocation measurement | Existing workspace `dhat` 0.3.3, separate from timing |
| CPU isolation | No affinity pinning, dedicated host or frequency control |

The candidate decoder source SHA-256 was:

```text
78fc14daddfae26e1b5769c9fa1394569423184bf1dfe6d7042cfe277d6cd565
```

The shared benchmark cases SHA-256 was:

```text
eddf8c0bd8389d7db371a602da8324316365fe046bcf0e271fcb3fd65244ac29
```

Each synthetic corpus is approximately 64 KiB, rounded to a whole pattern;
one corpus adds an initial UTF-8 BOM. Input chunks are 1, 3, 17 and 65,536
bytes. The small odd sizes split BOMs, UTF-8 prefixes, UTF-16 code units and
surrogate pairs. The largest size represents a bounded source turn, not a
required receiver configuration.

| Corpus | Contents |
| --- | --- |
| ASCII-heavy UTF-8 / ASCII | Timestamp-shaped ASCII log text, fields and CRLF; no semantic parsing |
| Multibyte UTF-8 / BOM | ASCII plus U+6771, U+4EAC, U+20AC and U+1F600 |
| UTF-16 BMP, both orders | ASCII, CJK, euro, CR and LF |
| UTF-16 supplementary, both orders | U+1F600, U+1F680, U+10000, U+10FFFF and LF |
| Raw | Repeated bytes 0 through 255 |
| Malformed UTF-8 | Valid prefix, interrupted sequence, surrogate encoding and out-of-range scalar bytes |
| Malformed ASCII | Valid prefix and bytes above `0x7f` |
| Malformed UTF-16, both orders | Valid prefix, high surrogate followed by `A`, and lone low surrogate |

The malformed corpora run under all three policies. Under `fail`, timing ends
at the first error: only four source bytes are consumed for UTF-8/ASCII, and
ten for UTF-16 including required lookahead. Those cases measure failure
latency, **not throughput over the offered 64 KiB**.

All normal timed scans discard payloads after consuming their events and
counting malformed units. This also exercises the validation-only/discard
use case without retaining a tail or materializing a string/record.

## Results

[results.csv](results.csv) contains all 178 implementation/case rows, including
every chunk size and policy. Values below use the median of three run-mean
latencies, converted to MiB/s. Brackets show the slowest and fastest run-mean
throughputs, not a confidence interval. Per-run statistical intervals remain
in the generated Criterion evidence.

### Clean input, 65,536-byte chunks

| Corpus | Prototype MiB/s [range] | Candidate MiB/s [range] |
| --- | --- | --- |
| ASCII-heavy UTF-8 | 32.7 [31.8-32.9] | 96.1 [91.6-97.9] |
| ASCII | 57.1 [55.3-58.0] | 92.7 [87.9-93.7] |
| Multibyte UTF-8 | 37.5 [33.8-38.5] | 71.4 [68.8-73.7] |
| UTF-8 with BOM | 37.5 [36.5-37.9] | 72.8 [69.4-73.5] |
| UTF-16LE BMP | 87.9 [84.6-88.3] | 95.5 [94.6-95.6] |
| UTF-16BE BMP | 87.9 [85.3-87.9] | 92.1 [86.1-97.8] |
| UTF-16LE supplementary | 140.6 [140.1-144.3] | 137.7 [124.7-137.9] |
| UTF-16BE supplementary | 140.2 [136.7-143.8] | 133.6 [120.2-138.7] |
| Raw | 56.9 [55.8-57.5] | 93.5 [80.2-96.1] |

The clean UTF-8 improvement is substantial, but it is not universal:
supplementary UTF-16 medians are about 2-5% lower at this chunk size.
The change adds explicit failure-state handling and prioritizes ordinary
UTF-8/ASCII input without replacing the tested UTF-16 decoder.
Short, virtualized-host measurements cannot separate all small regressions
from scheduling effects.

### Chunk sensitivity, ASCII-heavy UTF-8

| Chunk bytes | Prototype MiB/s | Candidate MiB/s |
| --- | --- | --- |
| 1 | 21.4 | 42.2 |
| 3 | 27.8 | 68.4 |
| 17 | 31.1 | 89.4 |
| 65,536 | 32.7 | 96.1 |

One-byte calls include repeated empty-input draining and magnify per-call
overhead. They are not a proxy for ordinary large-read receiver throughput.

### Malformed-heavy input, 65,536-byte chunks

| Encoding / policy | Prototype MiB/s [range] | Candidate MiB/s [range] |
| --- | --- | --- |
| UTF-8 / preserve | 29.7 [28.2-29.8] | 39.6 [34.4-39.7] |
| UTF-8 / replace | 29.0 [27.5-29.6] | 38.7 [38.5-40.1] |
| ASCII / preserve | 56.9 [44.4-57.4] | 77.6 [76.4-79.3] |
| ASCII / replace | 57.1 [57.0-57.9] | 76.3 [71.2-79.1] |
| UTF-16LE / preserve | 86.6 [84.2-87.2] | 90.5 [87.7-92.8] |
| UTF-16LE / replace | 84.5 [83.6-85.7] | 91.6 [84.9-92.7] |
| UTF-16BE / preserve | 83.9 [82.4-86.2] | 90.9 [86.0-92.4] |
| UTF-16BE / replace | 84.0 [78.8-85.4] | 91.8 [91.3-92.8] |

| Fail-policy encoding | Prototype ns/failure [range] | Candidate ns/failure [range] |
| --- | --- | --- |
| UTF-8 | 157.8 [155.1-162.6] | 119.5 [118.1-120.4] |
| ASCII | 93.0 [91.5-96.4] | 79.7 [78.8-81.7] |
| UTF-16LE | 112.8 [112.6-115.5] | 112.8 [111.7-120.9] |
| UTF-16BE | 116.3 [114.2-116.9] | 116.2 [115.1-117.5] |

## Bulk-output investigation and decision

The prototype baseline was measured before optimization. Merely adding a
UTF-8 ASCII shortcut inside its general cursor path did not offset the hardened
wrapper's cost. Inlining the successful wrapper, keeping failure handling cold,
and directly emitting complete one-byte units in `next` produced the retained
fast path. It does not read past a returned unit or allocate output storage.

A separate bounded caller-output experiment batches existing events into
reused inline slots before running the **same** consumer over identical
ASCII input. Capacity zero in the results denotes direct, unbuffered event
consumption. Counters/checksums are compared before timing.

| Caller batch capacity | Prototype MiB/s | Candidate MiB/s |
| --- | --- | --- |
| Direct (no batch) | 32.2 | 100.5 |
| 1 event | 19.3 | 44.6 |
| 4 events | 22.2 | 81.1 |
| 16 events | 21.1 | 76.3 |
| 64 events | 22.3 | 76.0 |

That experiment does not justify adding a batch-output API. It introduces
extra stores and delays the consumer's decision until its chosen batch end.
The experiment elects to stop only at that budget; a framer needing an earlier
boundary cannot use it.

This is **not** evidence that every possible borrowed-span or specialized bulk
decoder would be slower. Such a path would need a concrete consumer contract
that prevents prevalidation/error processing past its safe stopping boundary.
No uncontrolled read-ahead, retained event list, unsafe code or SIMD was added.
The shipping decoder retains the one-event interface and its fixed output slot;
output-capacity exhaustion tests do not apply to that interface.

## Allocation and retained-state evidence

Run the isolated allocation test separately from throughput:

```sh
cargo test -p otel-arrow-dfe-core-nodes --release \
  --test filelog_decoder_allocations
```

It uses one test in its own process, the existing `dhat::Alloc`, reused
4 KiB scratch, bytewise and full-chunk scans, all encodings and policies,
explicit incomplete-BOM completion and repeated terminal operations.
Observed decoder allocation is **0 blocks / 0 bytes**, including steady-state
discarding. A black-boxed 16-byte allocation afterward must be counted, so
forgetting to enable the allocator cannot produce a vacuous pass.
Caller corpus creation and Criterion allocations are not part of this count.

| Layout, x86-64 Rust 1.98.0 | Prototype bytes | Candidate bytes |
| --- | --- | --- |
| Decoder | 152 | 176 |
| Source evidence | 5 | 5 |
| Event | 32 | 32 |
| Step / returned result | 40 | 40 |
| Error reason | 24 | 24 |
| Failure with consumption | Not provided | 32 |

There are no decoder heap capacities. Healthy pending source is at most three
distinct bytes; fatal lookahead can make it four. Object layout includes
inactive storage, metadata and padding; it is not a logical retained-work or
RSS measurement. A 4 MiB reused-buffer unit test also verifies non-growing
pending state and bounded progress on every call.

## Coverage and limitations

The focused correctness run includes 41 unit tests plus the isolated allocation
test. Its unit-test portion completed locally in about 2.25 seconds, including
the complete Unicode scalar domain and 9,600 seeded arbitrary-stream decodes.
The seeded run is not coverage-guided fuzzing, and its short duration does not
establish arbitrary-input safety. Small exhaustive partitions, independent
malformed vectors and a separate UTF-16 oracle provide additional evidence.

The benchmark host is virtualized, shared, unpinned and has uncontrolled
frequency/scheduling. Three short runs are limited evidence; some cases show
substantial variability. Synthetic mostly repeated data has different cache
behavior from real files. These numbers include the event consumer but exclude
framing, raw-record retention, OTAP construction, file I/O, backpressure,
descriptor churn, checkpointing and process-memory pressure.

No production, zero-overhead, cross-platform, ARM or end-to-end throughput
claim follows from this measurement. Qualify the integrated framer/receiver
again with its actual stop decisions, output copies and resource bounds.

[module]: ../../src/receivers/filelog_receiver/README.md
