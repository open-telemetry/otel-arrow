<!-- markdownlint-disable MD013 -->

# `otap_parquet` benchmark: OTAP/IPC vs flattened Parquet

**Status:** experiment on branch `jmacd/parquet_study`.

This benchmark studies the cost of moving OTAP logs between a client and a server
two ways: as compressed Arrow IPC, which is the representation we have today, and
as a flattened single-file Parquet, which a server would store. It starts from an
OTAP logs batch, which is four Arrow record batches (Logs, ResourceAttrs,
ScopeAttrs, LogAttrs), and breaks each pipeline into its sub-steps so the cost of
every stage is visible on both the encode and decode side.

- OTAP/IPC encode is transport-optimize, then Arrow IPC serialize with
  compression. Decode is IPC deserialize, then transport-decode.
- Parquet encode is flatten to one Arrow record batch, then write Parquet. Decode
  is read Parquet, then unflatten.

## Contenders

- `ipc` is the OTAP representation we have today: interleaved Arrow IPC streams
  produced by `Producer` and consumed by `Consumer`, with each per-payload stream
  compressed.
- `parquet-nested` is a single flattened Parquet file where each log row carries
  its denormalized resource, scope, and log attributes as
  `List<Struct{key,type,str,int,double,bool,bytes,ser}>` columns.
- `parquet-map` is the same, with attributes stored as
  `Map<Utf8, Struct{type,...}>`.
- `parquet-wide` is the analytics-flat extreme, where every distinct attribute
  key becomes its own typed top-level column named `resource.<key>`,
  `scope.<key>`, or `log.<key>`.

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

Four tables are printed to stdout before the timed round-trip benchmarks run:
serialized size, the OTAP/IPC pipeline breakdown, the Parquet pipeline breakdown,
and the OTAP/IPC streaming amortization. The breakdown shapes are 10k, 30k, and
60k log records. A single OTAP logs batch holds at most 65,535 records because
the log id is a `u16`, so the shapes stay below that and larger volumes are
streamed. The full Criterion sweep is slow; the printed tables are the main
output. For a quick pass add `-- --measurement-time 0.5 --sample-size 10`, or
read the tables and stop the run.

## Pipeline steps

- IPC `t-enc` is the OTAP transport-optimized encoding, which applies
  delta and dictionary encodings to id and value columns and remaps parent ids.
  `ipc-ser` is the Arrow IPC serialization with compression plus the prost
  encoding of the `BatchArrowRecords`. Because `Producer::produce_bar` bundles
  the two, `ipc-ser` is reported as the encode total minus `t-enc`.
- IPC `ipc-des` is the prost decode plus `Consumer::consume_bar` plus
  `from_record_messages`, which yields a batch still in the transport-optimized
  encoding. `t-dec` is `decode_transport_optimized_ids`, which restores the
  logical batch.
- Parquet `flatten` builds the single flat Arrow record batch. `pq-write` is the
  Arrow Parquet writer. `pq-read` is the Arrow Parquet reader, and `unflat`
  reconstructs the four OTAP record batches.

The flattened layouts keep the entire root `Logs` record batch intact, so decode
carries its scalar and struct columns straight back just as the IPC path does.
Only the attribute tables are denormalized and rebuilt, re-normalizing resource
and scope sets with the `resource.id` and `scope.id` join keys the `Logs` batch
carries.

## Illustrative results

These numbers come from one development machine running WSL with jemalloc, at the
50,000 log-record shape, which is a large but valid single OTAP batch. Absolute
values vary by host, but the relationships are stable.

Serialized size in bytes:

| contender      | zstd    | lz4     | snappy  | none       |
|----------------|---------|---------|---------|------------|
| ipc            | 401,966 | 466,416 | n/a     | 12,221,748 |
| parquet-nested | 240,211 | 329,298 | 405,397 | 2,118,185  |
| parquet-wide   | 245,757 | 327,547 | 327,294 | 344,708    |

OTAP/IPC pipeline breakdown, milliseconds:

| comp | t-enc | ipc-ser | enc-tot | ipc-des | t-dec | dec-tot |
|------|-------|---------|---------|---------|-------|---------|
| zstd | 13.0  | 5.9     | 18.9    | 3.0     | 1.2   | 4.2     |
| lz4  | 13.5  | 2.6     | 16.1    | 18.7    | 1.3   | 20.0    |
| none | 12.6  | 29.6    | 42.2    | 27.1    | 1.2   | 28.3    |

Parquet pipeline breakdown, milliseconds:

| scheme / comp         | flatten | pq-write | enc-tot | pq-read | unflat | dec-tot |
|-----------------------|---------|----------|---------|---------|--------|---------|
| parquet-nested / zstd | 44      | 127      | 171     | 39      | 16     | 55      |
| parquet-nested / lz4  | 44      | 116      | 160     | 35      | 17     | 51      |
| parquet-map / zstd    | 52      | 133      | 185     | 53      | 14     | 67      |
| parquet-map / lz4     | 52      | 110      | 162     | 53      | 14     | 66      |
| parquet-wide / zstd   | 69      | 25       | 94      | 14      | 20     | 34      |
| parquet-wide / lz4    | 69      | 22       | 92      | 12      | 20     | 32      |

Streaming amortization, IPC bytes per batch when a long-lived Producer streams
many batches, with the equivalent single Parquet file for reference. `cold` is
the first batch, `warm` is steady-state, and `saved` is the fixed schema and
dictionary cost that streaming amortizes:

| logs   | comp | cold    | warm    | saved  | pq-nested | warm/pq |
|--------|------|---------|---------|--------|-----------|---------|
| 1,000  | zstd | 24,748  | 13,422  | 11,326 | 19,512    | 0.69x   |
| 10,000 | zstd | 92,270  | 80,944  | 11,326 | 57,023    | 1.42x   |
| 50,000 | zstd | 401,966 | 390,640 | 11,326 | 240,211   | 1.63x   |

A companion write-up of what these ratios mean, including the streaming effect,
is in [`ANALYSIS.md`](./ANALYSIS.md).

## Takeaways

- IPC is far cheaper than Parquet on both sides. At 50k with zstd, IPC encodes in
  about 19 ms and decodes in about 4 ms, while `parquet-nested` encodes in about
  171 ms and decodes in about 55 ms. That is roughly 9 times cheaper to encode and
  13 times cheaper to decode.
- Inside IPC encode, the transport-optimized encoding dominates, about 13 ms of
  the 19 ms, and it is essentially compressor-independent because it runs before
  compression. Inside IPC decode, the deserialization dominates and the transport
  decode is small.
- Streaming amortizes a fixed cost of about 11 KB per batch, which is roughly two
  thirds dictionary messages and one third schema, and is independent of the row
  count. For small frequent batches this flips the size verdict: at 1,000 records
  the steady-state IPC batch is 0.69x the Parquet file. Parquet has no equivalent
  per-batch amortization. Because a batch cannot exceed 65,535 records, high
  volume is streamed, which is where this applies.
- Inside Parquet encode, the Parquet writer dominates and the flatten is roughly
  a third to a half of the total. `parquet-wide` writes fastest because it has
  typed scalar columns rather than nested `List<Struct>` or `Map`, but it pays
  more in flatten, and it ends up the cheapest Parquet encoder overall at 94 ms.
- Compression choice matters in surprising ways. For IPC, compression makes the
  serialize step faster because there is far less data to move, so `none` is the
  slowest to serialize, and `lz4` is much slower to deserialize than `zstd` in
  this Arrow IPC implementation. For Parquet, the writer time is largely
  insensitive to the compressor, while the size is not.
- For the debate, if server CPU is the constraint, the client should keep sending
  OTAP/IPC, which is an order of magnitude cheaper to produce and consume than
  Parquet. Producing Parquet is worth it only where the columnar file and its
  smaller `zstd` size are needed at rest, and that cost lands wherever the flatten
  and Parquet write run.

## Extending

Contenders are the `Scheme` enum and its `Codec` implementations in
`benchmarks/src/parquet_study`. Add a variant to include a contender everywhere.
The IPC sub-steps are `ipc::transport_encode`, `ipc::encode_to_bytes`,
`ipc::deserialize`, and `ipc::transport_decode`, and `ipc::stream_batch_sizes`
measures streaming amortization; the Parquet steps are `Scheme::flatten`,
`parquet_io::write_parquet`, `parquet_io::read_parquet`, and `Scheme::unflatten`.
Input shapes are defined in `input_shapes()` and `streaming_shapes()` in
`benches/otap_parquet/main.rs`. The round-trips have unit tests runnable with
`cargo test -p benchmarks --lib parquet_study`.
