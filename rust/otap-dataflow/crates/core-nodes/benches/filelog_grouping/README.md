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

## Preliminary macOS results (2026-10-10)

Source MiB/s; higher is better. "Before" replays every buffered line through the
decoder. "After" copies already-decoded lines when they fit, reserving space for
the line and separator together. Overflow still uses source-unit replay.

| Input | Newline | Start before | Start after | End before | End after |
| --- | ---: | ---: | ---: | ---: | ---: |
| 128-byte UTF-8, one line/record | 76.01 | 15.13 | 28.77 | 15.30 | 29.18 |
| 128-byte UTF-8, four lines/group | 78.52 | 15.35 | 28.73 | 15.17 | 28.51 |
| 16-KiB UTF-8, four lines/group | 86.79 | 15.83 | 30.48 | 15.67 | 30.60 |
| UTF-16LE, four 128-char lines | 112.86 | 26.73 | 52.92 | 26.91 | 52.92 |

Apple M4 Pro, macOS arm64, one thread, not CPU-pinned. Rust 1.98.1,
regex-automata 0.4.18; isolated release build with optimization level 3, fat LTO,
and one codegen unit. Medians of seven samples, eight scans per sample, with a
warm matcher cache and 128-KiB input chunks. Policy is `preserve_raw` with
`^START` / `END$` patterns; bounds are 64 KiB per line and 1 MiB per record.

Four-line cases produce fewer records than newline framing. The one-line case
keeps record counts equal. These are local comparisons, not full-receiver
throughput or Linux qualification results. Repeat on the Linux test VM before
using them for deployment sizing.
