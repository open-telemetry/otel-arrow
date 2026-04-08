# GzipBatcher Compression Benchmark

## Motivation

The `GzipBatcher` streams log entries into a gzip compressor and produces ~1 MB
compressed batches for upload. The gzip compression level directly affects CPU
cost per batch. This benchmark measures the trade-off between compression level,
throughput, and compression ratio to inform the default level choice.

## Methodology

**Compression levels tested:** 1 (fastest), 6 (default), and 9 (maximum).
Gzip levels range from 1-9, where 1 prioritizes speed and 9 prioritizes
compression ratio.

**Procedure:** For each combination of (data type, entry size, compression
level), the benchmark:

1. Pre-generates a pool of unique random entries of the given size and type.
2. Pushes entries into a fresh `GzipBatcher` until the ~1 MB batch threshold is
   reached (`BatchReady`).
3. Records the number of log records, uncompressed size, compressed size, and
   compression ratio.
4. Measures the wall-clock time to fill one complete batch using
   [Criterion](https://github.com/bheisler/criterion.rs) (100 samples per
   benchmark).

**Data profiles:**

Three data profiles cover a range of compressibility:

- **json_log**  --  Structured JSON with random field values and a random
  lowercase-letter message body. Repeating key names and JSON structure give
  deflate many matching opportunities, making this the most compressible
  profile and the closest to real Azure Monitor ingestion payloads.
- **hex**  --  Minimal JSON wrapper around a random hexadecimal string
  (`0-9a-f`). The lack of repeating structure means deflate has little to
  match, so compression ratio is lower despite the small character set.
- **ascii**  --  Minimal JSON wrapper around random printable ASCII
  (`0x20-0x7E`). Similar to hex but with a larger character set, this
  compresses the least.

**Entry sizes:**

Three entry sizes are tested to capture how per-entry overhead and compression
window utilization change with payload length:

- **256 B**  --  Small log entries. Typical of short structured events (e.g., a
  metric data point or a brief log line with metadata). At this size,
  per-entry framing overhead is proportionally higher.
- **512 B**  --  Medium log entries. Representative of a typical log record with
  a moderate-length message body and several attributes.
- **1024 B**  --  Large log entries. Represents verbose log records with stack
  traces, detailed error messages, or rich attribute sets. Larger entries give
  the compressor more context per push, which can improve throughput.

**Column definitions:**

| Column | Description |
| ------------ | --------------------------------------------------------- |
| Entry Size | Size of each uncompressed log entry |
| Level | Gzip compression level (1 = fastest, 9 = max compression) |
| Log Records | Number of entries that fit in one ~1 MB compressed batch |
| Uncompressed | Total raw size of all entries in the batch |
| Compressed | Resulting gzip-compressed batch size (~1 MB target) |
| Ratio | Compressed / Uncompressed (lower is better) |
| Time | Wall-clock time to fill one batch |
| Throughput | Uncompressed data processed per second |

## Results

### JSON Log Data

| Entry Size | Level | Log Records | Uncompressed | Compressed | Ratio | Time | Throughput |
| ---------- | ----- | ----------- | ------------ | ---------- | ----- | ---- | ---------- |
| 256 B | 1 | 9,016 | 2.20 MB | 1.00 MB | 45.2% | 12.3 ms | 179 MiB/s |
| 256 B | 6 | 9,093 | 2.22 MB | 1.00 MB | 44.9% | 30.3 ms | 73.3 MiB/s |
| 256 B | 9 | 9,094 | 2.22 MB | 1.00 MB | 44.9% | 32.0 ms | 69.4 MiB/s |
| 512 B | 1 | 3,802 | 1.86 MB | 1.00 MB | 53.6% | 8.6 ms | 216 MiB/s |
| 512 B | 6 | 3,692 | 1.80 MB | 1.00 MB | 55.2% | 28.2 ms | 63.9 MiB/s |
| 512 B | 9 | 3,691 | 1.80 MB | 1.00 MB | 55.2% | 29.9 ms | 60.2 MiB/s |
| 1024 B | 1 | 1,773 | 1.73 MB | 1.00 MB | 57.5% | 6.5 ms | 267 MiB/s |
| 1024 B | 6 | 1,695 | 1.66 MB | 1.00 MB | 60.1% | 23.9 ms | 69.3 MiB/s |
| 1024 B | 9 | 1,696 | 1.66 MB | 1.00 MB | 60.1% | 24.0 ms | 69.0 MiB/s |

