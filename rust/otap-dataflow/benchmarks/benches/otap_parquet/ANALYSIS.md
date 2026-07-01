<!-- markdownlint-disable MD013 -->

# Analysis: why OTAP/IPC is cheaper than flattened Parquet

This explains the ratios the `otap_parquet` benchmark measures. Numbers are from
one development machine running WSL with jemalloc. Times are indicative medians
in milliseconds. Both `zstd` and `lz4` are shown, because `lz4` is how the
comparison was measured elsewhere.

A single OTAP logs batch holds at most 65,535 log records, because the log id
that links a log row to its attributes is a `u16`. The breakdown below therefore
uses a 50,000-record batch as its headline, which is a large but valid single
batch. Volumes larger than 65,535 records must be streamed as several batches,
which is analyzed at the end and turns out to change the size comparison.

## Headline ratios (50k records, one batch)

| contender      | comp | encode ms | decode ms | size bytes | size vs ipc |
|----------------|------|-----------|-----------|------------|-------------|
| ipc            | zstd | 18.9      | 4.2       | 401,966    | 1.00x       |
| ipc            | lz4  | 16.1      | 20.0      | 466,416    | 1.16x       |
| parquet-nested | zstd | 171       | 54.5      | 240,211    | 0.60x       |
| parquet-nested | lz4  | 160       | 51.4      | 329,298    | 0.71x       |
| parquet-wide   | zstd | 94.4      | 34.2      | 245,757    | 0.61x       |
| parquet-wide   | lz4  | 91.7      | 32.2      | 327,547    | 0.70x       |

Reading the ratios with `lz4`, which is the relevant compressor here, for a
single 50k batch:

- Parquet-nested is 0.71x the IPC size, so it is smaller at rest.
- Parquet-nested costs about 10x more to encode than IPC, 160 ms versus 16 ms.
- Parquet-nested costs about 2.6x more to decode than IPC, 51 ms versus 20 ms.

With `zstd` the encode ratio is about the same, 9x, but the decode ratio is 13x,
because IPC decode with `zstd` is only 4.2 ms. The reason is the `lz4` decode
penalty explained below.

## Where the time goes

The benchmark breaks each side into two steps.

OTAP/IPC, 50k, per step:

| comp | transport-encode | ipc-serialize | ipc-deserialize | transport-decode |
|------|------------------|---------------|-----------------|------------------|
| zstd | 13.0             | 5.9           | 3.0             | 1.2              |
| lz4  | 13.5             | 2.6           | 18.7            | 1.3              |

Parquet, 50k, per step:

| scheme / comp         | flatten | pq-write | pq-read | unflatten |
|-----------------------|---------|----------|---------|-----------|
| parquet-nested / zstd | 44      | 127      | 39      | 16        |
| parquet-nested / lz4  | 44      | 116      | 35      | 17        |
| parquet-wide / zstd   | 69      | 25       | 14      | 20        |
| parquet-wide / lz4    | 69      | 22       | 12      | 20        |

## Explaining each ratio

### IPC encode is dominated by transport-optimize, not compression

The transport-optimized encoding is about 13 ms of the 16 to 19 ms IPC encode,
regardless of compressor. It makes a full pass over the four record batches,
applying delta and dictionary encodings to the id and value columns and remapping
parent ids so the child batches still reference the right rows. That pass touches
all of the data once, which is why it is the largest single IPC cost and why it
does not change with the compressor. The Arrow IPC serialization that follows is
small, 3 to 6 ms, because it writes already-compact columns and the compressor
runs on far less data.

### IPC serialize is faster when compressed

Uncompressed IPC serialize is much slower than compressed, because writing the
record batch means copying bytes and there are far more of them without
compression. The compressor pays for itself on the write side by shrinking what
has to be moved.

### IPC decode is where lz4 and zstd diverge sharply

