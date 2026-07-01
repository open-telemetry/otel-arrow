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
server-side conversion cost for the same logs batch across several
representations and compressors.

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
  [Vortex](https://vortex.dev) file with Vortex's default BtrBlocks compression.
- `vortex-fast` is the same Vortex layout written with BtrBlocks disabled, to
  test whether skipping compression lets Vortex write faster than Parquet. The
  Vortex contenders require the `vortex` cargo feature, which is a heavy
  dependency.

## Compressors

Compressors are explicit codecs so `zstd` can be compared head-to-head with
`lz4`. This matters for cross-language consumers, because some Arrow and Parquet
stacks, for example certain .NET builds, may not support `zstd`. In that case
`lz4`, and `snappy` for Parquet, need first-class numbers. The Vortex contenders
manage their own encoding, so they expose a single setting reported as `none`.

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
printed to stdout before the timed benchmarks run. To include the Vortex
contenders:

```bash
cargo bench -p benchmarks --bench otap_parquet --features vortex
```

The default input shapes are 10k and 100k log records, so the full Criterion
sweep is slow. The printed tables are the main output. For a quick pass, add
`-- --measurement-time 0.5 --sample-size 10`, or read the printed tables and
stop the run.

## What each measurement covers

- IPC write runs `Producer::produce_bar` and prost-encodes the
  `BatchArrowRecords`.
- IPC read runs prost-decode, then `Consumer::consume_bar`,
  `from_record_messages`, and `decode_transport_optimized_ids` back to
  `OtapArrowRecords`.
- File write flattens `OtapArrowRecords` to one Arrow `RecordBatch`, then encodes
  it to Parquet or Vortex bytes.
- File read decodes the file to an Arrow `RecordBatch`, then unflattens back to
  `OtapArrowRecords`.
- The server_cost `convert-A` measurement takes IPC bytes through decode,
  flatten, and file encode. This is the cost the server pays in Option A. The
  flatten and IPC decode are shared across schemes, so differences between
  schemes isolate the file-encode cost.
- The server_cost `accept-B` measurement takes file bytes back to an Arrow
  `RecordBatch` without rebuilding OTAP. This is what the server pays in Option B
  when it must touch the data. If the server only persists the received bytes,
  its conversion CPU is roughly zero.

The flattened layouts keep the entire root `Logs` record batch intact, so decode
carries its scalar and struct columns straight back just as the IPC path does,
and the comparison is not penalized by re-walking the body. Only the attribute
tables are denormalized and rebuilt. Resource and scope attribute sets are
re-normalized on decode using the `resource.id` and `scope.id` join keys the
`Logs` batch still carries. `parquet-wide` is lossless for type-consistent keys.
Any attribute that does not fit its single typed column spills into a per-group
`List<Struct>` overflow column, so the round-trip stays exact.

## Illustrative results

These numbers come from one development machine running WSL with jemalloc.
Absolute values vary by host, but the relationships are stable. The shape is
10,000 log records under a single resource and scope, whose OTLP protobuf
encoding is 2,270,225 bytes.

Serialized size in bytes:

| contender      | zstd   | lz4    | snappy | none       |
|----------------|--------|--------|--------|------------|
| ipc            | 92,270 | 108,846| n/a    | 2,457,008  |
| parquet-nested | 57,023 | 75,336 | 90,529 | 432,935    |
| parquet-wide   | 62,088 | 78,840 | 78,851 | 82,219     |
| vortex         | n/a    | n/a    | n/a    | 199,088    |
| vortex-fast    | n/a    | n/a    | n/a    | 13,138,288 |

Server convert cost, which is IPC decode plus flatten plus file encode, in
indicative milliseconds. The flatten and decode are shared, so differences
isolate the encode. The 100k column is the same shape scaled to 100,000 records.

| contender / comp       | convert 10k | convert 100k |
|------------------------|-------------|--------------|
| parquet-nested / zstd  | 93 ms       | 889 ms       |
| parquet-wide / zstd    | 44 ms       | 577 ms       |
| vortex                 | 692 ms      | 1727 ms      |
| vortex-fast            | 935 ms      | 1858 ms      |

## Takeaways

- The conversion is the expensive part. Flatten plus file encode dominates, and
  IPC decode is a small fraction. Accepting client-precomputed Parquet and
  persisting it removes essentially all of that server CPU. Even a server that
  must reparse the Parquet into Arrow still avoids the flatten and encode.
- On zstd versus lz4, IPC with lz4 is about 18 percent larger than zstd, and
  Parquet-nested with lz4 is about 32 percent larger while snappy is about 59
  percent larger than zstd. lz4 stays competitive on the wire, so a client that
  cannot use zstd, such as some .NET stacks, can use lz4 or snappy.
- On layout, `parquet-nested` and `parquet-map` compress smallest with zstd,
  while `parquet-wide` is smaller uncompressed and cheaper to encode and reparse
  thanks to typed columns, at a small size premium under zstd.
- For the debate, if server CPU is the constraint, have clients send precomputed
  flattened Parquet and let the server persist it.

## Vortex write experiment

The goal here was to see whether Vortex, which is Arrow-native, could be written
faster than Parquet at realistic block sizes of 10k and 100k records, ideally by
skipping compression. The answer in this experiment is no.

Vortex 0.75 pins the same arrow-rs 58.3 as this workspace, so Arrow arrays flow
in and out with no version bridging and the whole round-trip runs in memory.
Vortex 0.75 has no `FixedSizeBinary` encoding, so `trace_id` and `span_id` are
cast to `Binary` and restored on read, and decode targets an explicit plain
Arrow schema that the OTAP schema check accepts. The `vortex-fast` contender
disables BtrBlocks by building the write strategy with an empty compressor.

Two results stand out. First, Vortex write is slower than Parquet write at both
sizes. At 10k the convert cost is 692 ms for `vortex` and 935 ms for
`vortex-fast`, versus 93 ms for `parquet-nested/zstd`. At 100k it is 1727 ms and
1858 ms versus 889 ms. Second, disabling compression made Vortex both larger and
slower, not faster. `vortex-fast` produces 13 MB at 10k and 186 MB at 100k,
because turning off BtrBlocks also turns off dictionary encoding, so the
denormalized resource and scope attributes fully expand. Serializing that volume
through Vortex's write pipeline outweighs any saving from skipping compression,
which is why `vortex-fast` is slower than the compressed `vortex`.

The write bottleneck is therefore the Vortex write pipeline itself, not the
compressor. Even with compression off, Vortex still canonicalizes arrays,
repartitions into row blocks, computes zone statistics, builds a layout, and
writes a flatbuffer footer through an async pipeline, and for this wide, deeply
nested schema with many small columns that fixed work is much heavier than
Parquet's encoder. Vortex's default compression also did not improve with scale
on this data, going from about 20 bytes per record at 10k to about 62 at 100k,
while Parquet-zstd improved from about 5.7 to about 3.8.

This is a fair result for using Vortex as a Parquet drop-in for this write path,
but it is not a verdict on Vortex overall. Vortex is designed for selective reads
with column and row-range projection, pushdown, and random access over large
files, none of which this full-batch write-and-read workload exercises. Reducing
the write pipeline overhead, for example a dictionary-only encoding that stays
small while skipping the compression search, and exercising selective reads are
the obvious next steps.

## Extending

Contenders are the `Scheme` enum and its `Codec` implementations in
`benchmarks/src/parquet_study`. Add a variant to include a contender everywhere.
Input shapes are defined in `input_shapes()` in `benches/otap_parquet/main.rs`.
The flatten and unflatten round-trips and the server-cost helpers have unit tests
runnable with `cargo test -p benchmarks --lib parquet_study`.
