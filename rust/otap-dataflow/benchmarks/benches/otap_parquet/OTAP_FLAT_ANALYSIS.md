<!-- markdownlint-disable MD013 -->

# Analysis: an efficient OTAP-flat single columnar view

This is a companion to [`ANALYSIS.md`](./ANALYSIS.md). That study measured the
cost of moving OTAP logs as compressed Arrow IPC versus flattened Parquet, and
found that producing Parquet costs roughly an order of magnitude more CPU than
IPC. A large and compressor-independent part of that Parquet cost is the
`flatten` step, which turns OTAP's four normalized record batches into one
denormalized table before the Parquet writer ever runs. This document focuses on
that intermediate. It asks how cheaply OTAP can be presented as a single
columnar view, and it measures three ways to do it. It then asks whether that
flat view is also a good format to move between two services, and measures it
against OTAP-standard and Parquet on the wire. Finally it measures every
conversion in the pipeline edge by edge, to show that the flat view sits at a
natural center where each move to a neighbor rewrites only a handful of columns
and copies the rest.

Numbers below are indicative medians in milliseconds and bytes from one
development machine running WSL with jemalloc. Both scenarios hold the record
count fixed at 60,000 logs and vary only the attribute mix, because the layouts
studied here differ only in how they treat attributes.

## The flatten tax and where it comes from

An OTAP logs batch is four Arrow record batches. The root `Logs` batch holds one
row per record with the scalar and struct columns. Three attribute batches,
`ResourceAttrs`, `ScopeAttrs`, and `LogAttrs`, hold the attributes, each linked
to its parent by a `parent_id` column. This is a normalized, relational shape
that is compact on the wire because a resource's attributes are stored once and
referenced by every log record under that resource.

A single columnar view is the opposite shape. It is one table with one row per
log record, where each row can reach its resource, scope, and log attributes
without a join. Parquet needs this shape, and so does a query engine that wants
to scan the data as columns. The `flatten` step performs the join. The existing
flattened contenders build it with `gather_by_parent`, which constructs a
`HashMap<u16, Vec<u32>>` for each attribute batch and then materializes every
value column with a random-access `take`. That hash join and full materialization
is the flatten tax.

## The structural opportunity

The OTAP encoder does not emit attributes in arbitrary order. It walks resources,
scopes, and records in order, and for each parent it appends that parent's
attributes contiguously before moving on. The result is that every attribute
batch arrives already grouped by `parent_id` in ascending, contiguous runs, and
the `Logs.id` column that keys the log attributes is a sequential, nullable
`u16`. A probe over generated data confirms it directly. `ResourceAttrs.parent_id`
is `0, 1, 2`, `ScopeAttrs.parent_id` is `0, 0, 1, 1, 2, 2`, and
`LogAttrs.parent_id` is a run of zeros, then a run of ones, and so on.

This ordering makes the hash join unnecessary. Two facts follow from it.

First, the per-record log attributes can become a `List<Struct>` almost for free.
The struct children are the existing `LogAttrs` value columns used as they are,
with no `take`, and the list offsets come from a single linear scan that walks
the sorted `parent_id` runs alongside the sequential `Logs.id` column. Every log
attribute belongs to exactly one record, so no value is copied and no value is
shared.

Second, the shared resource and scope attributes line up with runs of log rows,
because the records are already grouped by resource and then by scope. A resource
therefore spans a contiguous block of rows, which is exactly the structure that
run-end and dictionary encodings capture without physically repeating anything.

## Three layouts of the shared attributes

All three layouts studied here build the log attributes the same zero-copy way.
They differ only in how they present the shared resource and scope sets in the
single view.

- Materialized repeats each set physically across its rows as a plain
  `List<Struct>`. This is the same shape the `parquet-nested` contender produces,
  and it is the only one arrow-rs can write to Parquet.
- Run-end encoded stores each set once as `RunEndEncoded<i32, List<Struct>>`. The
  view is logically per-row, and the physical storage holds one list per resource
  or scope plus the run boundaries.
- Dictionary stores each set once as `Dictionary<u16, List<Struct>>`. The view
  carries a per-row `u16` index into a small table that holds one list per
  resource or scope.

A spike settled how far each form travels. The arrow-rs Parquet writer at version
58.3 cannot serialize a run-end-encoded column, and it cannot serialize a
dictionary whose values are `List<Struct>`. It reports the run-end case as not
supported and the nested-dictionary case as not yet implemented. A dictionary of
a primitive such as `Utf8` does write and round-trips as a dictionary. The
conclusion is that run-end and nested-dictionary views are in-memory and query
representations rather than on-disk ones. That fits a design where the live store
is Arrow-native and Parquet is an export and interchange format rather than the
serving format.

## Measurements

The data models realistic telemetry rather than identical rows. Every record
carries a unique pseudo-random `trace_id` and `span_id`, a varying timestamp, a
templated body, and mixed-type log attributes that blend low-cardinality enums
with high-cardinality per-record values. Resource and scope attributes are
distinct per resource or scope but shared across that resource's records. This
matters because a degenerate feed of identical records collapses under
dictionary and delta encoding and would flatter whichever format encodes best,
which distorts the comparison.

The table reports the conversion cost from OTAP to a single record batch, the
in-memory footprint of that view, and the Parquet encode of the view where
arrow-rs can write it. The baseline is the existing nested flatten with its hash
join and full `take`.

Log-heavy, 60,000 logs under one resource with one resource attribute, two scope
attributes, and nine attributes per record:

| contender              | convert ms |  view mem | pq-write ms |  pq bytes | writable |
| ---------------------- | ---------: | --------: | ----------: | --------: | -------- |
| nested, baseline       |       74.1 |   41.8 MB |       236.0 | 3,863,351 | yes      |
| otap-flat-materialized |       18.0 |   46.7 MB |       277.4 | 3,863,351 | yes      |
| otap-flat-ree          |        8.4 |   36.0 MB |         n/a |       n/a | no       |
| otap-flat-dict         |        8.9 |   36.2 MB |         n/a |       n/a | no       |

Resource-heavy, 60,000 logs across 600 resources with twenty attributes each,
five scope attributes, and two attributes per record:

| contender              | convert ms |  view mem | pq-write ms |  pq bytes | writable |
| ---------------------- | ---------: | --------: | ----------: | --------: | -------- |
| nested, baseline       |       94.3 |  101.9 MB |       355.6 | 2,750,262 | yes      |
| otap-flat-materialized |       86.8 |  101.9 MB |       380.0 | 2,750,262 | yes      |
| otap-flat-ree          |        4.4 |   12.7 MB |         n/a |       n/a | no       |
| otap-flat-dict         |        4.3 |   12.9 MB |         n/a |       n/a | no       |

## Reading the results

There are two independent levers, and the two scenarios separate them.

The first lever is the zero-copy log attributes, and the materialized layout
already captures it because it shares the same log-attribute path. In the
log-heavy scenario the record attributes dominate, and building them without a
hash join and without a full `take` cuts the conversion from 74 milliseconds to
18, about four times faster, while producing a byte-identical Parquet file. The
run-end and dictionary layouts go a little further in this scenario, to about 8
and 9 milliseconds, because they also stop repeating the small resource and scope
sets, and that trims the in-memory view from 47 to 36 megabytes.

The second lever is the shared attributes, and only the run-end and dictionary
layouts capture it. The resource-heavy scenario is where this shows. The
materialized layout has to repeat twenty resource attributes across every one of
the 60,000 rows, which is 1.2 million struct rows of pure duplication, so it is
no cheaper than the baseline at about 87 milliseconds and it holds a 102 megabyte
view. The run-end layout stores each resource's attributes once, 600 lists in
total, so it builds the same logical view in 4.4 milliseconds and holds it in
12.7 megabytes. That is about twenty times less conversion work and eight times
less memory for a view that answers the same per-row questions. The dictionary
layout is close behind at 4.3 milliseconds and 12.9 megabytes.

The Parquet column is the counterpoint. The on-disk size is the same for the
materialized layout as for the baseline, about 3.9 and 2.8 megabytes, because the
Parquet writer applies its own run-length and dictionary encodings and recovers
the repetition that the materialized view spelled out in memory. The physical
duplication in the materialized layout is therefore a cost paid in build time and
peak memory, not in file size. Writing Parquet still requires that materialized
form, so the conversion cannot be avoided when Parquet is the target. It can only
be avoided when the consumer reads the columnar view directly.

## Transferring between two services

The measurements so far are about building the single view and holding it in
memory. A different question is whether the flat view is a good format to move
between two services, which is what OTAP-standard does today. To answer it the
study serializes each form the way it would travel. OTAP-standard is the
transport-optimized `Producer` over the four normalized batches. Each flat layout
is one record batch written as a plain Arrow IPC stream, which unlike Parquet can
carry run-end and dictionary columns. Parquet is the flat batch written as a
file. The table reports the compressed wire size under zstd and lz4, the encode
cost from an OTAP batch to wire bytes, and the decode cost from wire bytes to the
receiver's working form.

Log-heavy:

| contender             |  zstd wire |   lz4 wire | encode ms | decode ms | receiver form   |
| --------------------- | ---------: | ---------: | --------: | --------: | --------------- |
| ipc-standard          |  4,479,026 |  6,442,420 |      84.3 |      12.6 | normalized OTAP |
| ipc-flat-materialized |  7,013,768 | 11,015,624 |      88.6 |      27.8 | flat table      |
| ipc-flat-ree          |  5,663,048 |  9,061,640 |      63.7 |      23.0 | flat table      |
| ipc-flat-dict         |  5,663,240 |  9,062,728 |      71.8 |      20.4 | flat table      |
| parquet-flat          |  3,863,351 |  5,777,270 |     238.1 |      57.9 | flat table      |

Resource-heavy:

| contender             |  zstd wire |   lz4 wire | encode ms | decode ms | receiver form   |
| --------------------- | ---------: | ---------: | --------: | --------: | --------------- |
| ipc-standard          |  3,013,172 |  3,841,908 |      28.0 |       5.6 | normalized OTAP |
| ipc-flat-materialized | 12,137,288 | 17,397,896 |     153.3 |      65.4 | flat table      |
| ipc-flat-ree          |  3,345,992 |  4,736,200 |      15.1 |       6.5 | flat table      |
| ipc-flat-dict         |  3,345,544 |  4,737,672 |      15.5 |       6.3 | flat table      |
| parquet-flat          |  2,750,262 |  3,783,505 |     416.4 |     172.6 | flat table      |

On realistic data the wire sizes land close together, and the ordering is
Parquet first, then OTAP-standard, then the run-end flat form, with the
materialized flat form well behind. Parquet is the smallest on the wire, about
3.9 megabytes log-heavy and 2.8 resource-heavy, because it applies run-length and
dictionary encoding to every column. OTAP-standard is next and close, about 4.5
and 3.0 megabytes, because it never denormalizes the shared attributes and
transport-optimizes ids and values. The run-end flat form is close behind again,
about 5.7 and 3.3 megabytes. Only the materialized flat form is far off, 7.0 and
12.1 megabytes, because it repeats the shared resource attributes on the wire and
Arrow IPC does not fold that repetition away the way Parquet does.

This is the reconciliation with the companion analysis, which found Parquet
smaller on the wire than OTAP-standard for a single large batch. The same holds
here, by a smaller margin because the high-cardinality per-record data sets a
floor that both formats carry. An earlier pass of this study used identical
records and reported OTAP-standard many times smaller than any flat form, which
was an artifact of that degenerate data, not a real result. With realistic
records the four forms are within about a factor of two on the wire, apart from
the materialized flat form.

The CPU picture is where the forms separate. OTAP-standard and the run-end flat
form are the cheap pair. OTAP-standard decodes fastest, about 6 to 13
milliseconds, because it is a light Arrow IPC deserialize plus a transport
decode. The run-end flat form is often the cheapest to encode, and in the
resource-heavy scenario it encodes in 15 milliseconds against OTAP-standard's 28,
because it only flattens with a run-end layout and writes plain Arrow IPC,
whereas OTAP-standard pays for its transport optimization. Parquet is the
expensive outlier on both ends, 238 to 416 milliseconds to encode and 58 to 173
to decode, five to fifteen times the others, which is the same result the
companion analysis reported.

Two caveats remain. The per-record trace ids here are unique, which is the
high-cardinality end for logs; correlated logs that share a trace id across a
span would compress somewhat better and lower every wire number together. And to
write the flat batch to Parquet the study first materializes the encoder's
dictionary columns, because arrow-rs 58.3 cannot read a dictionary-encoded
`FixedSizeBinary` such as `trace_id` back from Parquet. Arrow IPC has no such
limit and carries those dictionaries directly.