Deserialize dominates IPC decode, and the transport decode is cheap. The striking
result is that `lz4` deserialize is about six times slower than `zstd` for this
data. In this Arrow IPC implementation the LZ4 frame decompression path is much
slower than zstd. That single step is why IPC decode is 4.2 ms with `zstd` but
20 ms with `lz4`, and therefore why the IPC-over-Parquet decode advantage shrinks
from about 13x with `zstd` to under 3x with `lz4`. Anyone measuring with `lz4`
will see IPC decode look far worse than a `zstd` measurement would, even though
the data is identical.

### Parquet encode is dominated by the writer, with a large flatten tax

Flatten is compressor-independent, because it is a pure Arrow transformation that
joins the attribute batches onto the log rows and builds the nested columns. For
`parquet-nested` and `parquet-map` the Parquet writer then dominates, because
encoding `List<Struct>` and `Map` columns requires Parquet definition and
repetition levels over many leaf fields. `parquet-wide` writes much faster
because its attributes are flat typed scalar columns that Parquet encodes
cheaply, but it pays more in flatten to explode the keys into columns. The
compressor barely moves the writer time, which is why `pq-write` is nearly
constant across `zstd`, `lz4`, and `snappy`, while the output size is not.

### Parquet decode is dominated by the reader

Parquet read is larger than unflatten. `parquet-wide` reads fastest because flat
columns decode directly, though its unflatten is a little more expensive because
it reassembles attributes from many columns.

## Streaming changes the size story

The size numbers above encode each batch as a self-contained Arrow IPC stream,
which pays the full schema and dictionary cost every time. That is the cold, or
worst, case. In real OTAP streaming the `Producer` is long-lived: it writes the
Arrow schema into the stream once, and it delta-encodes dictionaries, so every
batch after the first omits the schema and re-sends only new dictionary entries.
Parquet has no equivalent per-batch amortization, because each Parquet file is
self-contained with its own schema, footer, and per-row-group dictionary pages.

Measured cold versus steady-state IPC size per batch, with the equivalent single
Parquet file for reference:

| logs   | comp | cold    | warm    | saved  | pq-nested | warm/pq |
|--------|------|---------|---------|--------|-----------|---------|
| 1,000  | zstd | 24,748  | 13,422  | 11,326 | 19,512    | 0.69x   |
| 1,000  | lz4  | 27,692  | 16,366  | 11,326 | 21,136    | 0.77x   |
| 10,000 | zstd | 92,270  | 80,944  | 11,326 | 57,023    | 1.42x   |
| 10,000 | lz4  | 108,846 | 97,520  | 11,326 | 75,336    | 1.29x   |
| 50,000 | zstd | 401,966 | 390,640 | 11,326 | 240,211   | 1.63x   |
| 50,000 | lz4  | 466,416 | 455,088 | 11,326 | 329,298   | 1.38x   |

The amortization is a fixed cost of about 11,326 bytes per batch, and it is the
same at every batch size. Splitting that fixed cost by Arrow IPC message type, at
1,000 records with `zstd`, shows it is mostly dictionaries rather than schema:

| payload       | schema | dictionaries | data (warm) |
|---------------|--------|--------------|-------------|
| Logs          | 1,856  | 4,032        | 3,968       |
| LogAttrs      | 832    | 2,048        | 7,744       |
| ResourceAttrs | 448    | 832          | 832         |
| ScopeAttrs    | 448    | 832          | 832         |
| total         | 3,584  | 7,744        | 13,376      |

So of the 11,328 bytes saved per steady-state batch, 7,744 are dictionary
messages and only 3,584 are schema, about 68 percent dictionaries and 32 percent
schema. The dictionary messages are large not because the values are large, since
this data has few distinct values, but because the four batches carry many
dictionary columns, and each carries per-message framing that the stream sends
once and then reuses.

