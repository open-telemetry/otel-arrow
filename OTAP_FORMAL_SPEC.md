# OpenTelemetry Arrow Protocol (OTAP) Formal Specification

**Version:** 0.1
**Status:** Draft

## Table of Contents

1. [Introduction](#1-introduction)
2. [Data Model](#2-data-model)
3. [Protocol Architecture](#3-protocol-architecture)
5. [Payload Specifications](#5-payload-specifications)
6. [Id Columns](#6-id-columns)
7. [Schema Management](#7-schema-management)
8. [Error Handling](#8-error-handling)
9. [Field Specifications](#9-field-specifications)
10. [Compliance Requirements](#10-compliance-requirements)

---

## 1. Introduction

### 1.1 Purpose

The OpenTelemetry Arrow Protocol (OTAP) defines a wire protocol for transmitting OpenTelemetry telemetry signals (logs, metrics, and traces) using Apache Arrow's columnar format wrapped in gRPC streams. OTAP is a column oriented protocol that optimizes for compression efficiency, memory usage, and CPU performance while being semantically equivalent to OpenTelemetry Protocol (OTLP).

### 1.2 Requirements Language

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

See [Appendix F: Glossary](#appendix-f-glossary) for terminology and [Appendix G: References](#appendix-g-references) for reference material.

---

## 2. Data Model

### 2.1 Normalized Representation

OTAP represents telemetry Signals as a set of normalized tables connected by foreign key relationships; 
effectively a database. Each Signal type has a different number and set of tables reflecting the data 
transported by that Signal. 

Each table within a Signal has a designated Payload Type that identifies it. The foreign key
relationships between these tables form a Rooted Directed Acyclic Graph (DAG) with the 
Root Payload Type being the root of that graph. 

For example, the Logs signal type consists of four Payload Types: LOGS, LOG_ATTRS, RESOURCE_ATTRS, 
and SCOPE_ATTRS. The LOGS table is the Root Payload Type for Logs and fills a similar role as an OTLP Log. 
Each Log has a unique `id` which identifies it, and links it to the LOG_ATTRS table which defines the log's
attributes. LOGS similarly contains `resource` and `scope` fields, each having an `id`  which links them 
to the RESOURCE_ATTRS and SCOPE_ATTRS tables.

The Metrics and Traces signals have a similar structure, but with more tables and are defined 
below along with the relationships between each table.

#### 2.1.1 Logs Signal Tables

| Payload Type | Description | Parent Payload Type |
|---|---|---|
| LOGS | Core log record data (Root) | — |
| LOG_ATTRS | Log-level attributes | LOGS |
| RESOURCE_ATTRS | Resource attributes | LOGS |
| SCOPE_ATTRS | Scope attributes | LOGS |

#### 2.1.2 Metrics Signal Tables

| Payload Type | Description | Parent Payload Type |
|---|---|---|
| UNIVARIATE_METRICS | Core metric metadata (Root) | — |
| MULTIVARIATE_METRICS | Core metric metadata (Root) | — |
| NUMBER_DATA_POINTS | Gauge and sum data points | METRICS |
| SUMMARY_DATA_POINTS | Summary data points | METRICS |
| HISTOGRAM_DATA_POINTS | Histogram data points | METRICS |
| EXP_HISTOGRAM_DATA_POINTS | Exponential histogram data points | METRICS |
| NUMBER_DP_ATTRS | Attributes for number data points | NUMBER_DATA_POINTS |
| SUMMARY_DP_ATTRS | Attributes for summary data points | SUMMARY_DATA_POINTS |
| HISTOGRAM_DP_ATTRS | Attributes for histogram data points | HISTOGRAM_DATA_POINTS |
| EXP_HISTOGRAM_DP_ATTRS | Attributes for exponential histogram data points | EXP_HISTOGRAM_DATA_POINTS |
| NUMBER_DP_EXEMPLARS | Exemplars for number data points | NUMBER_DATA_POINTS |
| HISTOGRAM_DP_EXEMPLARS | Exemplars for histogram data points | HISTOGRAM_DATA_POINTS |
| EXP_HISTOGRAM_DP_EXEMPLARS | Exemplars for exponential histogram data points | EXP_HISTOGRAM_DATA_POINTS |
| NUMBER_DP_EXEMPLAR_ATTRS | Exemplar attributes for number data points | NUMBER_DP_EXEMPLARS |
| HISTOGRAM_DP_EXEMPLAR_ATTRS | Exemplar attributes for histogram data points | HISTOGRAM_DP_EXEMPLARS |
| EXP_HISTOGRAM_DP_EXEMPLAR_ATTRS | Exemplar attributes for exponential histogram data points | EXP_HISTOGRAM_DP_EXEMPLARS |
| METRIC_ATTRS | Metric-level attributes | METRICS |
| RESOURCE_ATTRS | Resource attributes | METRICS |
| SCOPE_ATTRS | Scope attributes | METRICS |

#### 2.1.3 Traces Signal Tables

| Payload Type | Description | Parent Payload Type |
|---|---|---|
| SPANS | Core span data (Root) | — |
| SPAN_ATTRS | Span attributes | SPANS |
| SPAN_EVENTS | Span events | SPANS |
| SPAN_EVENT_ATTRS | Event attributes | SPAN_EVENTS |
| SPAN_LINKS | Span links | SPANS |
| SPAN_LINK_ATTRS | Link attributes | SPAN_LINKS |
| RESOURCE_ATTRS | Resource attributes | SPANS |
| SCOPE_ATTRS | Scope attributes | SPANS |

---

## 3. Protocol Architecture

OTAP consists of three distinct layers:

1. **gRPC Layer**: Bi-directional streaming RPC services for each signal type
2. **OTAP Message Layer**: BatchArrowRecords and ArrowPayload protobuf messages
3. **Arrow IPC Layer**: Apache Arrow Interprocess Communication streams

### 3.1 gRPC Layer

The gRPC layer is the outermost layer providing the transport mechanism and service definitions. 
It establishes bi-directional streaming connections between clients and servers over HTTP/2. 


#### 3.1.1 Service Definitions

OTAP defines signal-specific gRPC services in the [protobuf definition](https://github.com/open-telemetry/otel-arrow/blob/main/proto/opentelemetry/proto/experimental/arrow/v1/arrow_service.proto):
Each service accepts a stream of BatchArrowRecords (BAR) messages from the client and returns a stream of BatchStatus
acknowledgments. The bi-directional streaming pattern allows the client to continue sending BARs while waiting for
acknowledgments, enabling high throughput with backpressure control. The OTAP Message Layer places further restrictions
on the contents of BatchArrowRecords per service.

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

#### 3.1.2 Connection Lifecycle

The typical connection lifecycle for the service is as follows:

1. Client establishes gRPC connection to server
2. Client initiates bi-directional stream for appropriate signal type
3. Client sends stream of BatchArrowRecords (BAR) messages
4. Server processes each BAR and returns BatchStatus acknowledgments
5. Connection persists until explicitly closed or error occurs

#### 3.1.3 BatchStatus Acknowledgment

The BatchStatus message provides feedback from server to client about the success or failure of 
processing a BAR. This acknowledgment mechanism enables clients to track which BARs have been 
successfully processed and handle failures for BARs that were rejected.

```protobuf
message BatchStatus {
  // [REQUIRED] The identifier of the BAR being acknowledged. This MUST match the
  // batch_id from the BatchArrowRecords message that was received.
  int64 batch_id = 1;

  // [REQUIRED] Indicates whether processing succeeded or failed, and if failed,
  // what category of error occurred (e.g., invalid data, resource exhaustion,
  // authentication failure). MUST be a valid StatusCode (see section 8.2).
  StatusCode status_code = 2;

  // [OPTIONAL] Human-readable error details. For OK status, this is typically
  // empty. For errors, this provides context to help diagnose the issue (e.g.,
  // "dictionary key overflow in LOG_ATTRS table" or "unknown payload type: 99").
  // MAY provide additional context for non-OK statuses.
  string status_message = 3;
}
```

BatchStatus messages flow from server to client over the same bi-directional gRPC stream, allowing 
the server to acknowledge BARs as they are processed and potentially out of order. 

A status code of OK indicates the BAR was successfully received, decoded, and accepted. Non-OK status
codes indicate various error conditions which may or may not be retriable (see section 8 for details).

Servers MUST send BatchStatus messages to acknowledge received BARs.

### 3.2 OTAP Message Layer

The OTAP message layer defines additional restrictions and requirements for the contents of a BAR,
the ArrowPayload, and BatchStatus messages. It defines which Payload Types are valid for which Services/Signals; rules
around Schema Evolution, Schema Resets, and Error Handling; and when it is allowable to omit payloads
entirely.

#### 3.2.1 BatchArrowRecords Message

The BatchArrowRecords (BAR) message is the fundamental unit of data transmission in OTAP. It represents 
a complete set of related telemetry tables for a single signal type, containing all the tables needed 
to reconstruct that signal.

Each BAR is assigned a unique identifier that allows the server to acknowledge receipt and report errors
on a per-BAR basis.

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

**Requirements**

- `batch_id`s MUST be unique within the gRPC stream
- `batch_id`s MUST be strictly increasing // NEEDS_TRIAGE
- `arrow_payloads` MUST include the primary table e.g. LOGS, SPANS, or UNIVARIATE_METRICS/MULTIVARIATE_METRICS // NEEDS_TRIAGE
- `arrow_payloads` SHOULD omit tables with 0 rows
- `headers` MUST be HPACK compressed if present // NEEDS TRIAGE

#### 3.2.2 ArrowPayload Message

An ArrowPayload encapsulates the serialized Arrow IPC data for a single table within a BAR. 
Each payload is tagged with the Arrow Payload Type it represents and a schema identifier
which is  scoped to the Stream and payload type indicating the Arrow Schema used for the 
contents. See Section 7 for more details on the schema_id and the various mechanics surrounding
schema management.

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

**Requirements**

- Once sent, a `schema_id` MUST always indicate the same schema for a given payload type within a
given Stream.
- A `schema_id` MAY be derived from the schema structure (field names, types, and their ordering).
- `type` MUST NOT be sent as `UNKNOWN` (value 0) // NEEDS_TRIAGE
- `record` MUST contain at least one valid Encapsulated Arrow IPC message // NEEDS_TRIAGE: We could just ignore it if it doesn't

### 3.3 Arrow IPC Layer

The Arrow IPC (Interprocess Communication) layer is the innermost layer containing the actual telemetry data 
in Apache Arrow's columnar format. Arrow IPC defines how schemas and data are serialized into byte streams using 
a standardized format that can be read by any Arrow-compatible library. This layer enables:

- **Dictionary/Delta Dictionary encoding**: Efficient representation of repeated string values
- **Zero-copy deserialization**: Data can be read directly from wire format without copying
- **Columnar layout**: Data organized by column rather than by row, enabling better compression and SIMD processing

Clients and servers communicating over a Stream are managing multiple independent Arrow IPC streams,
one for each Payload Type that the client has sent. On the server side, each `record` is routed to a separate 
IPC stream consumer based on its `type` and `schema_id`. This means that a single BAR may feed multiple 
independent Arrow IPC stream readers simultaneously.

Note that Arrow IPC Streams are inherently stateful and must track the current schema and dictionary state for 
each stream.

#### 3.3.1 IPC Messages

Arrow IPC, including the Encapsulated Message types and ordering, is well defined in the Arrow IPC 
specification, but here is a brief description of the lifecycle.

Implementation specifics are discussed in later sections // NEEDS TRIAGE.

---

## 5. Payload Specifications

This section defines the complete Arrow schema for all OTAP payload types, organized by signal category.

**Table Column Descriptions:**
- **Name**: Field name in the Arrow schema
- **Type**: Base Arrow type for this field
- **Alt Representations**: Alternative encodings allowed for this field (e.g., `Dict(u8)` for Dictionary(UInt8, Type), `List(T)` for list types). See also: [Dictionary Encoding](https://arrow.apache.org/docs/format/Columnar.html#dictionary-encoded-layout)
- **Nullable**: Whether the field can contain null values
- **Required**: Whether this field must be present in every record
- **Id Encoding**: The encoding method used for id columns (see [Section 6.5](#65-transport-optimized-encodings))
- **Metadata**: Arrow field-level metadata keys that MAY be present (see [Section 6.5.4](#654-field-metadata))
- **Description**: Human-readable description of the field's purpose

NOTE: Column names that contain a `.` character indicate the presence of a struct typed field. For
example `resource.id` indicates a struct array column called `resource` with a field called `id`.
The type information in the table is for the `id` field.

### 5.1 Common Payloads

#### RESOURCE_ATTRS / SCOPE_ATTRS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| parent_id | UInt16 | — | No | Yes | [QUASI-DELTA](#653-quasi-delta-encoding) | encoding | Foreign key to parent table's `resource.id` or `scope.id` |
| key | Utf8 | Dict(u8), Dict(u16) | No | Yes | — | — | Attribute key name |
| type | UInt8 | — | No | Yes | — | — | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Dict(u8), Dict(u16) | Yes | No | — | — | String value (when type=1) |
| int | Int64 | — | Yes | No | — | — | Integer value (when type=3) |
| double | Float64 | — | Yes | No | — | — | Double value (when type=4) |
| bool | Boolean | — | Yes | No | — | — | Boolean value (when type=2) |
| bytes | Binary | — | Yes | No | — | — | Bytes value (when type=5) |
| ser | Binary | — | Yes | No | — | — | CBOR-encoded Array or Map (when type=6 or 7) |

### 5.2 Logs Payloads

#### LOGS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| id | UInt16 | — | Yes | Yes | [DELTA](#652-delta-encoding) | encoding | Log record identifier (primary key) |
| resource.id | UInt16 | — | Yes | No | [DELTA](#652-delta-encoding) | encoding | Foreign key to `resource.id` |
| resource.schema_url | Utf8 | — | Yes | No | — | — | Resource schema URL |
| resource.dropped_attributes_count | UInt32 | — | Yes | No | — | — | Number of dropped resource attributes |
| scope.id | UInt16 | — | Yes | No | [DELTA](#652-delta-encoding) | encoding | Foreign key to `scope.id` |
| scope.name | Utf8 | — | Yes | No | — | — | Instrumentation scope name |
| scope.version | Utf8 | — | Yes | No | — | — | Instrumentation scope version |
| scope.dropped_attributes_count | UInt32 | — | Yes | No | — | — | Number of dropped scope attributes |
| schema_url | Utf8 | — | Yes | No | — | — | Log schema URL |
| time_unix_nano | Timestamp(Nanosecond) | — | No | Yes | — | — | Log timestamp in Unix nanoseconds |
| observed_time_unix_nano | Timestamp(Nanosecond) | — | No | Yes | — | — | Observation timestamp in Unix nanoseconds |
| trace_id | FixedSizeBinary(16) | — | Yes | No | — | — | Trace `id` for correlation |
| span_id | FixedSizeBinary(8) | — | Yes | No | — | — | Span `id` for correlation |
| severity_number | Int32 | — | Yes | No | — | — | Numeric severity level |
| severity_text | Utf8 | — | Yes | No | — | — | Textual severity level |
| body_type | UInt8 | — | No | Yes | — | — | Body value type (same encoding as attribute type) |
| body_str | Utf8 | — | No | Yes | — | — | String body (may be empty) |
| body_int | Int64 | — | Yes | No | — | — | Integer body (when body_type=3) |
| body_double | Float64 | — | Yes | No | — | — | Double body (when body_type=4) |
| body_bool | Boolean | — | Yes | No | — | — | Boolean body (when body_type=2) |
| body_bytes | Binary | — | Yes | No | — | — | Bytes body (when body_type=5) |
| body_ser | Binary | — | Yes | No | — | — | CBOR-encoded complex body (when body_type=6 or 7) |
| dropped_attributes_count | UInt32 | — | Yes | No | — | — | Number of dropped log attributes |
| flags | UInt32 | — | Yes | No | — | — | Trace flags |

#### LOG_ATTRS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| parent_id | UInt16 | — | No | Yes | [QUASI-DELTA](#653-quasi-delta-encoding) | encoding | Foreign key to [LOGS](#logs).id |
| key | Utf8 | Dict(u8), Dict(u16) | No | Yes | — | — | Attribute key name |
| type | UInt8 | — | No | Yes | — | — | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Dict(u8), Dict(u16) | Yes | No | — | — | String value (when type=1) |
| int | Int64 | — | Yes | No | — | — | Integer value (when type=3) |
| double | Float64 | — | Yes | No | — | — | Double value (when type=4) |
| bool | Boolean | — | Yes | No | — | — | Boolean value (when type=2) |
| bytes | Binary | — | Yes | No | — | — | Bytes value (when type=5) |
| ser | Binary | — | Yes | No | — | — | CBOR-encoded Array or Map (when type=6 or 7) |

### 5.3 Metrics Payloads

#### UNIVARIATE_METRICS / MULTIVARIATE_METRICS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| id | UInt16 | — | No | Yes | [DELTA](#652-delta-encoding) | encoding | Metric identifier (primary key) |
| resource.id | UInt16 | — | Yes | No | [DELTA](#652-delta-encoding) | encoding | Foreign key to `resource.id` |
| resource.schema_url | Utf8 | — | Yes | No | — | — | Resource schema URL |
| resource.dropped_attributes_count | UInt32 | — | Yes | No | — | — | Number of dropped resource attributes |
| scope.id | UInt16 | — | Yes | No | [DELTA](#652-delta-encoding) | encoding | Foreign key to `scope.id` |
| scope.name | Utf8 | — | Yes | No | — | — | Instrumentation scope name |
| scope.version | Utf8 | — | Yes | No | — | — | Instrumentation scope version |
| scope.dropped_attributes_count | UInt32 | — | Yes | No | — | — | Number of dropped scope attributes |
| schema_url | Utf8 | — | Yes | No | — | — | Metric schema URL |
| metric_type | UInt8 | — | No | Yes | — | — | Metric type enum (Gauge, Sum, Histogram, etc.) |
| name | Utf8 | — | No | Yes | — | — | Metric name |
| description | Utf8 | — | Yes | No | — | — | Metric description |
| unit | Utf8 | — | Yes | No | — | — | Metric unit |
| aggregation_temporality | Int32 | — | Yes | No | — | — | Aggregation temporality enum |
| is_monotonic | Boolean | — | Yes | No | — | — | Whether the metric is monotonic |

#### NUMBER_DATA_POINTS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| id | UInt32 | — | No | Yes | [DELTA](#652-delta-encoding) | encoding | Data point identifier (primary key) |
| parent_id | UInt16 | — | No | Yes | [DELTA](#652-delta-encoding) | encoding | Foreign key to [UNIVARIATE_METRICS / MULTIVARIATE_METRICS](#univariate_metrics--multivariate_metrics).id |
| start_time_unix_nano | Timestamp(Nanosecond) | — | No | Yes | — | — | Start time in Unix nanoseconds |
| time_unix_nano | Timestamp(Nanosecond) | — | No | Yes | — | — | Timestamp in Unix nanoseconds |
| int_value | Int64 | — | No | Yes | — | — | Integer value |
| double_value | Float64 | — | No | Yes | — | — | Double value |
| flags | UInt32 | — | Yes | No | — | — | Data point flags |

#### SUMMARY_DATA_POINTS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| id | UInt32 | — | Yes | Yes | [DELTA](#652-delta-encoding) | encoding | Data point identifier (primary key) |
| parent_id | UInt16 | — | No | Yes | [DELTA](#652-delta-encoding) | encoding | Foreign key to [UNIVARIATE_METRICS / MULTIVARIATE_METRICS](#univariate_metrics--multivariate_metrics).id |
| start_time_unix_nano | Timestamp(Nanosecond) | — | Yes | No | — | — | Start time in Unix nanoseconds |
| time_unix_nano | Timestamp(Nanosecond) | — | Yes | No | — | — | Timestamp in Unix nanoseconds |
| count | UInt64 | — | Yes | No | — | — | Count of observations |
| sum | Float64 | — | Yes | No | — | — | Sum of observations |
| quantile | Float64 | List(Float64) | Yes | No | — | — | Quantile values |
| value | Float64 | List(Float64) | Yes | No | — | — | Quantile observation values |
| flags | UInt32 | — | Yes | No | — | — | Data point flags |

#### HISTOGRAM_DATA_POINTS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| id | UInt32 | — | Yes | Yes | [DELTA](#652-delta-encoding) | encoding | Data point identifier (primary key) |
| parent_id | UInt16 | — | No | Yes | [DELTA](#652-delta-encoding) | encoding | Foreign key to [UNIVARIATE_METRICS / MULTIVARIATE_METRICS](#univariate_metrics--multivariate_metrics).id |
| start_time_unix_nano | Timestamp(Nanosecond) | — | Yes | No | — | — | Start time in Unix nanoseconds |
| time_unix_nano | Timestamp(Nanosecond) | — | Yes | No | — | — | Timestamp in Unix nanoseconds |
| count | UInt64 | — | Yes | No | — | — | Count of observations |
| sum | Float64 | — | Yes | No | — | — | Sum of observations |
| bucket_counts | UInt64 | List(UInt64) | Yes | No | — | — | Count per bucket |
| explicit_bounds | Float64 | List(Float64) | Yes | No | — | — | Histogram bucket boundaries |
| flags | UInt32 | — | Yes | No | — | — | Data point flags |
| min | Float64 | — | Yes | No | — | — | Minimum value |
| max | Float64 | — | Yes | No | — | — | Maximum value |

#### EXP_HISTOGRAM_DATA_POINTS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| id | UInt32 | — | Yes | Yes | [DELTA](#652-delta-encoding) | encoding | Data point identifier (primary key) |
| parent_id | UInt16 | — | No | Yes | [DELTA](#652-delta-encoding) | encoding | Foreign key to [UNIVARIATE_METRICS / MULTIVARIATE_METRICS](#univariate_metrics--multivariate_metrics).id |
| start_time_unix_nano | Timestamp(Nanosecond) | — | Yes | No | — | — | Start time in Unix nanoseconds |
| time_unix_nano | Timestamp(Nanosecond) | — | Yes | No | — | — | Timestamp in Unix nanoseconds |
| count | UInt64 | — | Yes | No | — | — | Count of observations |
| sum | Float64 | — | Yes | No | — | — | Sum of observations |
| scale | Int32 | — | Yes | No | — | — | Exponential histogram scale |
| zero_count | UInt64 | — | Yes | No | — | — | Count of zero values |
| positive_offset | Int32 | — | Yes | No | — | — | Positive bucket offset |
| positive_bucket_counts | UInt64 | List(UInt64) | Yes | No | — | — | Positive bucket counts |
| negative_offset | Int32 | — | Yes | No | — | — | Negative bucket offset |
| negative_bucket_counts | UInt64 | List(UInt64) | Yes | No | — | — | Negative bucket counts |
| flags | UInt32 | — | Yes | No | — | — | Data point flags |
| min | Float64 | — | Yes | No | — | — | Minimum value |
| max | Float64 | — | Yes | No | — | — | Maximum value |

#### *_DP_ATTRS

Applies to: NUMBER_DP_ATTRS, SUMMARY_DP_ATTRS, HISTOGRAM_DP_ATTRS, EXP_HISTOGRAM_DP_ATTRS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| parent_id | UInt32 | — | No | Yes | [QUASI-DELTA](#653-quasi-delta-encoding) | encoding | Foreign key to the corresponding \*_DATA_POINTS.id |
| key | Utf8 | Dict(u8), Dict(u16) | No | Yes | — | — | Attribute key name |
| type | UInt8 | — | No | Yes | — | — | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Dict(u8), Dict(u16) | Yes | No | — | — | String value (when type=1) |
| int | Int64 | — | Yes | No | — | — | Integer value (when type=3) |
| double | Float64 | — | Yes | No | — | — | Double value (when type=4) |
| bool | Boolean | — | Yes | No | — | — | Boolean value (when type=2) |
| bytes | Binary | — | Yes | No | — | — | Bytes value (when type=5) |
| ser | Binary | — | Yes | No | — | — | CBOR-encoded Array or Map (when type=6 or 7) |

#### *_DP_EXEMPLARS

Applies to: NUMBER_DP_EXEMPLARS, HISTOGRAM_DP_EXEMPLARS, EXP_HISTOGRAM_DP_EXEMPLARS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| id | UInt32 | — | Yes | Yes | [DELTA](#652-delta-encoding) | encoding | Exemplar identifier (primary key) |
| parent_id | UInt32 | — | No | Yes | [COLUMNAR QUASI-DELTA](#653-quasi-delta-encoding) (int_value, double_value) | encoding | Foreign key to the corresponding \*_DATA_POINTS.id |
| time_unix_nano | Timestamp(Nanosecond) | — | Yes | No | — | — | Timestamp in Unix nanoseconds |
| int_value | Int64 | — | Yes | No | — | — | Integer exemplar value |
| double_value | Float64 | — | Yes | No | — | — | Double exemplar value |
| span_id | FixedSizeBinary(8) | — | Yes | No | — | — | Associated span `id` |
| trace_id | FixedSizeBinary(16) | — | Yes | No | — | — | Associated trace `id` |

#### *_DP_EXEMPLAR_ATTRS

Applies to: NUMBER_DP_EXEMPLAR_ATTRS, HISTOGRAM_DP_EXEMPLAR_ATTRS, EXP_HISTOGRAM_DP_EXEMPLAR_ATTRS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| parent_id | UInt32 | — | No | Yes | [QUASI-DELTA](#653-quasi-delta-encoding) | encoding | Foreign key to the corresponding \*_DP_EXEMPLARS.id |
| key | Utf8 | Dict(u8), Dict(u16) | No | Yes | — | — | Attribute key name |
| type | UInt8 | — | No | Yes | — | — | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Dict(u8), Dict(u16) | Yes | No | — | — | String value (when type=1) |
| int | Int64 | — | Yes | No | — | — | Integer value (when type=3) |
| double | Float64 | — | Yes | No | — | — | Double value (when type=4) |
| bool | Boolean | — | Yes | No | — | — | Boolean value (when type=2) |
| bytes | Binary | — | Yes | No | — | — | Bytes value (when type=5) |
| ser | Binary | — | Yes | No | — | — | CBOR-encoded Array or Map (when type=6 or 7) |

#### METRIC_ATTRS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| parent_id | UInt16 | — | No | Yes | [QUASI-DELTA](#653-quasi-delta-encoding) | encoding | Foreign key to [UNIVARIATE_METRICS / MULTIVARIATE_METRICS](#univariate_metrics--multivariate_metrics).id |
| key | Utf8 | Dict(u8), Dict(u16) | No | Yes | — | Attribute key name |
| type | UInt8 | — | No | Yes | — | — | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Dict(u8), Dict(u16) | Yes | No | — | String value (when type=1) |
| int | Int64 | — | Yes | No | — | — | Integer value (when type=3) |
| double | Float64 | — | Yes | No | — | — | Double value (when type=4) |
| bool | Boolean | — | Yes | No | — | — | Boolean value (when type=2) |
| bytes | Binary | — | Yes | No | — | — | Bytes value (when type=5) |
| ser | Binary | — | Yes | No | — | — | CBOR-encoded Array or Map (when type=6 or 7) |

### 5.4 Traces Payloads

#### SPANS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| id | UInt16 | — | Yes | Yes | [DELTA](#652-delta-encoding) | encoding | Span identifier (primary key) |
| resource.id | UInt16 | — | Yes | No | [DELTA](#652-delta-encoding) | encoding | Foreign key to resource |
| resource.schema_url | Utf8 | — | Yes | No | — | — | Resource schema URL |
| resource.dropped_attributes_count | UInt32 | — | Yes | No | — | — | Number of dropped resource attributes |
| scope.id | UInt16 | — | Yes | No | [DELTA](#652-delta-encoding) | encoding | Foreign key to scope |
| scope.name | Utf8 | — | Yes | No | — | — | Instrumentation scope name |
| scope.version | Utf8 | — | Yes | No | — | — | Instrumentation scope version |
| scope.dropped_attributes_count | UInt32 | — | Yes | No | — | — | Number of dropped scope attributes |
| schema_url | Utf8 | — | Yes | No | — | — | Span schema URL |
| start_time_unix_nano | Timestamp(Nanosecond) | — | No | Yes | — | — | Span start time in Unix nanoseconds |
| duration_time_unix_nano | Duration(Nanosecond) | — | No | Yes | — | — | Span duration in nanoseconds |
| trace_id | FixedSizeBinary(16) | — | No | Yes | — | — | Trace `id` |
| span_id | FixedSizeBinary(8) | — | No | Yes | — | — | Span `id` |
| trace_state | Utf8 | — | Yes | No | — | — | W3C trace state |
| parent_span_id | FixedSizeBinary(8) | — | Yes | No | — | — | Parent span `id` |
| name | Utf8 | — | No | Yes | — | — | Span name |
| kind | Int32 | — | Yes | No | — | — | Span kind enum |
| dropped_attributes_count | UInt32 | — | Yes | No | — | — | Number of dropped span attributes |
| dropped_events_count | UInt32 | — | Yes | No | — | — | Number of dropped events |
| dropped_links_count | UInt32 | — | Yes | No | — | — | Number of dropped links |
| status_code | Int32 | — | Yes | No | — | — | Span status code |
| status_status_message | Utf8 | — | Yes | No | — | — | Status message |

#### SPAN_ATTRS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| parent_id | UInt16 | — | No | Yes | [QUASI-DELTA](#653-quasi-delta-encoding) | encoding | Foreign key to [SPANS](#spans).id |
| key | Utf8 | Dict(u8), Dict(u16) | No | Yes | — | Attribute key name |
| type | UInt8 | — | No | Yes | — | — | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Dict(u8), Dict(u16) | Yes | No | — | String value (when type=1) |
| int | Int64 | — | Yes | No | — | — | Integer value (when type=3) |
| double | Float64 | — | Yes | No | — | — | Double value (when type=4) |
| bool | Boolean | — | Yes | No | — | — | Boolean value (when type=2) |
| bytes | Binary | — | Yes | No | — | — | Bytes value (when type=5) |
| ser | Binary | — | Yes | No | — | — | CBOR-encoded Array or Map (when type=6 or 7) |

#### SPAN_EVENTS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| id | UInt32 | — | Yes | Yes | [DELTA](#652-delta-encoding) | encoding | Event identifier (primary key) |
| parent_id | UInt16 | — | No | Yes | [COLUMNAR QUASI-DELTA](#653-quasi-delta-encoding) (name) | encoding | Foreign key to [SPANS](#spans).id |
| time_unix_nano | Timestamp(Nanosecond) | — | Yes | No | — | — | Event timestamp in Unix nanoseconds |
| name | Utf8 | — | No | Yes | — | — | Event name |
| dropped_attributes_count | UInt32 | — | Yes | No | — | — | Number of dropped event attributes |

#### SPAN_EVENT_ATTRS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| parent_id | UInt32 | — | No | Yes | [QUASI-DELTA](#653-quasi-delta-encoding) | encoding | Foreign key to [SPAN_EVENTS](#span_events).id |
| key | Utf8 | Dict(u8), Dict(u16) | No | Yes | — | Attribute key name |
| type | UInt8 | — | No | Yes | — | — | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Dict(u8), Dict(u16) | Yes | No | — | String value (when type=1) |
| int | Int64 | — | Yes | No | — | — | Integer value (when type=3) |
| double | Float64 | — | Yes | No | — | — | Double value (when type=4) |
| bool | Boolean | — | Yes | No | — | — | Boolean value (when type=2) |
| bytes | Binary | — | Yes | No | — | — | Bytes value (when type=5) |
| ser | Binary | — | Yes | No | — | — | CBOR-encoded Array or Map (when type=6 or 7) |

#### SPAN_LINKS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| id | UInt32 | — | Yes | Yes | [DELTA](#652-delta-encoding) | encoding | Link identifier (primary key) |
| parent_id | UInt16 | — | No | Yes | [COLUMNAR QUASI-DELTA](#653-quasi-delta-encoding) (trace_id) | encoding | Foreign key to [SPANS](#spans).id |
| trace_id | FixedSizeBinary(16) | — | Yes | No | — | — | Linked trace `id` |
| span_id | FixedSizeBinary(8) | — | Yes | No | — | — | Linked span `id` |
| trace_state | Utf8 | — | Yes | No | — | — | Linked trace state |
| dropped_attributes_count | UInt32 | — | Yes | No | — | — | Number of dropped link attributes |

#### SPAN_LINK_ATTRS

| Name | Type | Alt Representations | Nullable | Required | Id Encoding | Metadata | Description |
|------|------|---------------------|----------|----------|-------------|----------|-------------|
| parent_id | UInt32 | — | No | Yes | [QUASI-DELTA](#653-quasi-delta-encoding) | encoding | Foreign key to [SPAN_LINKS](#span_links).id |
| key | Utf8 | Dict(u8), Dict(u16) | No | Yes | — | Attribute key name |
| type | UInt8 | — | No | Yes | — | — | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Dict(u8), Dict(u16) | Yes | No | — | String value (when type=1) |
| int | Int64 | — | Yes | No | — | — | Integer value (when type=3) |
| double | Float64 | — | Yes | No | — | — | Double value (when type=4) |
| bool | Boolean | — | Yes | No | — | — | Boolean value (when type=2) |
| bytes | Binary | — | Yes | No | — | — | Bytes value (when type=5) |
| ser | Binary | — | Yes | No | — | — | CBOR-encoded Array or Map (when type=6 or 7) |

#### 5.5.1 Allowed Dictionary Key Types

Dictionary keys MUST use one of these unsigned integer types:
- **UInt8**: For dictionaries with ≤256 unique values
- **UInt16**: For dictionaries with ≤65,536 unique values

### 5.6 Special Field Rules

#### Attribute Value Fields

For attribute tables, exactly ONE of the value fields (`str`, `int`, `double`, `bool`, `bytes`, `ser`) MUST be non-null, corresponding to the `type` field:

| type value | Active field |
|------------|--------------|
| 1 | str |
| 2 | bool |
| 3 | int |
| 4 | double |
| 5 | bytes |
| 6 | ser (Array encoded as CBOR) |
| 7 | ser (Map encoded as CBOR) |

#### Body Fields (Logs)

For logs, the `body_type` field determines which `body_*` field is populated, similar to attribute fields.

#### Exemplar Value Fields

For exemplar tables, either `int_value` or `double_value` MUST be non-null (or both may be present with appropriate semantics).

### 5.7 Field Metadata

Fields MAY include metadata key-value pairs:

**Standard metadata keys**:

| Key | Values | Meaning |
|-----|--------|---------|
| `encoding` | `"plain"`, `"delta"`, `"quasidelta"` | Transport encoding applied |

**Custom metadata**:
- Implementations MAY define custom metadata keys
- Unknown metadata keys SHOULD be ignored by consumers




---

## 6. Id Columns

This section defines more details related to identifier columns used to establish relationships 
between payload types in the OTAP data model.

### 6.1 Primary Keys and Foreign Keys

All parent-child relationships in the OTAP data model follow a uniform convention:

- **Parent tables** define an `id` column as their primary key
- **Child tables** define a `parent_id` column as a foreign key that always references their parent table's `id` column

**Example**: In the Logs signal:
- The LOGS table has an `id` column (UInt16)
- The LOG_ATTRS table has a `parent_id` column (UInt16) that references LOGS.`id`
- Each LOG_ATTRS row belongs to exactly one LOGS row via this foreign key

Note: For documented table relationships see Section 2.1.
Note: Resource and Scope entities deviate slightly from these conventions, see section 6.3 

### 6.2 Id Column Types

Id columns use unsigned integer types sized according to expected cardinality. `id` columns are 
either u32 or u16 and they define the primary keys of the parent table. As such they are always 
unique within a Record Batch and do not benefit from dictionary encoding, so they must be plainly encoded.

On the other hand, child `parent_id` columns referencing u32 `id` columns of their parents may use
dictionary encoding with either `u8` or `u16` keys to save space.

### 6.3 Resource and Scope Identifiers

Resource and scope entities are **not** represented as separate payload types. Instead, they are embedded as struct fields within root tables (LOGS, SPANS, METRICS).

Each root table contains:
- `resource.id` (UInt16): Identifier for the resource
- `scope.id` (UInt16): Identifier for the instrumentation scope

These fields are commonly referenced as `resource.id` and `scope.id` in the context of struct field access.

Note that there are no RESOURCE or SCOPE payload types. Resources and scopes are defined implicitly by their 
presence in root table rows and can be shared among items of the same type. This gives them some special
characteristics:

1. There is a many-to-many relationship relationship between RESOURCE_ATTRS/SCOPE_ATTRS tables and their
parent payload types
2. The corresponding column in the LOGS/METRICS/SPANS tables for RESOURCE_ATTRS.parent_id and SCOPE_ATTRS.parent_id
are `resource.id` and `scope.id` respectively rather than just `id`.
3. Unlike other identifiers, `resource.id` and `scope.id` have no single table that "owns" them and defines the valid
set of Ids.

### 6.5 Transport Optimized Encodings

OTAP defines specialized column encodings that transform `id` and `parent_id` columns before serialization to 
maximize compression efficiency during network transport.

Id columns often exhibit strong sequential patterns:
- Primary IDs are often sequential (0, 1, 2, 3...)
- Foreign keys (`parent_id`) are often clustered (many attributes reference the same parent item)
- When sorted by `parent_id`, related records appear together

By encoding these patterns explicitly (e.g., storing deltas between values rather than absolute values),
we create long runs of small integers and repeated values that compress extremely well.

Id columns, including `id`, `parent_id`, `resource.id`, and `scope.id`, are by default encoded using
one of the delta encoding techniques listed below unless their field metadata has `"encoding": "plain"`
explicitly set.

Which fields use which encodings are listed in section 6.5.6

#### 6.5.1 PLAIN Encoding

**Encoding identifier**: `"plain"`

No transformation applied. Values are stored as-is in the Arrow array.

**Applicability**: All id columns

#### 6.5.2 DELTA Encoding

**Encoding identifier**: `"delta"`

Stores the difference between consecutive values instead of absolute values. Used for columns that are 
sorted and contain sequential or near-sequential values.

**Applicability**: Primary `id` columns

#### 6.5.3 QUASI-DELTA Encoding

**Encoding identifier**: `"quasidelta"`

A hybrid encoding that applies delta encoding selectively. Parent IDs are delta-encoded only within 
"runs" of rows that share the same attribute key/value or other identifying columns.

**Algorithm for Columnar Quasi-Delta**:
Similar, but matching based on specified column values (e.g., span event `name` field, exemplar `int_value`/`double_value`).

**Applicability**: `parent_id` columns in attribute and related tables

**Typical use**:
- Attribute table `parent_id` columns
- Span event/link `parent_id` columns
- Exemplar `parent_id` columns

#### 6.5.4 Field Metadata

Producers SHOULD include field metadata to indicate encoding: // NEEDS_TRIAGE

```json
{
  "encoding": "delta" | "plain" | "quasidelta"
}
```

**Requirements**:
- If metadata is present, `encoding` field SHOULD indicate the applied encoding
- If metadata is absent, consumers SHOULD assume the column is encoded according to the tables
in section 5.
- Consumers MUST handle both presence and absence of metadata

#### 6.5.5 Schema Metadata

Producers MAY include schema-level metadata: // NEEDS_TRIAGE: We have sort columns defined in the code, but no references?

```json
{
  "sort_columns": "field1,field2,..."
}
```

This indicates the columns by which the record batch has been sorted, which is useful context for understanding applied encodings.

#### 6.5.6 Encoding Application by Payload Type

The following table specifies encodings per payload type:

| Payload Type | Column | Encoding | Data Type |
|--------------|--------|----------|-----------|
| LOGS | id | DELTA (remapped) | UInt16 |
| LOGS | resource.id | DELTA (remapped) | UInt16 |
| LOGS | scope.id | DELTA (remapped) | UInt16 |
| UNIVARIATE_METRICS | id | DELTA (remapped) | UInt16 |
| UNIVARIATE_METRICS | resource.id | DELTA (remapped) | UInt16 |
| UNIVARIATE_METRICS | scope.id | DELTA (remapped) | UInt16 |
| SPANS | id | DELTA (remapped) | UInt16 |
| SPANS | resource.id | DELTA (remapped) | UInt16 |
| SPANS | scope.id | DELTA (remapped) | UInt16 |
| RESOURCE_ATTRS | parent_id | QUASI-DELTA | UInt16 |
| SCOPE_ATTRS | parent_id | QUASI-DELTA | UInt16 |
| LOG_ATTRS | parent_id | QUASI-DELTA | UInt16 |
| SPAN_ATTRS | parent_id | QUASI-DELTA | UInt16 |
| METRIC_ATTRS | parent_id | QUASI-DELTA | UInt16 |
| NUMBER_DATA_POINTS | id | DELTA (remapped) | UInt32 |
| NUMBER_DATA_POINTS | parent_id | DELTA | UInt16 |
| SUMMARY_DATA_POINTS | id | DELTA (remapped) | UInt32 |
| SUMMARY_DATA_POINTS | parent_id | DELTA | UInt16 |
| HISTOGRAM_DATA_POINTS | id | DELTA (remapped) | UInt32 |
| HISTOGRAM_DATA_POINTS | parent_id | DELTA | UInt16 |
| EXP_HISTOGRAM_DATA_POINTS | id | DELTA (remapped) | UInt32 |
| EXP_HISTOGRAM_DATA_POINTS | parent_id | DELTA | UInt16 |
| SPAN_EVENTS | id | DELTA (remapped) | UInt32 |
| SPAN_EVENTS | parent_id | COLUMNAR QUASI-DELTA (name) | UInt16 |
| SPAN_LINKS | id | DELTA (remapped) | UInt32 |
| SPAN_LINKS | parent_id | COLUMNAR QUASI-DELTA (trace_id) | UInt16 |
| {TYPE}_DP_ATTRS | parent_id | QUASI-DELTA | UInt32 |
| NUMBER_DP_EXEMPLARS | id | DELTA (remapped) | UInt32 |
| NUMBER_DP_EXEMPLARS | parent_id | COLUMNAR QUASI-DELTA (int_value, double_value) | UInt32 |
| HISTOGRAM_DP_EXEMPLARS | id | DELTA (remapped) | UInt32 |
| HISTOGRAM_DP_EXEMPLARS | parent_id | COLUMNAR QUASI-DELTA (int_value, double_value) | UInt32 |
| EXP_HISTOGRAM_DP_EXEMPLARS | id | DELTA (remapped) | UInt32 |
| EXP_HISTOGRAM_DP_EXEMPLARS | parent_id | COLUMNAR QUASI-DELTA (int_value, double_value) | UInt32 |

**Note**: "DELTA (remapped)" means the producer creates new sequential IDs and remaps parent references. This is necessary because the original IDs may not be sorted.

---

## 7. Schema Management

One of OTAP's key features is dynamic schema management. Unlike protocols with fixed schemas that 
must be known a priori by all parties, OTAP allows schemas to evolve during a streams lifetime.

Arrow IPC Streams provide negotiate schemas at the time the stream is established. Schemas define

1. The field names and types of a RecordBatch
2. The order in which the fields appear in the RecordBatch

Certain details are flexible like the subset of fields for each payload type that are used;
the order of the fields; and to some degree the type of some fields (such as Dictionary(u8, u32) vs u32),
according to the OTAP spec, but once these are negotiated at the start of an Arrow IPC stream, they
cannot be changed later without stopping and recreating a stream.

### 7.1 Schema Resets

The ability to negotiate a new schema by starting a new IPC Stream over the same gRPC connection,
is a feature of OTAP known as a Schema Reset. This is useful when a client wants to change anything
about the Schema of a payload, such as by upgrading the key size of a dictionary from u8 to u16 after detecting
a dictionary overflow.

Schema Resets are coordinated via a change in the `schema_id` field of ArrowPayload for a Payload Type.
Servers MUST track the `schema_id` for each Payload Type within a Stream. 

If the client changes the `schema_id` for a Payload Type, the client MUST reset any IPC writer state and 
include the appropriate start of stream messages in the `record` field ahead of any more Record Batch messages.
This means starting with a Schema message and any required dictionaries.

The server MUST detect the change and reset any IPC reader state and assume that the `record` in that message
contains the required messages to start a new Stream.

### 7.1 Schema Identification

Each Arrow schema for a given payload type is identified by a unique `schema_id` string. This identifier serves 
as a contract between producer and consumer: "the data in this payload conforms to the schema identified by this ID."

**Requirements**:
- Schema IDs MUST be unique within a payload type for a given stream

### 7.2 Schema ID Generation 

**Recommended algorithm**: // NEEDS_TRIAGE: Should this be some kind of appendix or implementation detail thing?

1. Sort fields by name at each nesting level
2. Generate compact representation:
   - Field name
   - `:` separator
   - Type abbreviation (e.g., `U16` for UInt16, `Str` for Utf8, `Dic<U16,Str>` for Dictionary)
3. Concatenate fields with `,` separator

**Example**: `id:U16,parent_id:U16,key:Str,type:U8,str:Dic<U16,Str>`

**Note**: Metadata-only changes (e.g., updating `encoding` metadata) do NOT require schema reset.

### 7.5 Schema Compatibility

OTAP does NOT require forward or backward schema compatibility. Consumers need only handle the specific schema
identified by schema_id. All schemas MUST conform to the specification in section 5.

---

## 8. Error Handling

Robust error handling is critical for reliable telemetry collection. OTAP uses gRPC status codes to signal different 
error conditions, allowing clients to distinguish between transient failures (that should be retried) and permanent 
failures (that indicate bugs or misconfigurations).

Error handling in OTAP operates at two levels:

1. **BAR-level errors**: Reported via BatchStatus messages with non-OK status codes
2. **Stream-level errors**: Reported by closing the gRPC stream with an error status

Understanding which errors are retryable versus non-retryable is essential for implementing correct client behavior.
Retrying non-retryable errors wastes resources, while failing to retry retryable errors can lead to data loss.

### 8.1 Error Categories

#### 8.1.1 Retryable Errors

Errors that MAY resolve with retry:

- **UNAVAILABLE**: Service temporarily unavailable
- **RESOURCE_EXHAUSTED**: Server temporarily overloaded
- **DEADLINE_EXCEEDED**: Request timeout, may succeed if retried
- **ABORTED**: Operation aborted, typically safe to retry
- **CANCELED**: Operation canceled by client

**Client behavior**: Clients SHOULD implement exponential backoff retry for these errors.

#### 8.1.2 Non-Retryable Errors

Errors indicating client problems or invalid data:

- **INVALID_ARGUMENT**: Malformed data or protocol violation
- **UNAUTHENTICATED**: Missing or invalid authentication
- **PERMISSION_DENIED**: Insufficient permissions
- **INTERNAL**: Internal server error (typically not recoverable by retry)

**Client behavior**: Clients SHOULD NOT retry these errors without corrective action.

### 8.2 Status Codes

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

These match gRPC status codes for consistency.

### 8.3 Error Handling Rules 

#### 8.3.1 Schema Errors

// NEEDS_TRIAGE: We probably need to define behaviors for all of these

**Invalid schema**:
- **Cause**: Schema message is malformed or uses unsupported types
- **Status**: INVALID_ARGUMENT
- **Action**: Client MUST fix schema definition

**Schema mismatch**:
- **Cause**: RecordBatch doesn't match declared schema
- **Status**: INVALID_ARGUMENT
- **Action**: Client MUST ensure consistency between schema and data

#### 8.3.2 Data Errors

**Dictionary key overflow**:
- **Cause**: Dictionary key exceeds maximum for key type
- **Status**: INVALID_ARGUMENT
- **Action**: Client MUST perform schema reset with larger key type

**Unknown field**:
- **Cause**: RecordBatch contains field not in schema
- **Action**: Server SHOULD ignore unknown fields and continue processing

**Unrecognized payload type**:
- **Cause**: ArrowPayloadType is unknown or unsupported
- **Status**: INVALID_ARGUMENT
- **Action**: Client MUST use valid payload type

**Unrecognized attribute type**:
- **Cause**: Attribute `type` field has unknown value
- **Action**: Server SHOULD skip unknown attribute types and continue processing

#### 8.3.3 Resource Errors

**Memory limit exceeded**:
- **Cause**: Server memory allocator limit reached
- **Status**: RESOURCE_EXHAUSTED
- **Action**: Client SHOULD retry with backoff or reduce BAR size

**Empty BAR**:
- **Cause**: BatchArrowRecords contains no payloads
- **Status**: INVALID_ARGUMENT
- **Action**: Client MUST send non-empty BARs

#### 8.3.4 Stream Errors

**Schema reset without schema message**:
- **Cause**: New schema_id used without sending Schema message first
- **Status**: INVALID_ARGUMENT
- **Action**: Client MUST send Schema message when schema_id changes

**RecordBatch before schema**:
- **Cause**: RecordBatch sent before Schema message for new schema_id
- **Status**: INVALID_ARGUMENT
- **Action**: Client MUST send Schema message first

**Dictionary used before definition**:
- **Cause**: RecordBatch references dictionary not yet sent
- **Status**: INVALID_ARGUMENT
- **Action**: Client MUST send DictionaryBatch before referencing in RecordBatch

### 8.4 Partial Failure Handling

If a BatchArrowRecords contains multiple payloads and one fails:

**Option 1: Fail entire BAR**
- Server returns non-OK status for entire BAR
- Client MUST resend entire BAR or skip it

**Option 2: Partial success (if supported)**
- Server returns OK status
- Server MAY include details about partial failure in status_message
- Responsibility on server to handle incomplete data

**Recommendation**: Implementations SHOULD fail entire BAR to ensure data consistency.

---

## 9. Field Specifications

This section provides detailed semantics for fields in OTAP schemas, including which fields are required versus 
optional, special handling rules for attribute and body fields, and field metadata conventions.

Understanding field requirements is important for both producers (to ensure they send valid data) and consumers 
(to know which fields they can rely on being present). OTAP inherits most field semantics from OTLP but adapts them 
to the columnar model.

### 9.1 Required vs Optional Fields
---

## 10. Compliance Requirements

### 10.1 Producer (Client) Requirements

A compliant OTAP producer MUST:

1. **Generate valid BatchArrowRecords messages**:
   - Unique, monotonically increasing batch_id
   - At least one ArrowPayload per BAR
   - Primary table payload listed first

2. **Follow Arrow IPC streaming format**:
   - Schema message first for new schema_id
   - DictionaryBatch before RecordBatch when needed
   - Valid Encapsulated Message format

3. **Use valid payload types and schemas**:
   - ArrowPayloadType matches actual schema
   - Field types match specification
   - Required fields present

4. **Handle schema resets correctly**:
   - Change schema_id when schema changes
   - Send complete Schema message
   - Reinitialize dictionaries

5. **Use proper encoding/decoding**:
   - If using transport optimizations, apply encodings correctly
   - Include metadata to indicate encoding

A compliant producer SHOULD:

1. Apply transport optimized encodings per section 6.5.6
2. Use dictionary encoding for high-cardinality string fields
3. Sort record batches within BARs for optimal compression
4. Implement exponential backoff for retryable errors
5. Handle dictionary overflow via schema reset

### 10.2 Consumer (Server) Requirements

A compliant OTAP consumer MUST:

1. **Accept valid BatchArrowRecords messages**:
   - Process all defined ArrowPayloadType values
   - Handle variable payload ordering (though primary first is expected)

2. **Maintain Arrow IPC reader state**:
   - Separate readers per schema_id
   - Dictionary state tracking
   - Proper cleanup on schema reset

3. **Decode Arrow data correctly**:
   - Parse Schema, DictionaryBatch, and RecordBatch messages
   - Apply dictionary lookups
   - Handle nullable fields

4. **Reverse transport encodings**:
   - Detect and reverse delta encoding
   - Detect and reverse quasi-delta encoding
   - Reconstruct original values

5. **Send BatchStatus acknowledgments**:
   - Acknowledge each received BAR
   - Use appropriate status codes
   - Provide meaningful error messages

6. **Handle errors gracefully**:
   - Ignore unknown fields in RecordBatches
   - Ignore unknown attribute types
   - Return appropriate error codes

A compliant consumer SHOULD:

1. Implement memory limits to prevent DoS
2. Validate data semantics (e.g., foreign key integrity)
3. Log warnings for unexpected but non-fatal conditions
4. Support all specified dictionary key types (UInt8, UInt16, UInt32)

### 10.3 Interoperability

**Cross-implementation compatibility**:
- Compliant producers and consumers from different implementations MUST interoperate
- Schema IDs MAY differ between implementations (determinism not required across implementations)
- Transport optimizations are optional; implementations MUST support both optimized and plain data

**Version compatibility**:
- This specification is version 1.0
- Future versions may add new payload types or fields
- Implementations SHOULD ignore unknown payload types and fields for forward compatibility

---

## Appendix A: Default Encodings Summary

When transport optimization is **enabled**, the following defaults are recommended:

| Payload Type | Default Optimizations |
|--------------|----------------------|
| Primary tables (LOGS, SPANS, METRICS) | Delta-encode `id`, `resource.id`, `scope.id` |
| Attribute tables | Quasi-delta encode `parent_id`, dictionary-encode `key` and `str` |
| Data point tables | Delta-encode `id` and `parent_id` |
| Event/Link tables | Delta-encode `id`, columnar quasi-delta encode `parent_id` |
| Exemplar tables | Delta-encode `id`, columnar quasi-delta encode `parent_id` |

When transport optimization is **disabled**:
- Use **PLAIN** encoding for all fields
- Dictionary encoding MAY still be applied for efficiency

---

## Appendix B: Example Flows

### B.1 First BAR (Schema Initialization)

**Client sends**:
```
BatchArrowRecords {
  batch_id: 0
  arrow_payloads: [
    {
      schema_id: "id:U16,time_unix_nano:Tns,body_str:Str"
      type: LOGS
      record: <Schema message><RecordBatch message>
    },
    {
      schema_id: "parent_id:U16,key:Str,type:U8,str:Dic<U16,Str>,..."
      type: LOG_ATTRS
      record: <Schema message><DictionaryBatch message><RecordBatch message>
    }
  ]
}
```

**Server responds**:
```
BatchStatus {
  batch_id: 0
  status_code: OK
  status_message: ""
}
```

### B.2 Subsequent BAR (Delta Dictionary)

**Client sends**:
```
BatchArrowRecords {
  batch_id: 1
  arrow_payloads: [
    {
      schema_id: "id:U16,time_unix_nano:Tns,body_str:Str"  // same as before
      type: LOGS
      record: <RecordBatch message>  // no Schema needed
    },
    {
      schema_id: "parent_id:U16,key:Str,type:U8,str:Dic<U16,Str>,..."  // same as before
      type: LOG_ATTRS
      record: <DictionaryBatch message (delta)><RecordBatch message>  // delta dictionary
    }
  ]
}
```

### B.3 Schema Reset (Dictionary Overflow)

**Client sends**:
```
BatchArrowRecords {
  batch_id: 10
  arrow_payloads: [
    {
      schema_id: "id:U16,time_unix_nano:Tns,body_str:Str"  // same
      type: LOGS
      record: <RecordBatch message>
    },
    {
      schema_id: "parent_id:U16,key:Str,type:U8,str:Dic<U32,Str>,..."  // NEW: U32 dictionary
      type: LOG_ATTRS
      record: <Schema message><DictionaryBatch message><RecordBatch message>  // full reset
    }
  ]
}
```

---

## Appendix C: Implementation Notes

### C.1 Performance Considerations

1. **BAR sizing**: Larger BARs improve compression but increase memory usage
2. **Dictionary encoding**: Most effective for low-to-medium cardinality (10-10,000 unique values)
3. **Transport encoding**: Most effective when data has strong sequential patterns
4. **Memory pooling**: Reuse Arrow allocators and buffers across BARs

---

## Appendix D: Changes from OTLP

Major differences from OTLP:

1. **Format**: Columnar (Arrow) vs row-based (Protobuf)
3. **Schema evolution**: Dynamic schemas with schema_id vs fixed protobuf schema
4. **Dictionaries**: Stateful dictionary encoding vs no dictionary support
5. **Normalization**: Related tables vs nested messages
6. **Transport optimization**: Built-in encodings vs no optimization

---

## Appendix E: Load Balancing

OTAP's stateful, long-lived gRPC streams introduce load-balancing challenges that do not arise with stateless unary RPCs. 
Because gRPC multiplexes streams over a single HTTP/2 connection, L4 (TCP-level) load balancers distribute work at connection
granularity, not per-stream. Combined with kernel `SO_REUSEPORT` hashing, too few client connections can pin traffic to a 
single backend core.

For a detailed treatment of challenges, solution techniques (client-side and server-side), and recommended baseline configurations,
see [Load Balancing: Challenges & Solutions](rust/otap-dataflow/docs/load-balancing.md).

---

## Appendix F: Glossary

- **Apache Arrow IPC Format**: https://arrow.apache.org/docs/format/Columnar.html#ipc-streaming-format
- **BAR**: Abbreviation for BatchArrowRecords, the client gRPC message
- **Client/Producer**: The sender of telemetry data
- **gRPC**: https://grpc.io/
- **Items**: The item type of a Signal e.g. Log, Data Point(s), or Span
- **OTLP Specification**: OpenTelemetry Protocol specification
- **Payload**: An ArrowPayload containing serialized Arrow IPC messages
- **Payload Type**: Also referred to as ArrowPayloadType, this is equivalent to a distinct table in the OTAP data model
- **Root Payload/Root Payload Type**: The root table in the Signal's DAG
- **Schema Reset**: The act of changing the Arrow schema for a Payload Type
- **Server/Consumer**: The receiver of telemetry data
- **Signal**: One of Logs, Metrics, or Traces

---

## Appendix G: References

1. Apache Arrow IPC Format: https://arrow.apache.org/docs/format/Columnar.html
2. OTLP Specification: https://opentelemetry.io/docs/specs/otlp/
3. gRPC Status Codes: https://grpc.io/docs/guides/status-codes/
4. OTEP 0156: https://github.com/open-telemetry/oteps/blob/main/text/0156-columnar-encoding.md
5. Reference Implementation (Go): [Producer](https://github.com/open-telemetry/otel-arrow/blob/main/pkg/otel/arrow_record/producer.go), [Consumer](https://github.com/open-telemetry/otel-arrow/blob/main/pkg/otel/arrow_record/consumer.go)
6. Rust Implementation: otap-dataflow/crates/pdata
7. RFC 2119: https://www.rfc-editor.org/rfc/rfc2119