## Hex Data

| Entry Size | Level | Log Records | Uncompressed | Compressed | Ratio | Time | Throughput |
| ---------- | ----- | ----------- | ------------ | ---------- | ----- | ---- | ---------- |
| 256 B | 1 | 7,591 | 1.85 MB | 1.00 MB | 53.7% | 12.9 ms | 144 MiB/s |
| 256 B | 6 | 7,233 | 1.77 MB | 1.00 MB | 56.4% | 32.2 ms | 54.8 MiB/s |
| 256 B | 9 | 7,234 | 1.77 MB | 1.00 MB | 56.4% | 32.3 ms | 54.7 MiB/s |
| 512 B | 1 | 3,767 | 1.84 MB | 1.00 MB | 54.1% | 10.3 ms | 179 MiB/s |
| 512 B | 6 | 3,592 | 1.75 MB | 1.00 MB | 56.8% | 30.4 ms | 57.6 MiB/s |
| 512 B | 9 | 3,592 | 1.75 MB | 1.00 MB | 56.8% | 30.9 ms | 56.7 MiB/s |
| 1024 B | 1 | 1,877 | 1.83 MB | 1.00 MB | 54.3% | 8.9 ms | 206 MiB/s |
| 1024 B | 6 | 1,791 | 1.75 MB | 1.00 MB | 56.9% | 30.2 ms | 57.9 MiB/s |
| 1024 B | 9 | 1,790 | 1.75 MB | 1.00 MB | 56.9% | 31.0 ms | 56.4 MiB/s |

## ASCII Data

| Entry Size | Level | Log Records | Uncompressed | Compressed | Ratio | Time | Throughput |
| ---------- | ----- | ----------- | ------------ | ---------- | ----- | ---- | ---------- |
| 256 B | 1 | 5,024 | 1.23 MB | 1.00 MB | 81.2% | 7.8 ms | 157 MiB/s |
| 256 B | 6 | 5,021 | 1.23 MB | 1.00 MB | 81.3% | 21.6 ms | 56.7 MiB/s |
| 256 B | 9 | 5,020 | 1.23 MB | 1.00 MB | 81.3% | 21.6 ms | 56.7 MiB/s |
| 512 B | 1 | 2,485 | 1.21 MB | 1.00 MB | 82.1% | 5.6 ms | 217 MiB/s |
| 512 B | 6 | 2,483 | 1.21 MB | 1.00 MB | 82.1% | 20.3 ms | 59.7 MiB/s |
| 512 B | 9 | 2,483 | 1.21 MB | 1.00 MB | 82.1% | 20.8 ms | 58.3 MiB/s |
| 1024 B | 1 | 1,236 | 1.21 MB | 1.00 MB | 82.5% | 4.5 ms | 269 MiB/s |
| 1024 B | 6 | 1,235 | 1.21 MB | 1.00 MB | 82.5% | 19.9 ms | 60.6 MiB/s |
| 1024 B | 9 | 1,235 | 1.21 MB | 1.00 MB | 82.5% | 19.8 ms | 60.9 MiB/s |

## Analysis

- **Level 1 vs 6:** Level 1 is 2.5-4.4x faster with only ~1-3 percentage points
  worse compression ratio.
- **Level 6 vs 9:** Virtually identical compression ratios across all data types,
  but level 9 is consistently 2-6% slower. Level 9 provides no measurable
  compression benefit over level 6.
- **Data type impact:** JSON logs compress best at small entry sizes (45% ratio
  at 256B) due to repeating key structure. Hex and json_log converge at larger
  entry sizes (~54-60% ratio). Random ASCII compresses the least (81-83%).
  Data type and entry size matter more than compression level.

## Caveats

- These benchmarks measure CPU-bound compression throughput in isolation.
  In practice, the exporter is often limited by outgoing HTTP request rate
  rather than compression speed.
- When the pipeline is I/O bound, higher compression levels pack more log
  records per batch, reducing the number of requests needed.
- Compression ratios were measured with synthetic data. Production payloads
  may compress differently depending on field cardinality, message repetition,
  and attribute diversity.