The cost is fixed with batch size because both parts are per-stream, not
per-row. The schema describes columns, not rows. The dictionaries are the set of
distinct values, and in this synthetic data that set is the same whether the
batch has 1,000 or 50,000 rows, so the dictionary messages do not grow. What
changes is how large that fixed cost is relative to the batch:

- At 1,000 records per batch it is about 46 percent of the cold size, and it
  flips the verdict: the steady-state IPC batch is smaller than the Parquet file,
  0.69x with `zstd` and 0.77x with `lz4`.
- At 10,000 records it is about 12 percent, and Parquet is still smaller on the
  wire, though the gap narrows.
- At 50,000 records it is about 3 percent, and Parquet keeps its size advantage.

This synthetic data is a best case for dictionary amortization, because every
batch carries the identical low-cardinality values, so batches after the first
send no new dictionary entries at all. Real telemetry amortizes only the stable
low-cardinality columns, such as attribute keys, severity, and scope names. A
high-cardinality column such as a trace id or a log body carries new values in
every batch, so its delta dictionary keeps sending new entries and does not
amortize, and such a column is often better left non-dictionary. The measured
amortization here should be read as the ceiling for dictionaries plus the schema,
which is always recovered.

This also explains why the steady-state batches do not keep shrinking. The drop
is one-time, at the first batch, and every batch after that is the same size. The
reason is that Arrow IPC amortizes the schema and the dictionary value tables
once, but it does not compress one batch against another: each batch's column
buffers are compressed independently so a reader can decode any batch on its own.
So even though the batches here are identical, every steady-state batch re-sends
and re-compresses the full per-row payload, which is the dictionary indices plus
the non-dictionary columns. Dictionary reuse saves the value tables, not the
per-row references, and the per-row payload is the actual information in the
batch. How much redundancy this leaves unexploited, and why a naive whole-stream
compressor does not recover it, is measured in the next section.

### The redundancy Arrow IPC leaves unexploited

Because each batch is compressed on its own, the steady-state batches are stored
at full size even when they are nearly identical. To measure how much that leaves
unexploited, the study also compresses a whole stream of eight batches as a single
unit and compares the per-batch cost. At 10,000 logs per batch with `zstd`:

| approach               | 8 batches | per extra batch |
|------------------------|-----------|-----------------|
| ipc, per-batch zstd    | 659 KB    | 81 KB           |
| whole stream, zstd L3  | 595 KB    | 74 KB           |
| whole stream, zstd L19 | 69 KB     | 269 B           |

The uncompressed batches here differ by a single byte, a batch counter, so they
are almost pure duplicates, yet Arrow IPC still spends about 81 KB on each one. A
default-effort whole-stream `zstd` barely does better, about 74 KB per extra
batch, because each uncompressed batch is roughly 2.4 MB, larger than the match
window at that level, so the compressor cannot see that the previous batch was a
duplicate. Only a large-window, long-distance configuration at level 19 finds the
match and collapses each extra batch to about 269 bytes, storing all eight in
69 KB, close to the size of one. That factor of roughly nine is the cross-batch
redundancy Arrow IPC leaves on the table in this best case.

Three caveats keep this from being free savings. First, it is a best case,
because these batches are byte-for-byte duplicates, whereas real telemetry batches
carry different records and share far less. Second, the large-window configuration
costs much more CPU than the light per-buffer codec Arrow IPC uses, so this is a
size ceiling, not a drop-in win. Third, whole-stream compression gives up the
per-batch independent decode that Arrow IPC provides, where any batch can be read
without the others. Arrow IPC trades cross-batch compression for low CPU and
independently decodable batches, which is the right trade for streaming transport,
so capturing the remaining redundancy is a job for a storage-side recompression
pass rather than the wire format.

So the single-batch size comparison understates OTAP/IPC, and it understates it
most for the small, frequent batches that low-latency telemetry actually sends.
The `u16` log-id limit reinforces this: because one batch cannot exceed 65,535
records, high volume is delivered as a stream of batches, which is exactly where
the schema and dictionary amortization applies.

