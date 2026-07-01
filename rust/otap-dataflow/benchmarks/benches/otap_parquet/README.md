<!-- markdownlint-disable MD013 -->

# `otap_parquet` benchmark: OTAP/IPC vs flattened Parquet

**Status:** *experiment* (branch `jmacd/parquet_study`)

This benchmark quantifies moving OTAP **logs** from a client to a server that
ultimately stores **flattened, single-file Parquet**. Because the data ends up
as Parquet on the server either way, the central question is where the
`OTAP -> Parquet` conversion CPU is spent:

- **Option A** — the client sends OTAP as Arrow IPC and the **server** converts
  it to Parquet (IPC-decode + flatten + Parquet-encode).
- **Option B** — the **client** precomputes the flattened Parquet and sends
  that; the server just persists the bytes (or reparses them into Arrow if it
  needs to partition/index/validate).

It measures **write time**, **read time**, **serialized size**, and the
**server-side conversion cost** for the same logs batch across four
representations and four compressors.

## Contenders

- `ipc` — the OTAP representation we have today: interleaved Arrow IPC streams
  (`Producer` / `Consumer`), each per-payload stream compressed.
- `parquet-nested` — a single flattened Parquet file where each log row carries
  its denormalized resource / scope / log attributes as
  `List<Struct{key,type,str,int,double,bool,bytes,ser}>` columns.
- `parquet-map` — the same, but attributes are `Map<Utf8, Struct{type,...}>`.
- `parquet-wide` — the "analytics-flat" extreme: every distinct attribute key
  becomes its own typed top-level column (`resource.<key>`, `scope.<key>`,
  `log.<key>`).

## Compressors

Compressors are explicit codecs so `zstd` can be compared head-to-head with
`lz4`. This matters for cross-language consumers: some Arrow/Parquet stacks (for
example certain .NET builds) may not support `zstd`, so `lz4` (and `snappy` for
Parquet) need first-class numbers.

| compressor | Arrow IPC     | Parquet     |
|------------|---------------|-------------|
| `zstd`     | `ZSTD`        | `ZSTD`      |
| `lz4`      | `LZ4_FRAME`   | `LZ4_RAW`   |
| `snappy`   | *unsupported* | `SNAPPY`    |
| `none`     | uncompressed  | uncompressed|

Arrow IPC only supports `zstd` and `lz4`, so `snappy` is offered for the Parquet
schemes only. Parquet uses `LZ4_RAW` (the cross-language interoperable variant,
not the deprecated Hadoop-framed `LZ4`).

## Running

```bash
cargo bench -p benchmarks --bench otap_parquet
```

Two tables (serialized size, and the server-side CPU model) are printed to
stdout before the timed benchmarks run. For a quick pass:

```bash
cargo bench -p benchmarks --bench otap_parquet -- \
  --warm-up-time 0.3 --measurement-time 0.6 --sample-size 10
```

## What each measurement covers

- **IPC write:** `Producer::produce_bar` + prost-encode `BatchArrowRecords`.
- **IPC read:** prost-decode, `Consumer::consume_bar`, `from_record_messages`,
  and `decode_transport_optimized_ids` back to `OtapArrowRecords`.
- **Parquet write:** flatten `OtapArrowRecords` → one Arrow `RecordBatch` →
  `ArrowWriter` → `Vec<u8>`.
- **Parquet read:** `ParquetRecordBatchReader` → Arrow `RecordBatch` → unflatten
  back to `OtapArrowRecords`.
- **server_cost `convert-A`:** IPC bytes → decode → flatten → Parquet bytes
  (Option A: the server converts).
- **server_cost `accept-B`:** Parquet bytes → Arrow `RecordBatch`, i.e. reparse
  without rebuilding OTAP (Option B when the server must touch the data). If the
  server only persists the received bytes, its conversion CPU is ~0.

The flattened Parquet layouts keep the entire root `Logs` record batch intact
(so decode carries its scalar/struct columns straight back, just like the IPC
path does — not penalized by re-walking the body) and only the attribute tables
are denormalized and rebuilt. Resource/scope attribute sets are re-normalized on
decode using the `resource.id` / `scope.id` join keys the `Logs` batch still
carries. `parquet-wide` is lossless for type-consistent keys; any attribute that
does not fit its single typed column spills into a per-group `List<Struct>`
overflow column, so the round-trip stays exact.

## Illustrative results

From one development machine (WSL, jemalloc); absolute values vary by host, but
the relationships are stable. Shape `r1_s1_l5000` = 5000 log records under a
single resource/scope (OTLP proto = 1,135,223 bytes).

Serialized size (bytes):

| contender      | zstd   | lz4    | snappy | none      |
|----------------|--------|--------|--------|-----------|
| ipc            | 53,614 | 63,598 | —      | 1,236,336 |
| parquet-nested | 35,634 | 44,904 | 52,471 | 223,441   |
| parquet-map    | 35,946 | 45,216 | 52,783 | 223,753   |
| parquet-wide   | 40,524 | 48,907 | 48,939 | 50,532    |

Server-side conversion cost (indicative ms), shape `r1_s1_l5000`:

| flatten / comp        | A: server converts | B: reparse | saved if server just stores |
|-----------------------|--------------------|------------|-----------------------------|
| parquet-nested / zstd | 14.46 ms           | 2.88 ms    | 14.46 ms                    |
| parquet-nested / lz4  | 13.49 ms           | 2.80 ms    | 13.49 ms                    |
| parquet-wide / lz4    | 9.95 ms            | 1.19 ms    | 9.95 ms                     |

## Takeaways

- **The conversion is the expensive part.** Flatten + Parquet-encode dominates;
  IPC decode is a small fraction. Accepting client-precomputed Parquet (Option
  B, persist) removes essentially all of that server CPU — here ~10–14 ms per
  5000-log batch, an ~80–100% reduction versus converting server-side. Even if
  the server must reparse the Parquet into Arrow, it still avoids the
  flatten+encode and saves ~80%.
- **zstd vs lz4.** For IPC, lz4 is ~19% larger than zstd; for Parquet-nested,
  lz4 is ~26% larger and snappy ~47% larger than zstd. But lz4 encode/decode is
  a touch *cheaper* on CPU, and `parquet-nested/lz4` (44.9 KB) is still smaller
  than `ipc/zstd` (53.6 KB) — so a zstd-less client (e.g. some .NET stacks) can
  use lz4 (or snappy) and remain competitive on the wire.
- **Layout.** `parquet-nested`/`parquet-map` compress smallest with zstd;
  `parquet-wide` is far smaller uncompressed and is the cheapest to write and to
  reparse (typed columns), at a modest size premium under zstd.
- **For the debate:** if server CPU is the constraint (it is here), have clients
  send precomputed flattened Parquet and let the server persist it. lz4 is a safe
  cross-language codec choice with a small size cost relative to zstd.

## Extending

Contenders are the `Scheme` enum and its `Codec` impls in
`benchmarks/src/parquet_study`; add a variant to include it everywhere. Input
shapes are in `input_shapes()` in `benches/otap_parquet/main.rs`. The
flatten/unflatten round-trips and the server-cost helpers have unit tests
(`cargo test -p benchmarks --lib parquet_study`).
