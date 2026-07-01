<!-- markdownlint-disable MD013 -->

# `otap_parquet` benchmark: OTAP/IPC vs flattened Parquet

**Status:** experiment on branch `jmacd/parquet_study`.

This benchmark quantifies moving OTAP logs from a client to a server that
ultimately stores flattened, single-file Parquet. Because the data ends up as
Parquet on the server either way, the central question is where the OTAP to
Parquet conversion CPU is spent. Option A has the client send OTAP as Arrow IPC
and the server convert it to Parquet, paying for IPC decode plus flatten plus
Parquet encode. Option B has the client precompute the flattened Parquet and
send that, so the server only persists the bytes, or reparses them into Arrow if
it must partition, index, or validate.

The benchmark measures write time, read time, serialized size, and the
server-side conversion cost for the same logs batch across four representations
and four compressors.

## Contenders

- `ipc` is the OTAP representation we have today: interleaved Arrow IPC streams
  produced by `Producer` and consumed by `Consumer`, with each per-payload
  stream compressed.
- `parquet-nested` is a single flattened Parquet file where each log row carries
  its denormalized resource, scope, and log attributes as
  `List<Struct{key,type,str,int,double,bool,bytes,ser}>` columns.
- `parquet-map` is the same, with attributes stored as
  `Map<Utf8, Struct{type,...}>`.
- `parquet-wide` is the analytics-flat extreme, where every distinct attribute
  key becomes its own typed top-level column named `resource.<key>`,
  `scope.<key>`, or `log.<key>`.