The reason this matters for the original hypothesis is that a flat wire format is
not obviously worse. The run-end flat form is within about a quarter of
OTAP-standard on the wire, is sometimes cheaper to encode, and hands the receiver
a query-ready table with no projection step. Against that, OTAP-standard is
slightly smaller and decodes fastest and yields the normalized form, and Parquet
is the smallest at rest but by far the most expensive to produce and consume. So
the choice is a real trade rather than a rout. If the receiver wants columns and
values CPU, shipping the run-end flat form is defensible. If it wants the smallest
wire or the normalized model, or the cheapest decode, OTAP-standard is the better
default, and the flat view is then a cheap projection at the consumer.

## OTAP-flat as the natural center

The pipeline this study considers has five representations. OTAP-standard is the
four normalized batches, written `S`. OTAP-flat with run-end shared attributes is
the single batch, written `F`. Their serialized forms are OTAP/IPC-standard `Ws`
and OTAP/IPC-flat `Wf`, and the storage form is Parquet `P`. The question is what
it costs to move between them, and how much of each move is a genuine transform
rather than a copy of columns that do not change.

Each directed edge, timed in isolation:

| edge                               | log-heavy | resource-heavy |
| ---------------------------------- | --------: | -------------: |
| S  -> F   flatten to REE           |       6.1 |            1.7 |
| F  -> S   unflatten                |      12.9 |            2.9 |
| S  -> Ws  standard serialize       |      62.9 |           25.8 |
| Ws -> S   standard deserialize     |       9.9 |            4.5 |
| F  -> Wf  flat serialize           |      37.0 |           12.2 |
| Wf -> F   flat deserialize         |      16.1 |            6.4 |
| F  -> P   parquet-ready (REE+FSB)  |      13.8 |           58.7 |
| F  -> P   parquet write            |     144.9 |          242.8 |
| P  -> F   parquet read             |      52.8 |          140.1 |

Absolute milliseconds on this shared machine vary by up to about a factor of two
between runs, so the matrix should be read for the relative cost of the edges
rather than exact values. The relationships are stable. The two OTAP forms are
the cheapest pair to move between, a few milliseconds each way. Serializing
standard to its wire form is the most expensive of the Arrow IPC edges because it
runs the transport optimization, while serializing flat is lighter because it
only compresses. Parquet is the heavy end on both write and read.

### What actually changes, column by column

The flat table has thirteen columns: ten that come straight from the standard
`Logs` batch, which are `id`, `resource`, `scope`, the two timestamps,
`trace_id`, `span_id`, the two severity columns, and `body`, plus the three
attribute columns `resource_attributes`, `scope_attributes`, and
`log_attributes`. Classifying each conversion by how many of the thirteen it
copies unchanged versus how many it must rewrite:

| conversion  |  copied |  transformed | what changes                                               |
| ----------- | ------: | -----------: | ---------------------------------------------------------- |
| S <-> F     |      10 |            3 | attribute containers: keyed batches to REE/List columns    |
| F <-> P     |       9 |            4 | trace_id/span_id dict to plain; resource/scope REE to List |
| S <-> Ws    |       0 |           13 | transport-optimize sort, delta, dict, then compress        |
| F <-> Wf    |      13 |            0 | buffer compression and framing only                        |

`S` to `F` copies the ten `Logs` columns verbatim and only builds the three
attribute containers, wrapping the resource and scope batches as run-end columns
and the log attributes as a list. `F` to `P` copies nine columns, including the
dictionary `Utf8` and `Int32` columns and the log-attribute list, all of which
Parquet round-trips, and rewrites only four: it expands the two run-end columns
to plain lists and materializes `trace_id` and `span_id` from dictionary to
plain. That last pair is forced by the reader, not the writer, since arrow-rs can
write a dictionary of `FixedSizeBinary` but cannot read one back. The two
serialize edges are the opposite extremes. Standard to its wire form rewrites
every column, because the transport optimization sorts each batch and delta and
dictionary encodes it before compression. Flat to its wire form rewrites none of
them structurally, because plain Arrow IPC carries the run-end and dictionary
encodings as they are and only compresses the buffers.

### Why flat is the center

Put together, `F` sits between `S` and `P` and is a short hop from each. It shares
its ten `Logs` columns with `S`, so reaching it from `S` is a copy plus three
small container builds. It shares its whole schema with `P`, so reaching `P` from
`F` is a copy plus four column rewrites plus the writer. And its own wire form is
a light compress rather than a re-encode. No single move rewrites more than a
handful of columns, and the bulk of every move is memory sharing. That is what it
means for a representation to be a natural center. It is close to standard on one
side and close to Parquet on the other, and the conversions in every direction
touch only the columns that genuinely differ between the encodings.

### Where sort order and future support change the picture

One transform in the matrix is not cheap. Expanding the run-end resource and
scope columns for Parquet costs about 59 milliseconds in the resource-heavy
scenario, because it re-materializes the repetition that run-end encoding had
folded away. This is the memory and wire saving being paid back at export time.
Two changes would remove it. If arrow-rs learns to write run-end columns to
Parquet, the expansion becomes a passthrough from run-end runs to Parquet
run-length pages, and if its reader learns to read a dictionary of
`FixedSizeBinary`, the `trace_id` and `span_id` materialization disappears too.
At that point `F` to `P` is nearly all copy plus the writer. An asserted sort
order compounds this, because a flat batch that declares its order lets the
Parquet writer skip its own sort and produces meaningful row-group statistics,
and it keeps the run-end runs maximal. The sort is optional, and its value grows
after a merge or shuffle where the natural resource clustering has been broken.

### A direct standard-to-Parquet path, and its ordering precondition

Recognizing which columns are shared also yields a direct OTAP-standard to
Parquet path that skips the hash join the naive flatten performs. It copies the
ten `Logs` columns, attaches the log attributes as a `List<Struct>` whose struct
children are the existing `LogAttrs` value columns, and materializes the resource
and scope attributes as lists, then writes. The log-attribute attach is the
zero-copy step, and it works by reading contiguous `parent_id` runs, so it
depends on the attribute batches being grouped by `parent_id`.

That grouping is present in a freshly encoded batch but not in a
transport-optimized one, and the difference is not a matter of speed but of
correctness. The OTAP encoder emits each parent's attributes contiguously, so a
batch straight from OTLP has `LogAttrs.parent_id` as `[0, 0, ..., 1, 1, ...]`.
The transport optimization then sorts each attribute batch by `(type, key,
value, parent_id)` to compress the value columns, which scatters `parent_id`.
A probe confirms it: on the fresh batch the zero-copy flatten round-trips
exactly, while on the same batch after a wire round-trip the `parent_id` column
is no longer grouped and the zero-copy path fails its own precondition. A
converter that receives a transport-optimized batch must therefore regroup the
attributes by `parent_id` first, which is a stable sort, or fall back to the hash
join.

This splits the pipeline cleanly. A gateway that encodes OTLP to OTAP and writes
Parquet in the same place holds the fresh, `parent_id`-grouped batch and gets the
direct path for free, which is the precompute-at-the-gateway case. A service that
receives OTAP-standard off the wire holds a transport-optimized batch and pays a
regroup before the direct path applies. There is a tension worth naming: the
transport optimization sorts the `Logs` batch by `(resource, scope, trace_id)`,
which helps Parquet by clustering the low-cardinality columns, but it sorts the
attribute batches by key, which the flatten must undo. The attribute sort that
shrinks the standard wire is wasted, and then some, for a receiver that flattens
to Parquet.

### Measuring the optimized path against the naive one

For the common OTLP to batch to Parquet case the batch is fresh and grouped, so
the direct path applies. Timing it against the naive hash-join flatten, both
followed by the same parquet-ready transform and Parquet write, since both
produce byte-identical files:

| scenario       | flatten naive | flatten opt | prep+write | total naive | total opt |
| -------------- | ------------: | ----------: | ---------: | ----------: | --------: |
| log-heavy      |          47.7 |        10.7 |      168.5 |       216.3 |     179.3 |
| resource-heavy |          79.6 |        60.4 |      265.1 |       344.7 |     325.5 |

The optimized flatten is much cheaper on its own, from 47.7 to 10.7 milliseconds
in the log-heavy case, a bit over four times faster, and from 79.6 to 60.4 in the
resource-heavy case. But the Parquet prepare-and-write is the floor and is shared
by both paths, so the end-to-end saving is smaller than the flatten saving alone,
about seventeen percent log-heavy and six percent resource-heavy.

The gap between the two scenarios is the point. The zero-copy build only helps the
columns it can share, which are the per-record log attributes. Log-heavy has nine
of them, so avoiding the join and the full `take` removes most of the flatten
cost. Resource-heavy has only two log attributes but twenty resource attributes,
and those must be materialized per row for Parquet whichever path builds them, so
the optimized path saves the join overhead and the small log-attribute `take` but
still pays the resource materialization. In both cases the flatten is roughly a
fifth of the OTAP-to-Parquet total, so this refines the companion analysis: the
flatten tax is real and the shared-column build cuts it several fold, but the
Parquet writer sets a floor that leaves the end-to-end win in the single to low
double digits until the writer itself is made cheaper, for instance by the
run-end passthrough discussed above.

## What this means for the pipeline

The applied section of the companion analysis argued that when the gateway owns
both the exporter and the store, the OTAP to Parquet conversion can move to the
sending gateway. This study refines where the remaining cost lives and when it is
avoidable.

If the target is a Parquet file, the materialized single view is the right
intermediate, and the win available is the zero-copy log-attribute build, which
removed about three quarters of the conversion time in the log-heavy case without
changing the output. The resource and scope repetition still has to be written
out for the Parquet writer to consume, though the file it produces is no larger
for it.

If the target is an Arrow-native store answering queries, which is the serving
path for this system, the run-end or dictionary single view is dramatically
cheaper to build and to hold, and the advantage grows with the weight of the
shared attributes. Real resource attributes in production telemetry are numerous
and highly shared, so the resource-heavy scenario is the representative one for a
host or a gateway that aggregates many records under a small number of resources.
In that regime the run-end view is the most efficient single columnar
presentation of OTAP measured here, because it never materializes a value that
OTAP already stored once.

## Caveats and limits

The zero-copy log-attribute build relies on the encoder emitting attributes
grouped by `parent_id`, which the current OTAP producer does and which the probe
confirmed. This is a correctness precondition, not just a performance one: the
transport optimization re-sorts each attribute batch by key, so a
transport-optimized batch is not grouped by `parent_id` and the zero-copy path
must regroup it first or fall back to the hash join, as the standard-to-Parquet
subsection above details. The key column is normalized from its dictionary
encoding to plain `Utf8`, so that one column is cast rather than shared, while
the other value columns are shared unchanged.

The generated data models realistic telemetry with unique per-record ids and
mixed-type attributes, which is what keeps the wire comparison honest, since
identical rows collapse under dictionary and delta encoding and mislead. The wire
magnitudes still depend on cardinality: correlated logs that share a trace id
would compress better and lower every number together, so the comparison should
be read as one point on a spectrum rather than a fixed ratio.

Two arrow-rs 58.3 limits shape the results. The Parquet writer cannot serialize
run-end or nested-dictionary columns, so those layouts are in-memory forms only.
The Parquet reader cannot read a dictionary-encoded `FixedSizeBinary` such as
`trace_id` back, so the study materializes the encoder's dictionary columns
before writing Parquet. Both may change as arrow-rs adds support, at which point
the run-end view could also become a Parquet write target.

## Bottom line

OTAP attribute batches are already grouped by parent, so presenting them as a
single columnar view does not need a hash join. Building the log attributes zero
copy makes the materialized view, which is the one Parquet can write, about four
times cheaper to produce for log-heavy data while leaving the file identical. For
the shared resource and scope attributes, a run-end or dictionary view stores
each set once and is the most efficient single presentation, about twenty times
cheaper to build and eight times smaller in memory for resource-heavy data, at
the cost of not being directly writable to Parquet today. The choice follows the
consumer. Materialize for a Parquet file, and keep the run-end view for an
Arrow-native store that serves queries.

For moving data between two services the answer is a genuine trade rather than a
rout. On realistic data the four forms sit within about a factor of two on the
wire, apart from the materialized flat form, which repeats shared attributes and
falls behind. Parquet is the smallest at rest but by far the most expensive to
produce and consume. OTAP-standard is close on size, decodes fastest, and yields
the normalized model. The run-end flat form is close again on size, is sometimes
cheaper to encode than OTAP-standard, and hands the receiver a query-ready table.
So a flat wire format is defensible when the receiver wants columns and values
CPU, while OTAP-standard remains the better default for the smallest wire, the
cheapest decode, or the normalized model, with the flat view computed as a cheap
projection at the consumer.
