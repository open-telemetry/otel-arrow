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

Three tables are printed to stdout before the timed round-trip benchmarks run:
serialized size, the OTAP/IPC pipeline breakdown, and the Parquet pipeline
breakdown. The default shapes are 10k, 50k, and 100k log records, so the full
Criterion sweep is slow. The printed tables are the main output; for a quick pass
add `-- --measurement-time 0.5 --sample-size 10`, or read the tables and stop the
run.

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
100,000 log-record shape whose OTLP protobuf encoding is 22,700,225 bytes.
Absolute values vary by host, but the relationships are stable.

Serialized size in bytes:

| contender      | zstd    | lz4     | snappy  | none       |
|----------------|---------|---------|---------|------------|
| ipc            | 702,702 | 915,632 | n/a     | 24,428,084 |
| parquet-nested | 384,740 | 518,503 | 800,452 | 7,079,037  |
| parquet-wide   | 392,782 | 509,659 | 608,717 | 2,838,767  |

OTAP/IPC pipeline breakdown, milliseconds:

| comp | t-enc | ipc-ser | enc-tot | ipc-des | t-dec | dec-tot |
|------|-------|---------|---------|---------|-------|---------|
| zstd | 35.2  | 9.2     | 44.4    | 6.6     | 3.2   | 9.8     |
| lz4  | 34.5  | 7.1     | 41.6    | 42.2    | 2.4   | 44.7    |
| none | 39.5  | 64.9    | 104.4   | 54.6    | 2.5   | 57.1    |

Parquet pipeline breakdown, milliseconds:

| scheme / comp         | flatten | pq-write | enc-tot | pq-read | unflat | dec-tot |
|-----------------------|---------|----------|---------|---------|--------|---------|
| parquet-nested / zstd | 148     | 404      | 552     | 152     | 65     | 217     |
| parquet-nested / lz4  | 148     | 329      | 477     | 143     | 70     | 213     |
| parquet-map / zstd    | 139     | 363      | 503     | 160     | 63     | 223     |
| parquet-map / lz4     | 139     | 334      | 474     | 167     | 67     | 234     |
| parquet-wide / zstd   | 183     | 147      | 330     | 56      | 91     | 147     |
| parquet-wide / lz4    | 183     | 147      | 330     | 55      | 87     | 142     |

A companion write-up of what these ratios mean is in
[`ANALYSIS.md`](./ANALYSIS.md).

## Takeaways

- IPC is far cheaper than Parquet on both sides. At 100k with zstd, IPC encodes
  in about 44 ms and decodes in about 10 ms, while `parquet-nested` encodes in
  about 552 ms and decodes in about 217 ms. That is roughly 12 times cheaper to
  encode and 22 times cheaper to decode.
- Inside IPC encode, the transport-optimized encoding dominates, about 35 ms of
  the 44 ms, and it is essentially compressor-independent because it runs before
  compression. Inside IPC decode, the deserialization dominates and the transport
  decode is small.
- Inside Parquet encode, the Parquet writer dominates and the flatten is roughly
  a third to a half of the total. `parquet-wide` writes fastest because it has
  typed scalar columns rather than nested `List<Struct>` or `Map`, but it pays
  more in flatten, and it ends up the cheapest Parquet encoder overall at 330 ms.
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
`ipc::deserialize`, and `ipc::transport_decode`; the Parquet steps are
`Scheme::flatten`, `parquet_io::write_parquet`, `parquet_io::read_parquet`, and
`Scheme::unflatten`. Input shapes are defined in `input_shapes()` in
`benches/otap_parquet/main.rs`. The round-trips have unit tests runnable with
`cargo test -p benchmarks --lib parquet_study`.