## Applying the model: precompute Parquet at the gateway

The read and write costs above assume the converter runs on the server. The
deployment that motivates this study is different. A large sender, a customer
collector gateway, ships to a .NET SaaS ingestion service whose CPU is the
resource being optimized, and the same organization owns both the client exporter
and the storage schema. That ownership changes the trade in several ways.

First, the coupling objection to client-side Parquet mostly goes away. When a
third party owns the storage format, making it the wire contract is brittle. When
the ingestion service owns the exporter, it can emit exactly the flattened layout
its store wants, the columns, partitioning, sort order, row-group sizing, and
compression, and it can version the exporter and the store together. The gateway
also aggregates many hosts, so it forms the large batches where flattened Parquet
is 0.60 to 0.71x the IPC size, which cuts ingress bandwidth into the service.

Second, the decisive question becomes how much the ingestion service must read.
Precomputing Parquet removes server CPU only if ingestion is close to a validated
append. Two facts from the cost model bound this. If the service fully decodes,
Parquet is the most expensive input, about 13x the IPC decode with zstd, and the
service would re-encode anyway, so precomputing helps only when it avoids that
decode. But Parquet also carries a footer and per-row-group statistics, so tenant
routing, quota and cardinality checks, and min or max pruning can run on that
metadata without decoding column data. A service that inspects metadata and
appends pays far less than the 13x figure, and that is the regime where accepting
client Parquet wins.

Third, any work that needs the row values still forces a full decode, so the goal
is to move that work into the exporter. Per-record transforms and redaction move
cleanly to the gateway, since the gateway is the last writer. Cross-source metric
re-aggregation does not, because one gateway sees only its own slice, so temporal
rollups across gateways remain a server-side read. Logs and traces are therefore
the strong case for precomputed Parquet, while aggregated metrics are the residual
case that still wants OTAP/IPC and a server-side convert.

Two operational costs remain. Because exporters run on customer-managed
collectors, a storage-layout change cannot deploy atomically, so the ingestion
service must accept several layout versions at once and conform older ones itself,
which returns some CPU to the server. And not every sender is a large gateway, so
small and legacy senders still emit small batches where IPC is smaller and cheaper
than Parquet. The robust intake accepts both, OTAP/IPC for small senders and for
traffic that needs a server-side transform, and precomputed Parquet for large
gateways running the custom exporter.

Finally, the compressor interacts with the .NET stack. The measured IPC decode
advantage depends on zstd. With lz4 the IPC decode is about 20 ms at 50k against
4 ms for zstd, so if the service does decode, the Parquet decode penalty over IPC
falls from about 13x to about 2.6x. And if the .NET Parquet writer can emit zstd
even where the .NET Arrow IPC writer cannot, the exporter can ship zstd Parquet,
which is smaller than lz4 and cheaper for any server read than lz4 IPC. That is
worth confirming on the .NET stack, because it removes the main compressor
argument against the Parquet path for that sender.

## Bottom line

For a single large batch, flattened Parquet is smaller on the wire, about 0.60 to
0.71x the IPC size, but it costs roughly an order of magnitude more CPU to
produce and, with `zstd`, to consume. When the traffic is streamed, which is the
normal case and is required above 65,535 records per batch, OTAP/IPC amortizes a
fixed cost of about 11 KB per batch that is roughly two thirds dictionaries and
one third schema, and for small frequent batches that makes IPC smaller on the
wire than Parquet as well as far cheaper to produce and consume. Produce Parquet
where the columnar file and its smaller size for large data at rest are needed,
and keep the streaming client on OTAP/IPC. When comparing measurements, hold the
compressor fixed and state whether the size is cold or steady-state, because both
choices move the ratio by a large factor. And when one organization owns both the
exporter and the store, the applied section above shows the convert step can move
to the sending gateway, as long as ingestion stays close to a metadata-validated
append rather than a full decode.