- `vortex` applies the same nested flattening but stores the result in a
  [Vortex](https://vortex.dev) file instead of Parquet. It requires the `vortex`
  cargo feature, which is a heavy dependency. Vortex applies its own cascading
  compression, so it exposes a single setting reported as `none`.

## Compressors

Compressors are explicit codecs so `zstd` can be compared head-to-head with
`lz4`. This matters for cross-language consumers, because some Arrow and Parquet
stacks, for example certain .NET builds, may not support `zstd`. In that case
`lz4`, and `snappy` for Parquet, need first-class numbers.

| compressor | Arrow IPC     | Parquet      |
|------------|---------------|--------------|
| `zstd`     | `ZSTD`        | `ZSTD`       |
| `lz4`      | `LZ4_FRAME`   | `LZ4_RAW`    |
| `snappy`   | *unsupported* | `SNAPPY`     |
| `none`     | uncompressed  | uncompressed |

Arrow IPC only supports `zstd` and `lz4`, so `snappy` is offered for the Parquet
schemes only. Parquet uses `LZ4_RAW`, the cross-language interoperable variant,
rather than the deprecated Hadoop-framed `LZ4`.

## Running

```bash
cargo bench -p benchmarks --bench otap_parquet
```

Two tables, one for serialized size and one for the server-side CPU model, are
printed to stdout before the timed benchmarks run. For a quick pass:

```bash
cargo bench -p benchmarks --bench otap_parquet -- \
  --warm-up-time 0.3 --measurement-time 0.6 --sample-size 10
```

To include the Vortex contender:

```bash
cargo bench -p benchmarks --bench otap_parquet --features vortex
```

## What each measurement covers

- IPC write runs `Producer::produce_bar` and prost-encodes the
  `BatchArrowRecords`.
- IPC read runs prost-decode, then `Consumer::consume_bar`,
  `from_record_messages`, and `decode_transport_optimized_ids` back to
  `OtapArrowRecords`.
- Parquet write flattens `OtapArrowRecords` to one Arrow `RecordBatch`, then runs
  `ArrowWriter` to a `Vec<u8>`.
- Parquet read runs `ParquetRecordBatchReader` to an Arrow `RecordBatch`, then
  unflattens back to `OtapArrowRecords`.
- The server_cost `convert-A` measurement takes IPC bytes through decode,
  flatten, and Parquet encode. This is the cost the server pays in Option A.
- The server_cost `accept-B` measurement takes Parquet bytes back to an Arrow
  `RecordBatch` without rebuilding OTAP. This is what the server pays in Option B
  when it must touch the data. If the server only persists the received bytes,
  its conversion CPU is roughly zero.

The flattened Parquet layouts keep the entire root `Logs` record batch intact, so
decode carries its scalar and struct columns straight back just as the IPC path
does, and the comparison is not penalized by re-walking the body. Only the
attribute tables are denormalized and rebuilt. Resource and scope attribute sets
are re-normalized on decode using the `resource.id` and `scope.id` join keys the
`Logs` batch still carries. `parquet-wide` is lossless for type-consistent keys.
Any attribute that does not fit its single typed column spills into a per-group
`List<Struct>` overflow column, so the round-trip stays exact.

## Illustrative results

These numbers come from one development machine running WSL with jemalloc.
Absolute values vary by host, but the relationships are stable. Shape
`r1_s1_l5000` is 5000 log records under a single resource and scope, and its OTLP
protobuf encoding is 1,135,223 bytes.

Serialized size in bytes:

| contender      | zstd   | lz4    | snappy | none      |
|----------------|--------|--------|--------|-----------|
| ipc            | 53,614 | 63,598 | n/a    | 1,236,336 |
| parquet-nested | 35,634 | 44,904 | 52,471 | 223,441   |
| parquet-map    | 35,946 | 45,216 | 52,783 | 223,753   |
| parquet-wide   | 40,524 | 48,907 | 48,939 | 50,532    |

Write and read time for shape `r1_s1_l5000` with `zstd`:

| contender      | write    | read    |
|----------------|----------|---------|
| ipc            | 1.31 ms  | 0.69 ms |
| parquet-nested | 13.88 ms | 8.00 ms |
| parquet-map    | 16.26 ms | 8.73 ms |
| parquet-wide   | 9.14 ms  | 6.37 ms |

Server-side conversion cost for shape `r1_s1_l5000`, in indicative
milliseconds. Column A is the server converting received OTAP/IPC, column B is
the server reparsing client Parquet, and the last column is the server CPU saved
when the server simply stores client Parquet.

| flatten / comp        | A convert | B reparse | saved on store |
|-----------------------|-----------|-----------|----------------|
| parquet-nested / zstd | 14.46 ms  | 2.88 ms   | 14.46 ms       |
| parquet-nested / lz4  | 13.49 ms  | 2.80 ms   | 13.49 ms       |
| parquet-wide / lz4    | 9.95 ms   | 1.19 ms   | 9.95 ms        |

## Takeaways

- The conversion is the expensive part. Flatten plus Parquet encode dominates,
  and IPC decode is a small fraction. Accepting client-precomputed Parquet and
  persisting it removes essentially all of that server CPU, which is roughly 10
  to 14 ms per 5000-log batch, an 80 to 100 percent reduction versus converting
  server-side. Even a server that must reparse the Parquet into Arrow still
  avoids the flatten and encode and saves about 80 percent.
- On zstd versus lz4, IPC with lz4 is about 19 percent larger than zstd, and
  Parquet-nested with lz4 is about 26 percent larger while snappy is about 47
  percent larger than zstd. lz4 encode and decode is a touch cheaper on CPU, and
  `parquet-nested/lz4` at 44.9 KB is still smaller than `ipc/zstd` at 53.6 KB, so
  a client that cannot use zstd, such as some .NET stacks, can use lz4 or snappy
  and remain competitive on the wire.
- On layout, `parquet-nested` and `parquet-map` compress smallest with zstd,
  while `parquet-wide` is far smaller uncompressed and is the cheapest to write
  and to reparse thanks to typed columns, at a modest size premium under zstd.
- For the debate, if server CPU is the constraint, and here it is, have clients
  send precomputed flattened Parquet and let the server persist it. lz4 is a safe
  cross-language codec choice with a small size cost relative to zstd.

## Vortex

Vortex is a next-generation, Arrow-native columnar file format. Vortex 0.75 pins
the same arrow-rs 58.3 as this workspace, so Arrow arrays flow in and out with no
version bridging, and the whole round-trip runs in memory over a `Vec<u8>` write
sink and a `ByteBuffer` reader. The contender reuses the nested flattening and
feeds the flat `RecordBatch` to Vortex.

Integration notes:

- Vortex 0.75 has no `FixedSizeBinary` encoding, so `trace_id` and `span_id` are
  cast to `Binary` before writing and restored on read.
- Vortex prefers view and plain Arrow types on read, so decode targets an
  explicit plain schema, with dictionaries decoded and `FixedSizeBinary`
  restored, that the OTAP schema check accepts.
- Vortex applies its own cascading BtrBlocks compression, so there is no external
  compressor knob.

Illustrative results for shape `r1_s1_l5000`, which is 5000 log records:

| metric                    | vortex   | parquet-nested/zstd | ipc/zstd |
|---------------------------|----------|---------------------|----------|
| serialized size           | 107.0 KB | 35.6 KB             | 53.6 KB  |
| size vs OTLP proto        | 10.6x    | 31.9x               | 21.2x    |
| server convert IPC to file| ~510 ms  | ~14 ms              | n/a      |
| reparse file to Arrow     | ~11.9 ms | ~2.9 ms             | n/a      |

Vortex write time at smaller shapes is about 43 ms for 250 logs and about 101 ms
for 1000 logs.

The finding is that for this OTAP-logs full round-trip, Vortex at default
settings is both larger on the wire, roughly twice `ipc/zstd` and three times
`parquet-nested/zstd`, and much slower to write, by an order of magnitude or
more, than zstd-Parquet. It therefore does not help the goal of having the client
precompute so the server offloads CPU.

Several caveats make this a starting point rather than a verdict on Vortex. The
benchmark does full batch round-trips, whereas Vortex is designed for selective
reads with column and row-range projection, pushdown, and random access, none of
which this workload exercises. The default BtrBlocks compressor samples encodings
to favor decode speed and random access over write speed and maximum ratio, and
neither a lighter nor a heavier write strategy was explored. Each write and read
creates a fresh `VortexSession`, whereas a real deployment would reuse it,
although the dominant cost is the compression itself. Finally, the nested
`List<Struct>` attribute layout may not play to Vortex's strengths, and the wide
layout was not tried with Vortex. Tuning the Vortex write strategy and exercising
selective reads are the obvious next steps.

## Extending

Contenders are the `Scheme` enum and its `Codec` implementations in
`benchmarks/src/parquet_study`. Add a variant to include a contender everywhere.
Input shapes are defined in `input_shapes()` in `benches/otap_parquet/main.rs`.
The flatten and unflatten round-trips and the server-cost helpers have unit tests
runnable with `cargo test -p benchmarks --lib parquet_study`.
