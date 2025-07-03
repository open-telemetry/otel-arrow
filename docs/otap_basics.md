# Open Telemetry Arrow Protocol (OTAP) Introduction

This document is meant to be an introduction to the Open Telemetry Arrow 
Protocol (OTAP). It is not a full technical specification, but enumerates the
major requirements of clients and servers communicating over OTAP along with
mechanical details of Schema resets and evolution. If you are inexperienced 
with OTAP and looking to familiarize yourself with the major components and 
mechanisms, then this is a good place to start.

It may also be helpful to consult the reference implementation of the protocol
while reading this document or vice versa:

- [Producer](https://github.com/open-telemetry/otel-arrow/blob/a429ef2e3d436e7c110fb312f2105341732fa233/pkg/otel/arrow_record/producer.go)
- [Consumer](https://github.com/open-telemetry/otel-arrow/blob/a429ef2e3d436e7c110fb312f2105341732fa233/pkg/otel/arrow_record/consumer.go)

This document does not revisit the motivations of technology choices like Apache
Arrow in detail. For that information, the following may be helpful:

- [OTEP 0156: Columnar Encoding](https://github.com/open-telemetry/oteps/blob/main/text/0156-columnar-encoding.md)
- [OTAP Validation Process](https://github.com/open-telemetry/otel-arrow/blob/main/docs/validation_process.md)
- [OTAP Benchmarks](https://github.com/open-telemetry/otel-arrow/blob/main/docs/benchmarks.md)
- F5 Journey with Arrow 
[Part 1](https://arrow.apache.org/blog/2023/04/11/our-journey-at-f5-with-apache-arrow-part-1/) 
and [Part 2](https://arrow.apache.org/blog/2023/06/26/our-journey-at-f5-with-apache-arrow-part-2/)

This document also assumes basic familiarity with the OpenTelemetry Data model
and OpenTelemetry Protocol (OTLP).

- [Logs data model](https://opentelemetry.io/docs/specs/otel/logs/data-model/)
- [Metrics data model](https://opentelemetry.io/docs/specs/otel/metrics/data-model/)
- [OTLP](https://opentelemetry.io/docs/specs/otlp/)

## Basic Description

OTAP is a sort of "protocol on top of a protocol". At the outer layer is a gRPC
service defined via protobuf. Within that we have Apache Arrow Interprocess 
Communication (Arrow IPC). OTAP leverages both it's own mechanisms at the gRPC
layer and existing Arrow IPC mechanisms to reliably transport telemetry signals
from a Client to a Server.

Before diving into the specifics of Arrow IPC and the intersection with the
gRPC streams that OTAP defines, we'll start with a high level overview of the 
data model and transport.

### Data model

Like its predecessor [OTLP](https://opentelemetry.io/docs/specs/otlp/), OTAP is
concerned with the transport of the Logs, Metrics, and Traces _signals_
that we know and love over the wire from a Client to a Server. The semantic 
model and meaning of these signals is independent of the format of the data, 
and OTAP makes a few different choices from OTLP in this regard.

OTAP opts for a normalized representation which spreads an OTLP signal across
multiple tables. These tables are described in the [data_model](./data_model).
Logs, for example, are split into four tables:

1. A primary Logs table that roughly corresponds to an OTLP 
[LogRecord](https://github.com/open-telemetry/opentelemetry-proto/blob/189b2648d29aa6039aeb262c0417ae56572e738d/opentelemetry/proto/logs/v1/logs.proto#L136C1-L137C1)
1. A Log Attributes table that corresponds to the 
[LogRecord attributes](https://github.com/open-telemetry/opentelemetry-proto/blob/189b2648d29aa6039aeb262c0417ae56572e738d/opentelemetry/proto/logs/v1/logs.proto#L177C3-L177C66)
1. A Resource Attributes table that corresponds to the 
[ResourceLogs resource attributes](https://github.com/open-telemetry/opentelemetry-proto/blob/189b2648d29aa6039aeb262c0417ae56572e738d/opentelemetry/proto/logs/v1/logs.proto#L53C3-L53C57)
1. A Scope Attributes table that corresponds to the 
[ScopeLogs scope attributes](https://github.com/open-telemetry/opentelemetry-proto/blob/189b2648d29aa6039aeb262c0417ae56572e738d/opentelemetry/proto/logs/v1/logs.proto#L72C3-L72C59)

The primary Logs table has foreign keys to each of the other three tables that
allows them to be joined together to reconstruct a complete Logs signal. Metrics
and Traces are similarly represented, though with more tables.

This is how we will think of Logs, Metrics, and Traces for the remainder of this
document.

### Transport

To transmit this data model, OTAP is defined in terms of a 
[gRPC service](https://github.com/open-telemetry/otel-arrow/blob/5b0da3dab952ad7e8196ffab00d59b27655fce76/proto/opentelemetry/proto/experimental/arrow/v1/arrow_service.proto#L49C1-L63C1).
Clients establish a persistent, stateful connection to a server and send a stream
of [BatchArrowRecords](https://github.com/open-telemetry/otel-arrow/blob/5b0da3dab952ad7e8196ffab00d59b27655fce76/proto/opentelemetry/proto/experimental/arrow/v1/arrow_service.proto#L66C1-L76C1).
The stateful nature of this communication will be described in more details in
later sections.

Each `BatchArrowRecords` contains a complete set of telemetry data for one 
particular signal in the form of multiple [ArrowPayloads](https://github.com/open-telemetry/otel-arrow/blob/5b0da3dab952ad7e8196ffab00d59b27655fce76/proto/opentelemetry/proto/experimental/arrow/v1/arrow_service.proto#L66C1-L76C1).
For example, a batch of logs would contain four payloads representing the four tables
of Logs, Log Attributes, Resource Attributes, and Scope Attributes.

> Note: If any of the tables are empty, for example if there are no Scope
Attributes set, the ArrowPayload for that table can be omitted.

As the name suggests, within each ArrowPayload is some Arrow data - Serialized 
[Encapsulated Arrow IPC Message(s)](https://arrow.apache.org/docs/format/Columnar.html#encapsulated-message-format)
 located in the [bytes field](https://github.com/open-telemetry/otel-arrow/blob/5b0da3dab952ad7e8196ffab00d59b27655fce76/proto/opentelemetry/proto/experimental/arrow/v1/arrow_service.proto#L135).
This is where the table data resides. Which table is represented by each Arrow Payload
is indicated by the 
[ArrowPayloadType](https://github.com/open-telemetry/otel-arrow/blob/5b0da3dab952ad7e8196ffab00d59b27655fce76/proto/opentelemetry/proto/experimental/arrow/v1/arrow_service.proto#L79C1-L80C1).

> Note: There may be more than one Encapsulated Arrow IPC message within the 
`bytes` of an Arrow Payload. More details below!

## Apache Arrow Primer

As mentioned earlier, OTAP is a sort of "protocol on top of a protocol" where
gRPC wraps Arrow IPC. This section will explain some key aspects of Arrow and 
Arrow IPC before we put everything together and look at some requests end to end.

Arrow is a deep topic in itself, you can refer to the 
[full manual](https://arrow.apache.org/docs/format/Intro.html)
on Apache Arrow for more details.

### Basics

Apache Arrow offers a language agnostic way to represent data such that it can
be shared between different systems without copying. Languages receive a byte
array that contains data formatted according to some Schema and rather than 
deserializing to a language specific struct/object equivalent type, the data can 
be read and operated on in-place.

Something different about the way that Arrow represents data compared to a 
typical struct or object is that the data is in a columnar format. This type
of format groups all of the values for a particular column in memory next to each 
other. [This article](https://arrow.apache.org/blog/2023/04/11/our-journey-at-f5-with-apache-arrow-part-1/)
from F5 has a great diagram comparing row and columnar data.

Typically data laid out in this way is beneficial for compression and also for
operation with SIMD instruction sets.

### Schemas and Encodings

In order for one machine to do anything interesting with those Arrow byte arrays 
coming from another machine, they need to know the Schema of that data. The Schema
defines the fields of the data, their types, and the order in which they appear.
This is strictly defined within Arrow such that there is enough information in
a Schema to process any column within the byte array.

One of the key features of Arrow is that the same data can be encoded in 
different ways to optimize its size. For example, a column can be _dictionary_
encoded. In a dictionary encoding, instead of writing out every value we can write
an integer key that is used to look up the value in a separate dictionary. This 
can be highly effective in data that has lots of repeated values e.g. a column 
whose values come from an enum.

The thing to highlight is such a column _could_ be encoded as a dictionary,
it doesn't _have_ to be encoded that way. And furthermore there can be different
dictionary encodings for the same data. You can imagine some data with lower
cardinality can make use of 8-bit integer keys, while some data with higher
cardinality might need 16-bit integer keys to avoid overflow. There are multiple
valid encodings for the same data and which to use is highly dependent on the 
characteristics of the data being transported. 
[This article](https://arrow.apache.org/blog/2023/04/11/our-journey-at-f5-with-apache-arrow-part-1/)
 from F5 has more details on considerations for picking an encoding.

Because Telemetry data varies wildly between domains, it's impossible to pick
a single encoding that will be near optimal for the entire world. OTAP provides
the flexibility required to find and use a good encoding for _any_ system. This
is discussed in more detail in the OTAP Client/Server sections.

### Interprocess Communication (IPC)

Unlike with protobuf, another advantage of the Arrow format is that clients and 
servers do not have to be aware ahead of time of the schema of the data being 
transmitted. How these schemas are negotiated is defined via the 
[Apache Arrow IPC Streaming format](https://arrow.apache.org/docs/format/Columnar.html#ipc-streaming-format).

This format is modeled as a one way stream of messages from Client to Server.
The types of messages that Clients can send and the order in which they are 
allowed to send them ensure that the Server has the information it needs to process 
the data. There are three kinds of so called 
[Encapsulated Messages](https://arrow.apache.org/docs/format/Columnar.html#encapsulated-message-format)
that can appear in this stream:

- Schema - Contains the schema of the messages that will follow
- Dictionary Batch - Contains dictionaries that can be used to interpret data
passed in the Record Batch
- Record Batch - Contains a shard of data (e.g. 100 rows) that follow the Schema

These messages must come in a particular order, the rules are:

- Schema must come first and only once
- Dictionary Batches are optional, but if dictionaries are used then they must
be transmitted before any Record Batches that need them are transmitted.
- Record Batches can come as needed and be interleaved with Dictionary Batches

Why are dictionaries not a part of the schema, and why can we interleave them
with Record Batches? Efficiency.

Once a dictionary encoding for some column is agreed upon, the server can simply
remember that dictionary and the client never has to send it again. That means
a string column containing what could be many bytes of data can be completely 
reduced to a column of (potentially very small) integers from that point on, 
yielding massive savings on network bandwidth.

In some cases, a Client will not know the full set of values that a column can
have at the outset. You can imagine a scraper that is collecting Kubernetes pod
logs and passes along the name of the pod as a resource attribute. Suppose a
dictionary encoding was chosen for these attributes because the cardinality of 
the pod names is relatively small.

When new pods come online, we don't have entries for them in the dictionary. We
could re-create our connection to the server and re-transmit the full schema and
dictionary with the new set of values, but this is wasteful and could happen
quite often. Instead we can communicate to the server that there are some new 
values for it to be aware of. These arrive in new Dictionary Batches that contain 
so called _Delta Dictionaries_ with just the new entries.

### Summary 

Apache Arrow allows systems to communicate structured data without knowing schemas
ahead of time. It allows for efficient encoding of that data via dictionaries
which can be updated on the fly as needed. The
[Apache Arrow IPC Streaming format](https://arrow.apache.org/docs/format/Columnar.html#ipc-streaming-format)
defines the mechanics of how this process works including the types of messages
that can be sent and the order that they must appear. This is inherently a
stateful process, and persistent connections are used to transmit many batches
of data between a client and server efficiently.

## OTAP Clients 

This section is going to walk through from start to finish the major things a 
client needs to do to create OTAP requests. 

For now, don't worry about the mechanics for constructing Arrow IPC messages. In 
practice we would use an existing library such as 
[arrow-go](https://github.com/apache/arrow-go).
To understand the protocol, it's enough to know _what_ you want to create. To see
it in practice, you can take a look at the `Producer` reference implementation
linked at the top of this document.

For simplicity and readablility we'll take some liberties in this section like 
trimming down the required fields. You can refer back to the data model for a 
full accounting of required and optional fields for every table.

The scenario is we're running a web application that logs some events such as
when a user logs in or a database connection fails. We're going to export
those events to an OTAP endpoint.

Here are a couple of sample log records. These are modeled as normalized tables 
consistent with OTAP and for readability are presented as JSON.

We have two log records:

```json
[
    {
      "id": 0,
      "time_unix_nano": 1719763200000000000,
      "body_str": "User login successful"
    },
    {
      "id": 1,
      "time_unix_nano": 1719763260000000000,
      "body_str": "Database connection failed"
    }
]
```

These log records each have some attributes:

```json
[
    {
      "parent_id": 0,
      "key": "user.id",
      "str": "user-1"
    },
    {
      "parent_id": 0,
      "key": "http.status_code",
      "int": 200
    },
    {
      "parent_id": 1,
      "key": "error.type",
      "str": "ConnectionTimeoutError"
    },
]
```

Our sample application is not instrumented with any resource or scope information, 
so we omit that data.

### Selecting a Schema

The first thing that we have to do is determine the encoding we want to use for
each column and translate that to a schema. To keep it simple we'll use a very 
direct schema for the Logs table and omit any dictionaries. Fields and types are 
as follows:

- id: u16
- time_unix_nano: timestamp
- body_str: string

For the attributes, however, we want to efficiently represent the `user.id` 
attribute because we attach that information to most logs. So we will choose
to represent the `str` column as a dictionary with 16 bit integer keys. Fields
and types are as follows:

- parent_id: u16
- key: string
- str: Dictionary<u16, string>
- int: i64

> Note that the attribute `key` and log `body_str` fields are also usually 
Dictionary encoded in practice and this is the default behavior of the reference 
implementation. The attributes produced by an application are often repeated and 
limited in cardinality. Log bodies are also often repeated with the variable 
parts of the message extracted out to attributes. This makes Dictionary encodings 
often a good choice for these fields.

### Constructing the protobuf envelope

Before sending our data off to the server, we need to construct the protobuf
envelope as described in the 
[OTAP gRPC definition](https://github.com/open-telemetry/otel-arrow/blob/main/proto/opentelemetry/proto/experimental/arrow/v1/arrow_service.proto#L79C1-L80C1).

The outermost structure is a `BatchArrowRecords`. Filling out the `batch_id` is 
straightforward, we can start at `0` and increment for each batch. `headers` 
we'll omit as optional. That leaves the `arrow_payloads`, which we need one of
per table that we're sending.

### ArrowPayload for the Logs Table

Let's start with the Logs table. Every `ArrowPayload` needs a `schema_id`, a
`type`, and a `record` containing Encapsulated Arrow IPC Messages.

For the `schema_id` we have just one schema for this table so far, so we'll set
`schema_id: "0"`. For `type`, we use the corresponding `ArrowPayloadType` enum
and in this case we're transmitting a `LOGS` table. Now we need to figure out 
the `record`.

Recall the
[Apache Arrow IPC Streaming format](https://arrow.apache.org/docs/format/Columnar.html#ipc-streaming-format)
from earlier. The rules state that the first message must be a schema. 

Following the schema we could add a Dictionary Batch if we needed it, but we
don't have dictionary encodings for this logs table, so there's no need.

Following Schema batches and any pre-requisite DictionaryBatch(s) we can add 
RecordBatch messages containing our data. 

So in the first `ArrowPayload` for the `LOGS` table, we will send two Encapsulated
Arrow IPC messages within the `record` body as follows:

```
------------------------
| Schema | RecordBatch |
------------------------
```

### ArrowPayload for the Log Attributes table

Next up is the Log Attributes table. The same ideas apply for the `schema_id`,
and `type`. This time for the `type` we'll use `LOG_ATTRS`. 

For the `record` we need to start again with a Schema message, but since we also
have dictionary encodings for the `str` column, we need to create a DictionaryBatch
message. The mechanics of tracking the set of keys and values for a particular
column are an implementation detail and out of scope for this document. Following
the DictionaryBatch, we can include RecordBatch messages.

So, in the first `ArrowPayload` for the `LOG_ATTRS` table, we will send three 
Encapsulated Arrow IPC messages within the `record` body as follows:

```
------------------------------------------
| Schema | DictionaryBatch | RecordBatch |
------------------------------------------
```

### gRPC vs Arrow IPC streams

At this point we have our `BatchArrowRecords` wrapping our two `ArrowPayloads` 
and we're ready to send them off to the server. This is a good time to discuss
the relationship between gRPC and Arrow IPC streams in OTAP.

At the gRPC level our communication is happening over a single _gRPC_ stream of
`BatchArrowRecords`. However within that gRPC stream, we have multiple independent 
_Apache Arrow IPC_ streams, one for each `ArrowPayloadType`. The number of _Arrow_
streams that we have to juggle per _gRPC_ stream is dependent on how many tables
we need to represent the telemetry signal. For a Logs signal that's a maximum
of four different Arrow streams, but for Metrics or Traces that could be more.

### Updating the dictionaries

Suppose a second user logs in. We have the following log record to export:

Logs:

```jsonc
[
    {
      "id": 2,
      "time_unix_nano": 1719764000000000000,
      "body_str": "User login successful"
    }
]
```

Attributes:

```json
[
    {
      "parent_id": 2,
      "key": "user.id",
      "str": "user-2"
    },
    {
      "parent_id": 0,
      "key": "http.status_code",
      "int": 200
    },
]
```

How do we construct the Protobuf envelope this time? 

To start we need to bump the `batch_id` to `1` because this is a new batch. Once
again we omit the `headers` as optional, but need two `arrow_payloads` for our
`LOGS` and `LOG_ATTRS` tables.

Handling ArrowPayload for the `LOGS` table is easy. We've already established the 
Schema for the _arrow_ stream and there's been no change, so the `schema_id` 
remains `"0"`. The only message we need is a single RecordBatch message 
containing the new log. The `record` field then looks like this:

```
---------------
| RecordBatch |
---------------
```

When constructing the ArrowPayload for the `LOG_ATTRS` table we have some more
work to do. We have opted to dictionary encode the `str` column in this table,
however this is the first time that we've seen the `user-2` value, so our 
dictionaries that we sent to the server are missing an entry for it. 

This can be handled on the _Arrow_ stream level via Delta Dictionaries.

With the new ArrowPayload all we have to do is include a Delta Dictionary message
with the key/value for `user-2` prior to the RecordBatch message. So we will
pack two messages as follows in the `record`:

```
---------------------------------
| DeltaDictionary | RecordBatch |
---------------------------------
```

Now the server will process the DeltaDictionary and update its lookup table for
that column before processing the RecordBatch which has the new key. Note that 
our Arrow Schema remains the same as before, we only updated the lookup table. 
So our `schema_id` remains `"0"` for this ArrowPayload.

### Resetting the Schema

Updating our Arrow stream with delta dictionaries will work great for the first
65 thousand or so users, but recall that we're using unsigned 16-bit integers 
for our dictionary keys.

At some point we'll have too many users and experience dictionary overflow. Then
the current Schema can no longer be used. When that happens we need to pick a new 
Schema, e.g swap the dictionary to use 32-bit integer keys, and signal that to 
the server.

However there is no mechanism within _Arrow_ streams to do this. Remember that
Schemas must be sent only a single time at the start of an Arrow stream. Instead
this is handled by OTAP within the gRPC stream via a _Schema Reset_.

For the sake of the example, let's assume our app had a big traffic spike and 
this happens after we send batch `9`.

At this point we're familiar with the mechanics of bumping the `batch_id` 
(now `10`) and starting a couple of `arrow_payloads`. For our `LOGS` table, 
nothing new happens here because the dictionary is for the `LOG_ATTRIBUTES` table. 
We keep our `schema_id` of `"0"` on this ArrowPayload and it's business as usual.
The `record` field is a single RecordBatch message once again: 

```
---------------
| RecordBatch |
---------------
```

For the `LOG_ATTRIBUTES` table we're first going to pick a new schema - This time
we'll update the `str` column to use 32-bit keys instead of 16. Because of this
we now bump the `schema_id` to `"1"` for this ArrowPayload. This tells the server 
that we're going to reset the inner _Arrow_ stream completely for the 
`LOG_ATTRIBUTES` table and start from scratch.

Because we're re-creating the Arrow stream from scratch, we need to start again 
with a Schema message and any DictionaryBatch messages prior to the RecordBatch
messages. So our `record` field for this ArrowPayload is going to look the same
as it did for batch `"0"`:

```
------------------------------------------
| Schema | DictionaryBatch | RecordBatch |
------------------------------------------
```

### Summary

OTAP consists of a gRPC stream which wraps multiple Apache Arrow IPC streams.
Certain evolutions of the schema can be handled in the normal course of Arrow IPC 
by using Delta Dictionary messages. Whenever we have a need to change Arrow Schemas
completely, however, there is no mechanism within Arrow IPC to do that. Instead
we signal to the server that we need to reset the underlying Arrow IPC stream
for some table by bumping the `schema_id` on the ArrowPayload and starting over 
with a Schema message.

## OTAP Servers

- TODO:
