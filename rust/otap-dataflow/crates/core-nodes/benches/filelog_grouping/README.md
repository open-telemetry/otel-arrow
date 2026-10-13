# Filelog grouping benchmark

Compares newline framing with start- and end-pattern grouping on the same input.
Includes decoding, matching, body allocation, and final group completion.
Excludes file I/O, input generation, and pattern compilation.

## Run

From `rust/otap-dataflow`:

```bash
cargo bench --locked -p otel-arrow-dfe-core-nodes --bench filelog_grouping
```

Output is CSV. Set `FILELOG_GROUPING_SAMPLES` (default 7) or
`FILELOG_GROUPING_REPEATS` (default 8) to change the run length. Each scan checks
its record count, body-byte count, and final source offset.

## macOS results (2026-10-13)

Measured revision: [`7f7429fdf`](https://github.com/lalitb/otel-arrow/commit/7f7429fdff3a2764ea00e0c04119911ac4eb3cc4),
including start-pattern precedence over record overflow.
Source MiB/s; higher is better.

| Input | Newline | Start grouping | End grouping |
| --- | ---: | ---: | ---: |
| 128-byte UTF-8, one line/record | 120.91 | 45.36 | 45.62 |
| 128-byte UTF-8, four lines/group | 120.09 | 44.94 | 45.19 |
| 16-KiB UTF-8, four lines/group | 133.31 | 46.96 | 46.96 |
| UTF-16LE, four 128-char lines | 168.42 | 81.71 | 81.96 |

Apple M4 Pro, macOS 26.7.1 arm64, one thread, not CPU-pinned. Rust 1.98.1,
regex-automata 0.4.18; workspace `bench` profile with optimization level 3, fat
LTO, and one codegen unit. The command above built and ran the benchmark; the
same executable was then run twice more. Each run used seven samples and eight
scans per sample, with a warm matcher cache and 128-KiB input chunks. The table
reports the median of the three per-run medians. Across all cases, the largest
spread between run medians was 1.83% of that case's reported median; this is a
repeatability observation, not a confidence interval.

Policy is `preserve_raw` with `^START` / `END$` patterns; bounds are 64 KiB per
line and 1 MiB per record. All record-count, body-byte-count, and final-offset
checks passed in all three runs.

These measurements replace the October 10 preliminary results. Both newline
and grouping throughput are higher in this run; the historical comparison is
not a controlled measurement of the start-boundary fix's performance effect.

Four-line cases produce fewer records than newline framing. The one-line case
keeps record counts equal. None of these inputs crosses a line or record bound,
so this benchmark does not measure overflow, split/truncate, or recovery costs.
These are local comparisons, not full-receiver throughput or Linux qualification
results. Repeat on Linux before using them for deployment sizing.
