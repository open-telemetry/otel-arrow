<!-- markdownlint-disable MD013 -->

# Analysis: why OTAP/IPC is cheaper than flattened Parquet

This explains the ratios the `otap_parquet` benchmark measures. Numbers are from
one development machine running WSL with jemalloc, at the 100,000 log-record
shape under a single resource and scope. The OTLP protobuf encoding of that batch
is 22,700,225 bytes. Times are indicative medians in milliseconds. Both `zstd`
and `lz4` are shown, because `lz4` is how the comparison was measured elsewhere.

## Headline ratios (100k records)

| contender      | comp | encode ms | decode ms | size bytes | size vs ipc |
|----------------|------|-----------|-----------|------------|-------------|
| ipc            | zstd | 44.4      | 9.8       | 702,702    | 1.00x       |
| ipc            | lz4  | 41.6      | 44.7      | 915,632    | 1.30x       |
| parquet-nested | zstd | 552       | 217       | 384,740    | 0.55x       |
| parquet-nested | lz4  | 477       | 213       | 518,503    | 0.57x       |
| parquet-wide   | zstd | 330       | 147       | 392,782    | 0.56x       |
| parquet-wide   | lz4  | 330       | 142       | 509,659    | 0.56x       |

Reading the ratios with `lz4`, which is the relevant compressor here:

- Parquet-nested is 0.57x the IPC size, so it is smaller on the wire.
- Parquet-nested costs 11.5x more to encode than IPC, 477 ms versus 41.6 ms.
- Parquet-nested costs 4.8x more to decode than IPC, 213 ms versus 44.7 ms.

With `zstd` the encode ratio is about the same, 12.4x, but the decode ratio jumps
to 22.2x, because IPC decode with `zstd` is only 9.8 ms. The reason is explained
below and it is the single most important thing to know when comparing `lz4`
measurements to `zstd` measurements.

## Where the time goes

The benchmark breaks each side into two steps.

OTAP/IPC, 100k, per step:

| comp | transport-encode | ipc-serialize | ipc-deserialize | transport-decode |
|------|------------------|---------------|-----------------|------------------|
| zstd | 35.2             | 9.2           | 6.6             | 3.2              |
| lz4  | 34.5             | 7.1           | 42.2            | 2.4              |

Parquet, 100k, per step:

| scheme / comp         | flatten | pq-write | pq-read | unflatten |
|-----------------------|---------|----------|---------|-----------|
| parquet-nested / zstd | 148     | 404      | 152     | 65        |
| parquet-nested / lz4  | 148     | 329      | 143     | 70        |
| parquet-wide / zstd   | 183     | 147      | 56      | 91        |
| parquet-wide / lz4    | 183     | 147      | 55      | 87        |

## Explaining each ratio

### IPC encode is dominated by transport-optimize, not compression

The transport-optimized encoding is about 35 ms of the 42 to 44 ms IPC encode,
regardless of compressor. It makes a full pass over the four record batches,
applying delta and dictionary encodings to the id and value columns and remapping
parent ids so the child batches still reference the right rows. That pass touches
all of the data once, which is why it is the largest single IPC cost and why it
does not change with the compressor. The Arrow IPC serialization that follows is
small, 7 to 9 ms, because it writes already-compact columns and the compressor
runs on far less data.

### IPC serialize is faster when compressed

Uncompressed IPC serialize is 65 ms, far more than the 7 to 9 ms with `zstd` or
`lz4`. Writing the record batch means copying bytes, and without compression that
is 24 MB instead of about 700 KB. The compressor pays for itself on the write
side by shrinking what has to be moved.

### IPC decode is where lz4 and zstd diverge sharply

Deserialize dominates IPC decode, and the transport decode is cheap, 2 to 3 ms.
The striking result is that `lz4` deserialize is 42 ms while `zstd` deserialize
is 6.6 ms, a 6x difference. In this Arrow IPC implementation the LZ4 frame
decompression path is much slower than zstd for this data. That single step is
why IPC decode is 9.8 ms with `zstd` but 44.7 ms with `lz4`, and therefore why
the IPC-over-Parquet decode advantage shrinks from 22x with `zstd` to under 5x
with `lz4`. Anyone measuring with `lz4` will see IPC decode look far worse than a
`zstd` measurement would suggest, even though the data is identical.

### Parquet encode is dominated by the writer, with a large flatten tax

Flatten is 140 to 185 ms and is compressor-independent, because it is a pure
Arrow transformation that joins the attribute batches onto the log rows and
builds the nested columns. For `parquet-nested` and `parquet-map` the Parquet
writer then costs 330 to 400 ms, because encoding `List<Struct>` and `Map`
columns requires Parquet definition and repetition levels over many leaf fields.
`parquet-wide` writes in about 147 ms instead, because its attributes are flat
typed scalar columns that Parquet encodes cheaply, but it pays more in flatten,
183 ms, to explode the keys into columns. The compressor barely moves the writer
time, which is why `pq-write` is nearly constant across `zstd`, `lz4`, and
`snappy`, while the output size is not.

### Parquet decode is dominated by the reader

Parquet read is 55 to 152 ms and unflatten is 65 to 91 ms. `parquet-wide` reads
much faster, 55 ms, because flat columns decode directly, but its unflatten is a
little more expensive because it must reassemble attributes from many columns.

## Bottom line

Flattened Parquet is smaller on the wire, about 0.55 to 0.57x the IPC size, but
it costs roughly an order of magnitude more CPU to produce and, with `zstd`, to
consume. If the constraint is server CPU, keep the client on OTAP/IPC. Produce
Parquet only where the columnar file and its smaller size at rest are needed, and
expect that cost to land wherever the flatten and Parquet write run. When
comparing measurements, hold the compressor fixed, because the `lz4` IPC decode
penalty alone changes the decode ratio by more than 4x.
