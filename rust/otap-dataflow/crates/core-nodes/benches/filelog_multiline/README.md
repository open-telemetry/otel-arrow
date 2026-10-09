# Filelog pattern qualification

This harness measures the pattern primitive, not the complete Filelog receiver.
It excludes decoding, line framing, source I/O, batching, and scheduling.

## Run

From the OTAP workspace:

```bash
cargo bench --locked -p otel-arrow-dfe-core-nodes --features bench \
  --bench filelog_multiline
cargo bench --locked -p otel-arrow-dfe-core-nodes --features bench \
  --bench filelog_multiline_allocations
```

Both targets include the exact source module to exercise qualification controls
that are absent from normal library builds. Their `bench` feature is required.
An isolated Cargo manifest may also point at these binaries, with a `bench`
feature and the same pinned regex versions, Rust toolchain, and optimization
settings. This avoids rebuilding unrelated receivers for matcher-only runs.

Timing and DHAT allocation runs are separate. Text input is validated once
before timing, matching production's already-valid `&str` contract. Cold timing
creates a fresh cache before each timed search; warm timing reuses a cache after
one untimed search. Input creation and cache construction are outside the timer.
The allocation run includes cache creation and repeated searches, excluding
fixture and output storage.

| Control | Meaning |
| --- | --- |
| `FILELOG_BENCH_BYTES` | Body size; default 1 MiB, minimum 128 bytes |
| `FILELOG_BENCH_SAMPLES` | Timing samples per cold/warm case; default 7 |
| `FILELOG_BENCH_SEARCHES` | Searches per allocation interval; default 3 |
| `FILELOG_BENCH_VARIANTS` | Comma-separated variants to run |
| `FILELOG_BENCH_CASES` | Comma-separated fixture names to run |
| `FILELOG_BENCH_LARGE` | Include large-program fixtures when set |
| `FILELOG_BENCH_SKIP_COMPILE_STRESS` | Skip construction stress when set |
| `FILELOG_BENCH_CGROUP_REPORT` | Linux output path for enforced cgroup limits |

## Execution variants

| Variant | Policy |
| --- | --- |
| `meta` | 2 MiB cache ceiling, prefilters, equivalent fallback |
| `meta_no_prefilter` | Same with automatic prefilters disabled |
| `meta_64k` / `meta_256k` | Smaller cache ceilings |
| `meta_backtrack` | Meta plus the bounded-backtracker candidate |
| `pike` | No lazy DFA or prefilter |
| `pike_prefilter` | No lazy DFA, automatic prefilters retained |
| `pike_backtrack` | No lazy DFA/prefilter, backtracker candidate enabled |

`pike_prefilter` measures execution without lazy acceleration but with
prefilters.
The no-prefilter PikeVM baseline must not be described as the performance of a
production fallback that retains prefilters. Backtracking remains a candidate,
not a default selected on the strength of a few short-line measurements.

## Fixtures

Ordinary cases cover a 128-byte timestamp nonmatch, maximum-length literal
nonmatches with absent/plausible literal candidates, and the anchored
`^[ab]*a[ab]{20}[ab]$` case over varied bytes. Its required suffix position is
forced to `b`; it cannot succeed after an early prefix. Raw and text versions
are both included.

Opt-in large fixtures are:

- `repetition_cap`: `^[ab]*a[ab]{1000}$`, with the required suffix position
  forced to fail on sufficiently long inputs.
- `large_program_search`: an anchored pattern with an unbounded prefix and
  a 200-byte literal repeated 1,000 times, followed by an absent terminator.
  Its unbounded prefix prevents a maximum-length shortcut. This fixture measures
  allocated fallback arrays during search.
- `branching_nonmatch`: `^a*(?:a|aa){1000}Z$` over `a` bytes, exercising many
  simultaneously active states without a usable prefilter.

Use external time limits and start with one sample for large cases. A timeout
is an incomplete measurement, never a nonmatch or a measured maximum. Static
program size alone does not predict latency; active states, input, and shortcuts
matter. Every completed fixture asserts its expected nonmatch result.

The allocation target also measures accepted, cache-capacity fallback, and
rejected construction, printing each execution policy. The capacity fixture
asserts that acceleration was skipped; it does not exercise reverse compilation.
The harness asserts measured search peak requested bytes do not exceed the
source-derived worker bound. This is regression evidence for the
model, not a proof from samples alone.

## Linux measurement snapshot (2026-10-09)

These exploratory measurements cover nonmatching inputs. They are not latency
limits or full-receiver throughput results. `us` means microseconds; cold/warm
refer to the matcher cache, not the operating system's file cache.

| Fixture | Variant | Body | Samples | Cold | Warm |
| --- | --- | ---: | ---: | ---: | ---: |
| Timestamp | `meta` | 128 B | 7 | 0.20 us | 0.037 us |
| Required literal absent | `meta` | 1 MiB | 7 | 34.3 us | 32.0 us |
| Plausible literal candidates | `meta` | 1 MiB | 7 | 1.66 ms | 1.61 ms |
| Cache churn (raw) | `meta` | 1 MiB | 7 | 167 ms | 141 ms |
| Large program | `meta` | 1 MiB | 1 | 32.7 ms | 31.5 ms |
| Repetition limit | `meta` | 1 MiB | 1 | 4.72 s | 4.67 s |
| Branching pattern | `meta` | 1 MiB | 1 | 14.7 ms | 1.79 ms |
| Branching pattern | `pike_prefilter` | 1 MiB | 1 | 19.09 s | 19.07 s |

Seven-sample rows report medians. Single-sample rows are observations only.
`meta` uses the default policy, including fallback; the large-program fixture
runs without lazy acceleration because its minimum cache exceeds the ceiling.
The last row explicitly disables lazy acceleration while retaining prefilters.
Its result must not be presented as default-policy performance.

Setup:

- Azure Linux VM, Intel Xeon 6973P-C; one logical CPU (CPU 2), 100% CPU quota.
- Cgroup memory cap: 32 MiB for the first four rows, 128 MiB for the large
  fixtures; swap disabled. These runs reported no cgroup OOM events.
- Rust 1.98.1, regex-automata 0.4.18, regex-syntax 0.8.11. Isolated release
  build with optimization level 3, fat LTO, and one codegen unit.
- Samples exclude input preparation, UTF-8 validation, and cache construction,
  as described above. The small sample counts do not characterize tail latency.

A separate DHAT run of the large-program fixture measured the following search
allocations, including cache creation and one search:

| Measured peak matching memory | Worker memory estimate |
| ---: | ---: |
| 9,600,496 bytes | 9,602,136 bytes |

This test stayed within the worker memory estimate. The numbers exclude the
compiled pattern and other process memory. One passing case does not cover
all patterns and inputs.

These measurements used the matching code in
[commit 727113330][measured-source].

[measured-source]: https://github.com/lalitb/otel-arrow/commit/727113330e4110a44d2ee7697d059430c03db887

## Evidence and interpretation

Keep CSVs, source hashes, cgroup reports, and detailed logs as review artifacts.
Re-run or clearly mark this snapshot as historical when the implementation or
timed region changes. Record the compiler, dependency lock, CPU, sample count,
and exact source hashes alongside each run.

On Linux, pin the process to one CPU and enforce memory/swap limits using cgroup
v2. Compare 16/32 MiB matcher-process runs, smaller eligible lines, and reduced
cache ceilings. Build binaries outside timing cgroups; compilation has separate
memory requirements. Inspect OOM and CPU-throttling counters.

Observed medians/maxima describe only the selected fixtures and hardware. Engine
memory reports can omit spare capacity; requested allocation, RSS, cgroup usage,
and modeled heap bounds are different measures. A successful 16 MiB matcher
process does not establish a 16 MiB full-receiver footprint. Receiver admission
integration and a numeric control-latency qualification target remain separate.
