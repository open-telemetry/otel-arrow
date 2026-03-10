# OpenTelemetry Arrow Protocol (OTAP) Formal Specification

**Status:** Draft

## Table of Contents

1. [Introduction](#1-introduction)
2. [Data Model](#2-data-model)
3. [Transport and Service Definitions](#3-transport-and-service-definitions)
4. [IPC Stream Management and Schema Resets](#4-ipc-stream-management-and-schema-resets)
5. [Payload Specifications](#5-payload-specifications)
6. [Id Columns](#6-id-columns)
7. [Error Handling](#7-error-handling)

---

## 1. Introduction

### 1.1 Purpose

The OpenTelemetry Arrow Protocol (OTAP) defines a wire protocol for transmitting
OpenTelemetry telemetry signals (logs, metrics, and traces) from a Client to a
Server. OTAP optimizes on multiple axis for compression efficiency, memory
usage, and processing speed while being semantically equivalent to OpenTelemetry
Protocol (OTLP).

### 1.2 Requirements Language

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in
[RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

See [Appendix A: Glossary](#appendix-a-glossary) for terminology and
[Appendix B: References](#appendix-b-references) for reference material.

---

## 2. Data Model

### 2.1 Normalized Representation

OTAP represents telemetry Signals as a set of normalized tables connected by
foreign key relationships. Each Signal type has a different number and set of
tables reflecting the data transported by that Signal.

Each table within a Signal has a designated Payload Type that identifies it.
The foreign key relationships between these tables form a Rooted Directed
Acyclic Graph (DAG) with the Root Payload Type for each signal being the root
of that graph.

For example, the Logs signal type consists of four Payload Types: LOGS,
LOG_ATTRS, RESOURCE_ATTRS, and SCOPE_ATTRS. The LOGS table is the Root Payload
Type for Logs and fills a similar role as an OTLP Log. Each Log has an `id`
which identifies it, and links it to the LOG_ATTRS table which defines the
log's attributes. LOGS similarly contains `resource` and `scope` fields, each
having an `id` which links them to the RESOURCE_ATTRS and SCOPE_ATTRS tables
respectively.

The Metrics and Traces signals have a similar structure, but with more tables.
The allowed Payload Types for each signal can be found in section 3.

### 2.2 Apache Arrow and Columnar Representation

OTAP leverages the the
[Arrow Columnar Format](https://arrow.apache.org/docs/format/Columnar.html#arrow-columnar-format)
to transmit the payloads described in the previous section. The Arrow Columnar
Format defines a Serialization and Interprocess Communication (IPC) protocol for
exchanging data as "Record Batches".

Record Batches have a columnar data structure that can be exchanged and
processed without deserializing to a language specific construct.

Columnar data has advantages such as being beneficial for compression and
friendly to operate on with SIMD instruction sets.

## 3. Transport and Service Definitions

OTAP uses gRPC for its transport mechanism and defines data exchange as three
signal-specific
[service definitions](https://github.com/open-telemetry/otel-arrow/blob/main/proto/opentelemetry/proto/experimental/arrow/v1/arrow_service.proto):

```protobuf
service ArrowTracesService {
  rpc ArrowTraces(stream BatchArrowRecords) returns (stream BatchStatus) {}
}

service ArrowLogsService {
  rpc ArrowLogs(stream BatchArrowRecords) returns (stream BatchStatus) {}
}

service ArrowMetricsService {
  rpc ArrowMetrics(stream BatchArrowRecords) returns (stream BatchStatus) {}
}
```

Clients create an OTAP connection by intiating a gRPC connection and starting a
gRPC stream for one or more services. `BatchArrowRecords` (BAR) messages are
streamed from the client with corresponding `BatchStatus` acknowledgments
streamed back from the server. The bi-directional statefule streaming pattern
allows the client to continue sending BARs while waiting for acknowledgments,
which may come out of order enabling high throughput with backpressure control.

### 3.1 BatchArrowRecords

The BatchArrowRecords (BAR) message is the fundamental unit of data transmission
for the OTAP data model.

```protobuf
message BatchArrowRecords {
  // [REQUIRED] A unique identifier for this BAR within the current gRPC stream.
  // This ID is used by the server to send acknowledgments (BatchStatus messages)
  // and by the client to correlate those acknowledgments with sent BARs. The ID
  // space is scoped to a single gRPC stream connection.
  int64 batch_id = 1;

  // [REQUIRED] A collection of ArrowPayload messages, each containing the
  // serialized Arrow IPC data for one table.
  repeated ArrowPayload arrow_payloads = 2;

  // [OPTIONAL] Additional metadata transmitted alongside the BAR.
  // This field is typically used for authentication tokens, tracing context,
  // or other out-of-band metadata. If present, MUST be encoded using HPACK.
  // Servers MAY ignore this field.
  bytes headers = 3;
}
```

The `batch_id` is an identifier used for the server to ack or nack successful
receipt of the batch and MUST be unique within a gRPC stream.

The `headers` field can contain additional application level metadata and MUST
(TODO: MAY?) be HPACK compressed if present.

Each `ArrowPayload` in the `arrow_payloads` field contains data for one Payload
Type in the Signal, as indicated by the `type` field.

```protobuf
message ArrowPayload {
  // [REQUIRED] A unique identifier for the Arrow schema used for this payload type.
  // a change in this id for some payload type indicate a Schema Reset which requires
  // a reset of the underlying stream.
  string schema_id = 1;

  // [REQUIRED] An enum value identifying which table this payload represents
  // (e.g., LOGS, SPAN_ATTRS, NUMBER_DATA_POINTS).
  ArrowPayloadType type = 2;

  // [REQUIRED] The raw bytes containing one or more Apache Arrow Encapsulated
  // IPC Messages. These messages follow the Arrow IPC Streaming Format and
  // include Schema messages (for new schemas), DictionaryBatch messages (for
  // dictionary state), and RecordBatch messages (for the actual data rows).
  bytes record = 3;
}
```

The `schema_id` is an indicator of the schema being used for this Payload Type
and is discussed in more detail in section 4.

The `record` field contains the telemetry data for this Payload Type. Data for
each Payload Type is transmitted in this field as Arrow Record Batches using the
Apache Arrow IPC protocol. The `record` field MUST contain at least one valid
Encapsulated Arrow IPC message.

Allowed Payload Types are defined per gRPC service/Signal. Clients MUST only
send ArrowPayloads with allowed ArrowPayload types for that Signal according to
the tables defined in this section.

`arrow_payloads` additionally MUST include the primary/root table (LOGS, SPANS,
or UNIVARIATE_METRICS) and SHOULD omit payloads with 0 rows.

A `type` MUST NOT be sent as `UNKNOWN` (value 0).

#### 3.1.1 Logs Allowed Payload Types

| Payload Type   | Description                 | Parent Payload Type |
| -------------- | --------------------------- | ------------------- |
| LOGS           | Core log record data (Root) | -                   |
| LOG_ATTRS      | Log-level attributes        | LOGS                |
| RESOURCE_ATTRS | Resource attributes         | LOGS                |
| SCOPE_ATTRS    | Scope attributes            | LOGS                |

#### 3.1.2 Metrics Allowed Payload Types

| Payload Type                    | Description                                               | Parent Payload Type        |
| ------------------------------- | --------------------------------------------------------- | -------------------------- |
| UNIVARIATE_METRICS              | Definition of univariate metrics (Root)                   | -                          |
| NUMBER_DATA_POINTS              | Gauge and sum data points                                 | METRICS                    |
| SUMMARY_DATA_POINTS             | Summary data points                                       | METRICS                    |
| HISTOGRAM_DATA_POINTS           | Histogram data points                                     | METRICS                    |
| EXP_HISTOGRAM_DATA_POINTS       | Exponential histogram data points                         | METRICS                    |
| NUMBER_DP_ATTRS                 | Attributes for number data points                         | NUMBER_DATA_POINTS         |
| SUMMARY_DP_ATTRS                | Attributes for summary data points                        | SUMMARY_DATA_POINTS        |
| HISTOGRAM_DP_ATTRS              | Attributes for histogram data points                      | HISTOGRAM_DATA_POINTS      |
| EXP_HISTOGRAM_DP_ATTRS          | Attributes for exponential histogram data points          | EXP_HISTOGRAM_DATA_POINTS  |
| NUMBER_DP_EXEMPLARS             | Exemplars for number data points                          | NUMBER_DATA_POINTS         |
| HISTOGRAM_DP_EXEMPLARS          | Exemplars for histogram data points                       | HISTOGRAM_DATA_POINTS      |
| EXP_HISTOGRAM_DP_EXEMPLARS      | Exemplars for exponential histogram data points           | EXP_HISTOGRAM_DATA_POINTS  |
| NUMBER_DP_EXEMPLAR_ATTRS        | Exemplar attributes for number data points                | NUMBER_DP_EXEMPLARS        |
| HISTOGRAM_DP_EXEMPLAR_ATTRS     | Exemplar attributes for histogram data points             | HISTOGRAM_DP_EXEMPLARS     |
| EXP_HISTOGRAM_DP_EXEMPLAR_ATTRS | Exemplar attributes for exponential histogram data points | EXP_HISTOGRAM_DP_EXEMPLARS |
| METRIC_ATTRS                    | Metric-level attributes (metadata)                        | METRICS                    |
| RESOURCE_ATTRS                  | Resource attributes                                       | METRICS                    |
| SCOPE_ATTRS                     | Scope attributes                                          | METRICS                    |

#### 3.1.3 Traces Allowed Payload Types

| Payload Type     | Description           | Parent Payload Type |
| ---------------- | --------------------- | ------------------- |
| SPANS            | Core span data (Root) | -                   |
| SPAN_ATTRS       | Span attributes       | SPANS               |
| SPAN_EVENTS      | Span events           | SPANS               |
| SPAN_EVENT_ATTRS | Event attributes      | SPAN_EVENTS         |
| SPAN_LINKS       | Span links            | SPANS               |
| SPAN_LINK_ATTRS  | Link attributes       | SPAN_LINKS          |
| RESOURCE_ATTRS   | Resource attributes   | SPANS               |
| SCOPE_ATTRS      | Scope attributes      | SPANS               |

### 3.2 BatchStatus Acknowledgment

The `BatchStatus` message provides feedback from server to client about the
success or failure of processing a BAR. This acknowledgment mechanism enables
clients to track which BARs have been successfully processed and handle failures
for BARs that were rejected.

```protobuf
message BatchStatus {
  // [REQUIRED] The identifier of the BAR being acknowledged. This MUST match the
  // batch_id from the BatchArrowRecords message that was received. Unique within
  // the context of a single stream.
  int64 batch_id = 1;

  // [REQUIRED] Indicates whether processing succeeded or failed, and if failed,
  // what category of error occurred (e.g., invalid data, resource exhaustion,
  // authentication failure). MUST be a valid StatusCode (see section 7.2).
  StatusCode status_code = 2;

  // [OPTIONAL] Human-readable error details. For OK status, this is typically
  // empty. For errors, this provides context to help diagnose the issue (e.g.,
  // "dictionary key overflow in LOG_ATTRS table" or "unknown payload type: 99").
  // MAY provide additional context for non-OK statuses.
  string status_message = 3;
}
```

Servers MUST send BatchStatus messages to acknowledge all received BARs. The
status code MUST be set to OK to indicate a BAR was successfully accepted and
MUST be set to a Non-OK status code to indicate any error conditions. See
section 7.

### 3.3 Connection Lifecycle

The typical lifecycle for an OTAP connection is as follows:

1. Client establishes gRPC connection to server
2. Client initiates bi-directional stream for the appropriate signal type
3. Client sends a stream of BatchArrowRecords (BAR) messages and the Server
   responds with a stream of BatchStatus acknowledgements.
4. Connection persists until explicitly closed or an error occurs

## 4. IPC Stream Management and Schema Resets

This section defines how Clients and Servers manage Arrow IPC Streams
transmitted over an ArrowPayload's `record` field along with the related
mechanics of Schema Resets.

### 4.1 Arrow IPC Background

[Arrow IPC protocol](https://arrow.apache.org/docs/format/Columnar.html#serialization-and-interprocess-communication-ipc)
is defined by the Arrow project. While some details which are especially
relevant to OTAP are mentioned here, please refer to the full documentation on
Arrow IPC. If you are familiar with Arrow IPC you may want to skip this section.

Arrow IPC is a binary protocol for efficiently transmitting Record Batches which
share a Schema. It can be used with any transport and is defined in terms of a
one way stream of
[Encapsulated Arrow IPC Messages](https://arrow.apache.org/docs/format/Columnar.html#encapsulated-message-format)
which must be processed by the recipient in order.

A Schema defines a Record Batch's field names and types. Arrow supports many
types including primitive types like uint16, list types, struct types, and
types with special encodings such as Dictionary types.

Dictionary types can be used to save space for columns with repeated values. For
example, rather than storing a UTF8 (String) column with many duplicates, you
may instead store a Dictionary<U8,UTF8> which stores offsets into a separate
array of strings, thereby saving space. It's important to note that
Dictionary<U8, UTF8> is considered a different type from UTF8 rather than a
different encoding of the same type. Schemas which are otherwise identical, but
use these two types for the same field are NOT considered the same.

IPC Streams are stateful in order to reduce the amount of data that has to be
transmitted. The Schema of the Record Batches is written first as it's own
message and indicates the schema for all Record Batch messages to follow.

If the Schema contains Dictionary columns, then Dictionary Batches containing
the Dictionary values must be sent ahead of any Record Batch messages which
reference them. The Record batch messages then only need to include the offsets
into the values array. If the initial Dictionary message for a column did not
include all possible values which may be discovered after the Stream begins,
Delta Dictionary messages can be sent to inform the receiver of the new values.

### 4.2 OTAP Adaptive Schemas

To enable domain specific optimizations, OTAP is flexible in the Schemas that
it permits for Record Batches of a given Payload Type.

First, OTAP defines defines some columns for a Payload Type as nullable. Any
column marked as nullable is also considered "optional" and SHOULD be
omitted entirely by the Client if there is no data for that Column.

Additionally, there are a range of Data Types allowed for some columns. For
example, a LOGS payload's `resource.schema_url` field may have the `Utf8` type,
a `Dictionary<U8, Utf8>`, or a `Dictionary<U16, Utf8>` as appropriate to reduce
the size of transmitted data.

Finally, OTAP allows for the Schema to be updated in the middle of a gRPC stream
via a Schema Reset. This is a feature unique to OTAP. Once a Schema is
established for an Arrow IPC Stream, it cannot be updated and the IPC Stream
needs to be re-established. OTAP therefore defines a process for negotiating an
IPC Stream Reset is outlined in section 4.4 Schema Resets and IPC Stream
Management.

The full range of Schemas that are permitted for a given Payload Type are
described in Section 5.

### 4.3 Optimistic Schema Selection

Clients SHOULD take advantage of OTAP adaptive schemas and optimistically
Dictionary encode eligible columns. Clients that do this MUST detect Dictionary
overflow during the Stream and initiate a Schema Reset to upgrade key sizes or
change to the native type.

### 4.4 IPC Stream Management and Schema Resets

Within a gRPC Stream for some Signal, there is a separate Arrow IPC Stream for
each Payload Type transmitted by the Client.

Each IPC Stream is uniquely identified within a gRPC Stream by a combination of
an ArrowPayload's `type` and `schema_id` fields. The first time a Client sends
an ArrowPayload with a unique combination of these fields, it marks the start
of an Arrow IPC Stream for that Payload Type.

This means that the messages contained in the `record` field MUST contain a
Schema Message prior to any Dictionary, Delta Dictionary, or Record Batch
Messages in the Stream.

Servers MUST keep track of Arrow IPC Stream state per gRPC Stream and per
Payload Type and `schema_id` within the Stream.

When a Client changes the `schema_id` sent for some Payload Type, this is known
as a Schema Reset. The Server, upon receiving such an `ArrowPayload`, MUST
begin processing the Messages in the `record` field of that message as if they
begin a new IPC Stream.

### 4.5 Generating Schema Ids

- A `schema_id` SHOULD be derived deterministically from the schema structure
  (field names, types, and their ordering).
- A client SHOULD NOT generate multiple `schema_id` for the same schema as this
  will trigger wasteful Stream Resets.

See Appendix H on a sample algorithm to generate Schema IDs.

---

## 5. Payload Specifications

This section defines the set of allowable Arrow Schemas for all OTAP Payload
Types. Entities are organized by signal category with attributes for all tables
at the end.

**Table Column Descriptions:**

- **Name**: Field name in the Arrow schema
- **Native Type**: Base Arrow type for this field
- **Optimized Encodings**: Indicates acceptable optimized Arrow types allowed
  for this field (e.g., `Dict(u8)` for Dictionary(UInt8, Type), `List(T)` for
  list types). See also:
  [Dictionary Encoding](https://arrow.apache.org/docs/format/Columnar.html#dictionary-encoded-layout)
- **Nullable**: Whether the field can contain null values
- **Required**: Whether this field must be present in every record
- **Id Encoding**: The encoding method used for id columns (see
  [Section 6.4](#64-transport-optimized-encodings))
- **Description**: Human-readable description of the field's purpose

Note: For Columns which have a Struct type, there is one entry in the table
representing the definition of the struct Column e.g. `resource`. Then there are
additional entries in the table for each of their sub fields named according to
a `struct_column.struct_field` syntax. For example `resource.id` defines the
properties for the `resource` column's `id` field.

### 5.1 Logs

#### 5.1.1 LOGS (ROOT)

| Name                              | Native Type           | Optimized Encodings | Nullable | Required | Id Encoding                  | Metadata | Description                                       |
| --------------------------------- | --------------------- | ------------------- | -------- | -------- | ---------------------------- | -------- | ------------------------------------------------- |
| id                                | UInt16                | -                   | Yes      | No       | [DELTA](#642-delta-encoding) | encoding | Log record identifier (primary key)               |
| resource                          | Struct                | -                   | Yes      | No       | -                            | -        | Resource information                              |
| resource.id                       | UInt16                | -                   | Yes      | No       | [DELTA](#642-delta-encoding) | encoding | Foreign key to RESOURCE_ATTRS                     |
| resource.schema_url               | Utf8                  | Dict(u8), Dict(u16) | Yes      | No       | -                            | -        | Resource schema URL                               |
| resource.dropped_attributes_count | UInt32                | -                   | Yes      | No       | -                            | -        | Number of dropped resource attributes             |
| scope                             | Struct                | -                   | Yes      | No       | -                            | -        | Instrumentation scope                             |
| scope.id                          | UInt16                | -                   | Yes      | No       | [DELTA](#642-delta-encoding) | encoding | Foreign key to SCOPR_ATTRS                        |
| scope.name                        | Utf8                  | Dict(u8), Dict(u16) | Yes      | No       | -                            | -        | Instrumentation scope name                        |
| scope.version                     | Utf8                  | Dict(u8), Dict(u16) | Yes      | No       | -                            | -        | Instrumentation scope version                     |
| scope.dropped_attributes_count    | UInt32                | -                   | Yes      | No       | -                            | -        | Number of dropped scope attributes                |
| schema_url                        | Utf8                  | Dict(u8), Dict(u16) | Yes      | No       | -                            | -        | Log schema URL                                    |
| time_unix_nano                    | Timestamp(Nanosecond) | -                   | No       | Yes      | -                            | -        | Log timestamp in Unix nanoseconds                 |
| observed_time_unix_nano           | Timestamp(Nanosecond) | -                   | No       | Yes      | -                            | -        | Observation timestamp in Unix nanoseconds         |
| trace_id                          | FixedSizeBinary(16)   | Dict(u8), Dict(u16) | Yes      | No       | -                            | -        | Trace id for correlation                          |
| span_id                           | FixedSizeBinary(8)    | Dict(u8), Dict(u16) | Yes      | No       | -                            | -        | Span id for correlation                           |
| severity_number                   | Int32                 | Dict(u8), Dict(u16) | Yes      | No       | -                            | -        | Numeric severity level                            |
| severity_text                     | Utf8                  | Dict(u8), Dict(u16) | Yes      | No       | -                            | -        | Textual severity level                            |
| event_name                        | Utf8                  | Dict(u8), Dict(u16) | Yes      | No       | -                            | -        | Event name                                        |
| body                              | Struct                | -                   | Yes      | Yes      | -                            | -        | Log body                                          |
| body.type                         | UInt8                 | -                   | No       | Yes      | -                            | -        | Body value type (same encoding as attribute type) |
| body.str                          | Utf8                  | Dict(u16)           | Yes      | Yes      | -                            | -        | String body (may be empty)                        |
| body.int                          | Int64                 | Dict(u16)           | Yes      | No       | -                            | -        | Integer body (when body.type=3)                   |
| body.double                       | Float64               | -                   | Yes      | No       | -                            | -        | Double body (when body.type=4)                    |
| body.bool                         | Boolean               | -                   | Yes      | No       | -                            | -        | Boolean body (when body.type=2)                   |
| body.bytes                        | Binary                | Dict(u16)           | Yes      | No       | -                            | -        | Bytes body (when body.type=5)                     |
| body.ser                          | Binary                | Dict(u16)           | Yes      | No       | -                            | -        | CBOR-encoded complex body (when body.type=6 or 7) |
| dropped_attributes_count          | UInt32                | -                   | Yes      | No       | -                            | -        | Number of dropped log attributes                  |
| flags                             | UInt32                | -                   | Yes      | No       | -                            | -        | Trace flags                                       |

### 5.2 Traces

#### 5.2.1 SPANS (ROOT)

| Name                              | Native Type           | Optimized Encodings | Nullable | Required | Id Encoding                  | Metadata | Description                           |
| --------------------------------- | --------------------- | ------------------- | -------- | -------- | ---------------------------- | -------- | ------------------------------------- |
| id                                | UInt16                | -                   | Yes      | No       | [DELTA](#642-delta-encoding) | encoding | Span identifier (primary key)         |
| resource                          | Struct                | -                   | Yes      | No       | -                            | -        | Resource information                  |
| resource.id                       | UInt16                | -                   | Yes      | No       | [DELTA](#642-delta-encoding) | encoding | Foreign key to RESOURCE_ATTRS         |
| resource.schema_url               | Utf8                  | Dict(u8), Dict(u16) | Yes      | No       | -                            | -        | Resource schema URL                   |
| resource.dropped_attributes_count | UInt32                | -                   | Yes      | No       | -                            | -        | Number of dropped resource attributes |
| scope                             | Struct                | -                   | Yes      | No       | -                            | -        | Instrumentation scope                 |
| scope.id                          | UInt16                | -                   | Yes      | No       | [DELTA](#642-delta-encoding) | encoding | Foreign key to SCOPE_ATTRS            |
| scope.name                        | Utf8                  | -                   | Yes      | No       | -                            | -        | Instrumentation scope name            |
| scope.version                     | Utf8                  | -                   | Yes      | No       | -                            | -        | Instrumentation scope version         |
| scope.dropped_attributes_count    | UInt32                | -                   | Yes      | No       | -                            | -        | Number of dropped scope attributes    |
| schema_url                        | Utf8                  | -                   | Yes      | No       | -                            | -        | Span schema URL                       |
| start_time_unix_nano              | Timestamp(Nanosecond) | -                   | No       | Yes      | -                            | -        | Span start time in Unix nanoseconds   |
| duration_time_unix_nano           | Duration(Nanosecond)  | -                   | No       | Yes      | -                            | -        | Span duration in nanoseconds          |
| trace_id                          | FixedSizeBinary(16)   | -                   | No       | Yes      | -                            | -        | Trace id                              |
| span_id                           | FixedSizeBinary(8)    | -                   | No       | Yes      | -                            | -        | Span id                               |
| trace_state                       | Utf8                  | -                   | Yes      | No       | -                            | -        | W3C trace state                       |
| parent_span_id                    | FixedSizeBinary(8)    | -                   | Yes      | No       | -                            | -        | Parent span id                        |
| name                              | Utf8                  | -                   | No       | Yes      | -                            | -        | Span name                             |
| kind                              | Int32                 | -                   | Yes      | No       | -                            | -        | Span kind enum                        |
| dropped_attributes_count          | UInt32                | -                   | Yes      | No       | -                            | -        | Number of dropped span attributes     |
| dropped_events_count              | UInt32                | -                   | Yes      | No       | -                            | -        | Number of dropped events              |
| dropped_links_count               | UInt32                | -                   | Yes      | No       | -                            | -        | Number of dropped links               |
| status                            | Struct                | -                   | Yes      | No       | -                            | -        | Span status                           |
| status.code                       | Int32                 | Dict(u8), Dict(u16) | Yes      | No       | -                            | -        | Span status code                      |
| status.status_message             | Utf8                  | Dict(u8), Dict(u16) | Yes      | No       | -                            | -        | Status message                        |

#### 5.2.2 SPAN_EVENTS

| Name                     | Native Type           | Optimized Encodings | Nullable | Required | Id Encoding                                       | Metadata | Description                                         |
| ------------------------ | --------------------- | ------------------- | -------- | -------- | ------------------------------------------------- | -------- | --------------------------------------------------- |
| id                       | UInt32                | -                   | Yes      | No       | [DELTA](#642-delta-encoding)                      | encoding | Event identifier (primary key)                      |
| parent_id                | UInt16                | -                   | No       | Yes      | [QUASI-DELTA](#643-quasi-delta-encoding) (`name`) | encoding | Foreign key to [SPANS](#521-spans-root) `id` column |
| time_unix_nano           | Timestamp(Nanosecond) | -                   | Yes      | No       | -                                                 | -        | Event timestamp in Unix nanoseconds                 |
| name                     | Utf8                  | -                   | No       | Yes      | -                                                 | -        | Event name                                          |
| dropped_attributes_count | UInt32                | -                   | Yes      | No       | -                                                 | -        | Number of dropped event attributes                  |

#### 5.2.3 SPAN_LINKS

| Name                     | Native Type         | Optimized Encodings | Nullable | Required | Id Encoding                                         | Metadata | Description                                         |
| ------------------------ | ------------------- | ------------------- | -------- | -------- | --------------------------------------------------- | -------- | --------------------------------------------------- |
| id                       | UInt32              | -                   | Yes      | No       | [DELTA](#642-delta-encoding)                        | encoding | Link identifier (primary key)                       |
| parent_id                | UInt16              | -                   | No       | Yes      | [QUASI-DELTA](#643-quasi-delta-encoding) (trace_id) | encoding | Foreign key to [SPANS](#521-spans-root) `id` column |
| trace_id                 | FixedSizeBinary(16) | -                   | Yes      | No       | -                                                   | -        | Linked trace `id`                                   |
| span_id                  | FixedSizeBinary(8)  | -                   | Yes      | No       | -                                                   | -        | Linked span `id`                                    |
| trace_state              | Utf8                | -                   | Yes      | No       | -                                                   | -        | Linked trace state                                  |
| dropped_attributes_count | UInt32              | -                   | Yes      | No       | -                                                   | -        | Number of dropped link attributes                   |

### 5.3 Metrics

#### 5.3.1 UNIVARIATE_METRICS (ROOT)

| Name                              | Native Type | Optimized Encodings | Nullable | Required | Id Encoding                  | Metadata | Description                                    |
| --------------------------------- | ----------- | ------------------- | -------- | -------- | ---------------------------- | -------- | ---------------------------------------------- |
| id                                | UInt16      | -                   | No       | Yes      | [DELTA](#642-delta-encoding) | encoding | Metric identifier (primary key)                |
| resource                          | Struct      | -                   | Yes      | No       | -                            | -        | Resource information                           |
| resource.id                       | UInt16      | -                   | Yes      | No       | [DELTA](#642-delta-encoding) | encoding | Foreign key to RESOURCE_ATTRS                  |
| resource.schema_url               | Utf8        | -                   | Yes      | No       | -                            | -        | Resource schema URL                            |
| resource.dropped_attributes_count | UInt32      | -                   | Yes      | No       | -                            | -        | Number of dropped resource attributes          |
| scope                             | Struct      | -                   | Yes      | No       | -                            | -        | Instrumentation scope information              |
| scope.id                          | UInt16      | -                   | Yes      | No       | [DELTA](#642-delta-encoding) | encoding | Foreign key to SCOPE_ATTRS                     |
| scope.name                        | Utf8        | -                   | Yes      | No       | -                            | -        | Instrumentation scope name                     |
| scope.version                     | Utf8        | -                   | Yes      | No       | -                            | -        | Instrumentation scope version                  |
| scope.dropped_attributes_count    | UInt32      | -                   | Yes      | No       | -                            | -        | Number of dropped scope attributes             |
| schema_url                        | Utf8        | -                   | Yes      | No       | -                            | -        | Metric schema URL                              |
| metric_type                       | UInt8       | -                   | No       | Yes      | -                            | -        | Metric type enum (Gauge, Sum, Histogram, etc.) |
| name                              | Utf8        | -                   | No       | Yes      | -                            | -        | Metric name                                    |
| description                       | Utf8        | -                   | Yes      | No       | -                            | -        | Metric description                             |
| unit                              | Utf8        | -                   | Yes      | No       | -                            | -        | Metric unit                                    |
| aggregation_temporality           | Int32       | -                   | Yes      | No       | -                            | -        | Aggregation temporality enum                   |
| is_monotonic                      | Boolean     | -                   | Yes      | No       | -                            | -        | Whether the metric is monotonic                |

#### 5.3.2 NUMBER_DATA_POINTS

| Name                 | Native Type           | Optimized Encodings | Nullable | Required | Id Encoding                  | Metadata | Description                                   |
| -------------------- | --------------------- | ------------------- | -------- | -------- | ---------------------------- | -------- | --------------------------------------------- |
| id                   | UInt32                | -                   | Yes      | No       | [DELTA](#642-delta-encoding) | encoding | Data point identifier (primary key)           |
| parent_id            | UInt16                | -                   | No       | Yes      | [DELTA](#642-delta-encoding) | encoding | Foreign key to UNIVARIATE_METRICS `id` column |
| start_time_unix_nano | Timestamp(Nanosecond) | -                   | No       | Yes      | -                            | -        | Start time in Unix nanoseconds                |
| time_unix_nano       | Timestamp(Nanosecond) | -                   | No       | Yes      | -                            | -        | Timestamp in Unix nanoseconds                 |
| int_value            | Int64                 | -                   | No       | Yes      | -                            | -        | Integer value                                 |
| double_value         | Float64               | -                   | No       | Yes      | -                            | -        | Double value                                  |
| flags                | UInt32                | -                   | Yes      | No       | -                            | -        | Data point flags                              |

#### 5.3.3 SUMMARY_DATA_POINTS

| Name                 | Native Type           | Optimized Encodings | Nullable | Required | Id Encoding                  | Metadata | Description                                   |
| -------------------- | --------------------- | ------------------- | -------- | -------- | ---------------------------- | -------- | --------------------------------------------- |
| id                   | UInt32                | -                   | Yes      | No       | [DELTA](#642-delta-encoding) | encoding | Data point identifier (primary key)           |
| parent_id            | UInt16                | -                   | No       | Yes      | [DELTA](#642-delta-encoding) | encoding | Foreign key to UNIVARIATE_METRICS `id` column |
| start_time_unix_nano | Timestamp(Nanosecond) | -                   | Yes      | No       | -                            | -        | Start time in Unix nanoseconds                |
| time_unix_nano       | Timestamp(Nanosecond) | -                   | Yes      | No       | -                            | -        | Timestamp in Unix nanoseconds                 |
| count                | UInt64                | -                   | Yes      | No       | -                            | -        | Count of observations                         |
| sum                  | Float64               | -                   | Yes      | No       | -                            | -        | Sum of observations                           |
| quantile             | List(Struct)          | -                   | Yes      | No       | -                            | -        | List of quantil values                        |
| quantile[].quantile  | Float64               | Float64             | Yes      | No       | -                            | -        | Quantile quantile                             |
| quantile[].value     | Float64               | Float64             | Yes      | No       | -                            | -        | Quantile value                                |
| value                | Float64               | List(Float64)       | Yes      | No       | -                            | -        | Quantile observation values                   |
| flags                | UInt32                | -                   | Yes      | No       | -                            | -        | Data point flags                              |

#### 5.3.4 HISTOGRAM_DATA_POINTS

| Name                 | Native Type           | Optimized Encodings | Nullable | Required | Id Encoding                  | Metadata | Description                                   |
| -------------------- | --------------------- | ------------------- | -------- | -------- | ---------------------------- | -------- | --------------------------------------------- |
| id                   | UInt32                | -                   | Yes      | No       | [DELTA](#642-delta-encoding) | encoding | Data point identifier (primary key)           |
| parent_id            | UInt16                | -                   | No       | Yes      | [DELTA](#642-delta-encoding) | encoding | Foreign key to UNIVARIATE_METRICS `id` column |
| start_time_unix_nano | Timestamp(Nanosecond) | -                   | No       | No       | -                            | -        | Start time in Unix nanoseconds                |
| time_unix_nano       | Timestamp(Nanosecond) | -                   | Yes      | No       | -                            | -        | Timestamp in Unix nanoseconds                 |
| count                | UInt64                | -                   | Yes      | No       | -                            | -        | Count of observations                         |
| sum                  | Float64               | -                   | Yes      | No       | -                            | -        | Sum of observations                           |
| bucket_counts        | List(UInt64)          | -                   | Yes      | No       | -                            | -        | Count per bucket                              |
| explicit_bounds      | List(Float64)         | -                   | Yes      | No       | -                            | -        | Histogram bucket boundaries                   |
| flags                | UInt32                | -                   | Yes      | No       | -                            | -        | Data point flags                              |
| min                  | Float64               | -                   | Yes      | No       | -                            | -        | Minimum value                                 |
| max                  | Float64               | -                   | Yes      | No       | -                            | -        | Maximum value                                 |

#### 5.3.5 EXP_HISTOGRAM_DATA_POINTS

| Name                   | Native Type           | Optimized Encodings | Nullable | Required | Id Encoding                  | Metadata | Description                                   |
| ---------------------- | --------------------- | ------------------- | -------- | -------- | ---------------------------- | -------- | --------------------------------------------- |
| id                     | UInt32                | -                   | Yes      | No       | [DELTA](#642-delta-encoding) | encoding | Data point identifier (primary key)           |
| parent_id              | UInt16                | -                   | No       | Yes      | [DELTA](#642-delta-encoding) | encoding | Foreign key to UNIVARIATE_METRICS `id` column |
| start_time_unix_nano   | Timestamp(Nanosecond) | -                   | Yes      | No       | -                            | -        | Start time in Unix nanoseconds                |
| time_unix_nano         | Timestamp(Nanosecond) | -                   | Yes      | No       | -                            | -        | Timestamp in Unix nanoseconds                 |
| count                  | UInt64                | -                   | Yes      | No       | -                            | -        | Count of observations                         |
| sum                    | Float64               | -                   | Yes      | No       | -                            | -        | Sum of observations                           |
| scale                  | Int32                 | -                   | Yes      | No       | -                            | -        | Exponential histogram scale                   |
| zero_count             | UInt64                | -                   | Yes      | No       | -                            | -        | Count of zero values                          |
| positive               | Struct                | -                   | Yes      | No       | -                            | -        | Positive data                                 |
| positive.offset        | Int32                 | -                   | Yes      | No       | -                            | -        | Positive bucket offset                        |
| positive.bucket_counts | List(UInt64)          | -                   | Yes      | No       | -                            | -        | Positive bucket counts                        |
| negative               | Struct                | -                   | Yes      | No       | -                            | -        | Negative data                                 |
| negative.offset        | Int32                 | -                   | Yes      | No       | -                            | -        | Negative bucket offset                        |
| negative.bucket_counts | List(UInt64)          | -                   | Yes      | No       | -                            | -        | Negative bucket counts                        |
| flags                  | UInt32                | -                   | Yes      | No       | -                            | -        | Data point flags                              |
| min                    | Float64               | -                   | Yes      | No       | -                            | -        | Minimum value                                 |
| max                    | Float64               | -                   | Yes      | No       | -                            | -        | Maximum value                                 |

#### 5.3.6 NUMBER_DP_EXEMPLARS / HISTOGRAM_DP_EXEMPLARS / EXP_HISTOGRAM_DP_EXEMPLARS

Applies to: NUMBER_DP_EXEMPLARS, HISTOGRAM_DP_EXEMPLARS, EXP_HISTOGRAM_DP_EXEMPLARS

| Name           | Native Type           | Optimized Encodings | Nullable | Required | Id Encoding                                                            | Metadata | Description                                                  |
| -------------- | --------------------- | ------------------- | -------- | -------- | ---------------------------------------------------------------------- | -------- | ------------------------------------------------------------ |
| id             | UInt32                | -                   | Yes      | No       | [DELTA](#642-delta-encoding)                                           | encoding | Exemplar identifier (primary key)                            |
| parent_id      | UInt32                | Dict(u8), Dict(u16) | No       | Yes      | [QUASI-DELTA](#643-quasi-delta-encoding) (`int_value`, `double_value`) | encoding | Foreign key to the corresponding \*\_DATA_POINTS `id` column |
| time_unix_nano | Timestamp(Nanosecond) | -                   | Yes      | No       | -                                                                      | -        | Timestamp in Unix nanoseconds                                |
| int_value      | Int64                 | Dict(u8), Dict(u16) | Yes      | No       | -                                                                      | -        | Integer exemplar value                                       |
| double_value   | Float64               | -                   | Yes      | No       | -                                                                      | -        | Double exemplar value                                        |
| span_id        | FixedSizeBinary(8)    | Dict(u8), Dict(u16) | Yes      | No       | -                                                                      | -        | Associated span id                                           |
| trace_id       | FixedSizeBinary(16)   | Dict(u8), Dict(u16) | Yes      | No       | -                                                                      | -        | Associated trace id                                          |

### 5.4 Attributes

#### 5.4.1 U32 Attributes

Applies to: SPAN_EVENT_ATTRS / SPAN_LINK_ATTRS / NUMBER_DP_ATTRS /
SUMMARY_DP_ATTRS / HISTOGRAM_DP_ATTRS / EXP_HISTOGRAM_DP_ATTRS /
NUMBER_DP_EXEMPLAR_ATTRS / HISTOGRAM_DP_EXEMPLAR_ATTRS /
EXP_HISTOGRAM_DP_EXEMPLAR_ATTRS

| Name      | Native Type | Optimized Encodings | Nullable | Required | Id Encoding                              | Metadata | Description                                                                     |
| --------- | ----------- | ------------------- | -------- | -------- | ---------------------------------------- | -------- | ------------------------------------------------------------------------------- |
| parent_id | UInt32      | Dict(u8), Dict(u16) | No       | Yes      | [QUASI-DELTA](#643-quasi-delta-encoding) | encoding | Foreign key to the corresponding \*\_DP_EXEMPLARS `id` column                   |
| key       | Utf8        | Dict(u8), Dict(u16) | No       | Yes      | -                                        | -        | Attribute key name                                                              |
| type      | UInt8       | -                   | No       | Yes      | -                                        | -        | Value type: 0=Empty, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str       | Utf8        | Dict(u16)           | Yes      | No       | -                                        | -        | String value (when type=1)                                                      |
| int       | Int64       | Dict(u16)           | Yes      | No       | -                                        | -        | Integer value (when type=3)                                                     |
| double    | Float64     | -                   | Yes      | No       | -                                        | -        | Double value (when type=4)                                                      |
| bool      | Boolean     | -                   | Yes      | No       | -                                        | -        | Boolean value (when type=2)                                                     |
| bytes     | Binary      | Dict(u16)           | Yes      | No       | -                                        | -        | Bytes value (when type=5)                                                       |
| ser       | Binary      | Dict(u16)           | Yes      | No       | -                                        | -        | CBOR-encoded Array or Map (when type=6 or 7)                                    |

#### 5.4.2 U16 Attributes

Applies to: RESOURCE_ATTRS / SCOPE_ATTRS / LOG_ATTRS / METRIC_ATTRS / SPAN_ATTRS

| Name      | Native Type | Optimized Encodings | Nullable | Required | Id Encoding                              | Metadata | Description                                                                     |
| --------- | ----------- | ------------------- | -------- | -------- | ---------------------------------------- | -------- | ------------------------------------------------------------------------------- |
| parent_id | UInt16      | -                   | No       | Yes      | [QUASI-DELTA](#643-quasi-delta-encoding) | encoding | Foreign key to parent table's `resource.id`, `scope.id`, or `id` column         |
| key       | Utf8        | Dict(u8), Dict(u16) | No       | Yes      | -                                        | -        | Attribute key name                                                              |
| type      | UInt8       | -                   | No       | Yes      | -                                        | -        | Value type: 0=Empty, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str       | Utf8        | Dict(u16)           | Yes      | No       | -                                        | -        | String value (when type=1)                                                      |
| int       | Int64       | Dict(u16)           | Yes      | No       | -                                        | -        | Integer value (when type=3)                                                     |
| double    | Float64     | -                   | Yes      | No       | -                                        | -        | Double value (when type=4)                                                      |
| bool      | Boolean     | -                   | Yes      | No       | -                                        | -        | Boolean value (when type=2)                                                     |
| bytes     | Binary      | Dict(u16)           | Yes      | No       | -                                        | -        | Bytes value (when type=5)                                                       |
| ser       | Binary      | Dict(u16)           | Yes      | No       | -                                        | -        | CBOR-encoded Array or Map (when type=6 or 7)                                    |

### 5.5 Special Field Rules

#### 5.5.1 "AnyValue" Unions

OpenTelemetry defines the concept of an
["AnyValue"](https://opentelemetry.io/docs/specs/otel/common/#anyvalue).
This is a value representing a union of one or more types:
`string`, `int`, `double`, `bool`, `map`, `slice`, or `bytes`.

In OTAP this is implemented via multiple "value" columns either `str`, `int`,
`double`, `bool`, `bytes`, or `ser`. `type` field that serves as the
discriminant and note that multiple AnyValue types can map to the `bytes`
column. See the below table for the exact mappings.

This technique is used in all Payload Types that utilize Attribute16 and
Attribute32 schemas as well as the LOGS `body` field.

The "Active Field" is defined as the field that contains valid data for the
row, all other value fields MUST be ignored for that row.

The mappings between `type` discriminants and Active Fields is as follows:

| type value | AnyValue type | Active Field |
| ---------- | ------------- | ------------ |
| 0          | empty         | N/A          |
| 1          | str           | str          |
| 2          | int           | int          |
| 3          | double        | double       |
| 4          | bool          | bool         |
| 5          | map           | ser          |
| 6          | slice         | ser          |
| 7          | bytes         | bytes        |

The `ser` types for `map` (5) and `slice` (6) additionally indicate that the
field is CBOR encoded according to RFC 8949. If the type is `bytes` (7)
then the structure of the field is unknown at the OTAP level.

A `type` of `0` indicates that the value for the attribute is empty.

In the case of the LOGS payload, note that the AnyValue is contained in the
`body` column which is also nullable. In this case a null entry is semantically
equivalent to a `type` of `0`.

If the column for the Active Field is not present, then the value for the key
is also interpreted as empty.

If the `type` falls outside of the allowed range (0-7), then the data is
considered invalid and SHOULD be rejected.

---

## 6. Id Columns

This section defines more details related to identifier columns used to
establish relationships between payload types in the OTAP data model.

### 6.1 Primary Keys and Foreign Keys

Most parent-child relationships in the OTAP data model follow a uniform
convention. Note Resource and Scope entities deviate slightly from these
conventions, see section 6.3.

- **Parent/Entity tables** define an `id` column as their primary key
- **Child tables** define a `parent_id` column as a foreign key that always
  references their parent table's `id` column
- All `id` columns are nullable, with a null indicating that there are no child
  rows
- `parent_id` columns are not nullable as they must be linked back to some
  parent `id` column
- `id` columns are only unique within a BAR, Ids may be reused across batches
  in the same stream

**Example**: In the Logs signal:

- The LOGS table has an `id` column (UInt16)
- The LOG_ATTRS table has a `parent_id` column (UInt16) that references
  LOGS.`id`
- Each LOG_ATTRS row belongs to exactly one LOGS row via this foreign key

### 6.2 Id Column Types

Id columns use unsigned integer types sized according to expected cardinality.
`id` columns are either u32 or u16 and they define the primary keys of the
parent table. As such they are always unique within a Record Batch and do not
benefit from dictionary encoding, so they MUST NOT use a dictionary type.

On the other hand, child `parent_id` columns referencing u32 `id` columns of
their parents MAY use dictionary encoding with either `u8` or `u16` keys to
save space.

### 6.3 Resource and Scope Identifiers

Resource and scope entities are **not** represented as separate payload types.
Instead, they are embedded as struct fields within root tables (LOGS, SPANS,
or METRICS).

Resources and scopes are defined implicitly by their presence in root table
rows and can be shared among items of the same type. This gives them some
special characteristics:

1. There is a many-to-many relationship relationship between
   RESOURCE_ATTRS/SCOPE_ATTRS tables and their parent payload types
2. The corresponding column in the LOGS/METRICS/SPANS tables for
   RESOURCE_ATTRS.parent_id and SCOPE_ATTRS.parent_id are `resource.id` and
   `scope.id` respectively rather than just `id`.
3. Unlike other identifiers, `resource.id` and `scope.id` have no single table
   that "owns" them and defines the valid set of Ids.

### 6.4 Transport Optimized Encodings

OTAP defines specialized column encodings that MAY be used to transform `id`
and `parent_id` columns before serialization to maximize compression efficiency
during network transport.

Id columns often exhibit strong sequential patterns:

- Primary IDs are usually sequential (0, 1, 2, 3...)
- Foreign keys (`parent_id`) are often clustered (many attributes reference the
  same parent item)
- When sorted by `parent_id`, related records appear together

By encoding these patterns explicitly we create long runs of small integers
and repeated values that compress extremely well.

The encoding MAY be indicated explicitly in the field's metadata with one of
the following values:

| Key        | Values                               | Meaning                    |
| ---------- | ------------------------------------ | -------------------------- |
| `encoding` | `"plain"`, `"delta"`, `"quasidelta"` | Indicated encoding applied |

If there is no value listed, the column is by default encoded using the
appropriate delta encoding type for that column. Which fields use which
encodings are listed in summarized in 6.4.3, but also listed completely in the
Payload Specifications in section 5.

#### 6.4.1 Plain Encoding

The column contains literal values. No decoding needed.

#### 6.4.2 Delta Encoding

The entire column is delta-encoded. Decode by computing a prefix sum - each
decoded value equals the previous decoded value plus the current encoded value.
This is used on `id` columns and on `parent_id` columns of data point batches
(where the parent IDs are already sorted).

Null values are excluded from the delta encoding.

### Example

```txt
| encoded | decoded |
|---------|---------|
|    0    |    0    |  <- first value is always absolute
|    0    |    0    |  <- 0 + 0 = 0
|    1    |    1    |  <- 0 + 1 = 1
|    0    |    1    |  <- 1 + 0 = 1
|    0    |    1    |  <- 1 + 0 = 1
|    1    |    2    |  <- 1 + 1 = 2
|    1    |    3    |  <- 2 + 1 = 3
```

#### 6.4.3 Quasi-delta Encoding

Each value is _either_ a delta from the previous row or an absolute value,
depending on whether certain "equality columns" in the same batch match the
previous row.

Record batches are sorted by all three equality columns in order. When they
match, accumulate the delta. When they differ, reset to the absolute value.

The quasi-delta columns for each Payload Type are defined by the OTAP
specification and are always the same. The columns are summarized in the below
table and also indicated in the Payload Specifications in section 5.

| Payload types        | Equality columns                  |
| -------------------- | --------------------------------- |
| All `*Attrs` batches | `type`, `key`, the "Active Field" |
| `SpanEvents`         | `name`                            |
| `SpanLinks`          | `trace_id`                        |
| `*DpExemplars`       | `int_value`, `double_value`       |

#### Attribute Quasidelta Encoding

The attribute batch has one value column per attribute type and the`type`
column identifies which value column holds the attribute value for a given row.
See 5.5.1 "Any Value". Delta encoding is applied to runs of rows where type,
key, and value all match, with two exceptions: Map and Slice types are never
delta encoded, and neither are rows with null values.

```txt
 type      | key    | str  | int  | encoded parent_id | decoded parent_id
-----------|--------|------|------|---------|---------
 1 (str)   | "k1"   | "v1" | null |  1      |  1      <- absolute
 1 (str)   | "k1"   | "v1" | null |  1      |  2      <- type,key,val match prev -> delta: 1 + 1 = 2
 1 (str)   | "k1"   | "v1" | null |  1      |  3      <- type,key,val match prev -> delta: 2 + 1 = 3
 1 (str)   | "k1"   | "v2" | null |  1      |  1      <- val changed -> reset to absolute: 1
 1 (str)   | "k1"   | "v2" | null |  1      |  2      <- type,key,val match prev -> delta: 1 + 1 = 2
 1 (str)   | "k2"   | "v2" | null |  1      |  1      <- key changed -> reset to absolute: 1
 1 (str)   | "k2"   | "v2" | null |  1      |  2      <- type,key,val match prev -> delta: 1 + 1 = 2
 1 (str)   | "k2"   | "v2" | null |  1      |  3      <- type,key,val match prev -> delta: 2 + 1 = 3
 2 (int)   | "k2"   | "v2" | 1    |  1      |  1      <- type changed -> reset to absolute: 1
 2 (int)   | "k2"   | "v2" | 1    |  1      |  2      <- type,key,val match prev -> delta: 1 + 1 = 2
 ...
 0 (empty) | null   | null | null |  1      |  1      <- type = empty -> never delta encoded
 ...
 5 (map)   | null   | null | null |  1      |  1      <- type = map -> never delta encoded
 ...
 6 (slice) | null   | null | null |  1      |  1      <- type = slice -> never delta encoded
```

### Non-attribute Quasidelta Encoding

For non-attributes tables, equality columns are defined per table. For example,
SPAN_EVENTS uses a quasidelta encoding with just its `name` column.

```txt
| name | encoded parent_id | decoded parent_id |
|------|-------------------:|------------------:|
| "a"  |                 0  |                0  | <- first value is always absolute
| "a"  |                 1  |                1  | <- name matches prev -> delta: 0 + 1 = 1
| "a"  |                 1  |                2  | <- name matches prev -> delta: 1 + 1 = 2
| "a"  |                 2  |                4  | <- name matches prev -> delta: 2 + 2 = 4
| "b"  |                 0  |                0  | <- name differs -> reset to absolute: 0
| "b"  |                 3  |                3  | <- name matches prev -> delta: 0 + 3 = 3
```

---

## 7. Error Handling

### 7.1 Status Codes

Error codes are returned as a part of BatchStatus messages and map to their
[gRPC counterparts](https://grpc.io/docs/guides/status-codes/).

```protobuf
enum StatusCode {
  OK = 0;
  CANCELED = 1;
  INVALID_ARGUMENT = 3;
  DEADLINE_EXCEEDED = 4;
  PERMISSION_DENIED = 7;
  RESOURCE_EXHAUSTED = 8;
  ABORTED = 10;
  INTERNAL = 13;
  UNAVAILABLE = 14;
  UNAUTHENTICATED = 16;
}
```

---

## Appendix A: Glossary

- **Apache Arrow IPC Format**:
  <https://arrow.apache.org/docs/format/Columnar.html#ipc-streaming-format>
- **BAR**: Abbreviation for BatchArrowRecords, the client gRPC message
- **Client/Producer**: The sender of telemetry data
- **gRPC**: <https://grpc.io/>
- **Items**: The item type of a Signal e.g. Log, Data Point(s), or Span
- **OTLP Specification**: OpenTelemetry Protocol specification
- **Payload**: An ArrowPayload containing serialized Arrow IPC messages
- **Payload Type**: Also referred to as ArrowPayloadType, this is equivalent to
  a distinct table in the OTAP data model
- **Root Payload/Root Payload Type**: The root table in the Signal's DAG
- **Schema Reset**: The act of changing the Arrow schema for a Payload Type
- **Server/Consumer**: The receiver of telemetry data
- **Signal**: One of Logs, Metrics, or Traces

---

## Appendix B: References

1. Apache Arrow IPC Format: <https://arrow.apache.org/docs/format/Columnar.html>
2. OTLP Specification: <https://opentelemetry.io/docs/specs/otlp/>
3. gRPC Status Codes: <https://grpc.io/docs/guides/status-codes/>
4. OTEP 0156: <https://github.com/open-telemetry/oteps/blob/main/text/0156-columnar-encoding.md>
5. Reference Implementation (Go):
   [Producer](https://github.com/open-telemetry/otel-arrow/blob/main/pkg/otel/arrow_record/producer.go),
   [Consumer](https://github.com/open-telemetry/otel-arrow/blob/main/pkg/otel/arrow_record/consumer.go)
6. Rust Implementation: otap-dataflow/crates/pdata
7. RFC 2119: <https://www.rfc-editor.org/rfc/rfc2119>

---

## Appendix C: Load Balancing

OTAP's stateful, long-lived gRPC streams introduce load-balancing challenges
that do not arise with stateless unary RPCs. Because gRPC multiplexes streams
over a single HTTP/2 connection, L4 (TCP-level) load balancers distribute work
at connection granularity, not per-stream. Combined with kernel `SO_REUSEPORT`
hashing, too few client connections can pin traffic to a single backend core.

For a detailed treatment of challenges, solution techniques (client-side and
server-side), and recommended baseline configurations, see
[Load Balancing: Challenges & Solutions](rust/otap-dataflow/docs/load-balancing.md).

---

## Appendix E: Example Schema ID Generation Algorithm

1. Sort fields by name at each nesting level
2. Generate compact representation:
   - Field name
   - `:` separator
   - Type abbreviation (e.g., `U16` for UInt16, `Str` for Utf8, `Dic<U16,Str>`
     for Dictionary)
3. Concatenate fields with `,` separator

**Example**: `id:U16,parent_id:U16,key:Str,type:U8,str:Dic<U16,Str>`
