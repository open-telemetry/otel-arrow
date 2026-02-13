# OpenTelemetry Arrow Protocol (OTAP) Formal Specification

**Version:** 0.1
**Status:** Draft

## Table of Contents

1. [Introduction](#1-introduction)
2. [Data Model](#2-data-model)
3. [Protocol Architecture](#3-protocol-architecture)
4. [Transport Layer](#4-transport-layer)
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

OTAP represents Signals as a set of normalized tables connected by foreign key relationships; 
effectively a database. Each Signal type has a different number and set of tables reflecting the data 
transported by that Signal. 

Each table within a Signal has a designated Payload Type that identifies it. For example, the Logs
signal type consists of four Payload Types: LOGS, LOG_ATTRS, RESOURCE_ATTRS, and SCOPE_ATTRS.

The foreigh key relationships between these tables form a Rooted Directed Acyclic Graph (DAG) 
with the Root Payload Type being the root of that graph. 

The LOGS table is the Root Payload Type for Logs and fills a similar role as an OTLP Log. 
Each Log has a unique `id` which identifies it,
and links it to the LOG_ATTRS table which defines the logs attributes. Logs similarly contains
`resource` and `scope` fields, each having an `id`  which links them to the RESOURCE_ATTRS and
SCOPE_ATTRS tables.

The Metrics and Traces signals have a similar structure, but with more tables and are defined 
below along with the relationships between each table.

TODO: It is probably enough information for this section to enumerate the payload types for
each signal and a description of them. I think we can move the relationships and Id semantics
to a later section where we define the full schema and arrow value type for each column.

#### 2.1.1 Logs Signal Tables

| Payload Type | Enum Value | Description | Id | Child Payload Types | Parent Payload Type |
|---|---|---|---|---|---|
| LOGS | 30 | Core log record data (Root) | Yes | LOG_ATTRS, RESOURCE_ATTRS, SCOPE_ATTRS | — |
| LOG_ATTRS | 31 | Log-level attributes | No | — | LOGS |
| RESOURCE_ATTRS | 1 | Resource attributes | No | — | LOGS |
| SCOPE_ATTRS | 2 | Instrumentation scope attributes | No | — | LOGS |

#### 2.1.2 Metrics Signal Tables

| Payload Type | Enum Value | Description | Id | Child Payload Types | Parent Payload Type |
|---|---|---|---|---|---|
| UNIVARIATE_METRICS | 10 | Core metric metadata (Root) | Yes | NUMBER_DATA_POINTS, SUMMARY_DATA_POINTS, HISTOGRAM_DATA_POINTS, EXP_HISTOGRAM_DATA_POINTS, METRIC_ATTRS, RESOURCE_ATTRS, SCOPE_ATTRS | — |
| MULTIVARIATE_METRICS | 25 | Core metric metadata (Root) | Yes | NUMBER_DATA_POINTS, SUMMARY_DATA_POINTS, HISTOGRAM_DATA_POINTS, EXP_HISTOGRAM_DATA_POINTS, METRIC_ATTRS, RESOURCE_ATTRS, SCOPE_ATTRS | — |
| NUMBER_DATA_POINTS | 11 | Gauge and sum data points | Yes | NUMBER_DP_ATTRS, NUMBER_DP_EXEMPLARS | METRICS |
| SUMMARY_DATA_POINTS | 12 | Summary data points | Yes | SUMMARY_DP_ATTRS | METRICS |
| HISTOGRAM_DATA_POINTS | 13 | Histogram data points | Yes | HISTOGRAM_DP_ATTRS, HISTOGRAM_DP_EXEMPLARS | METRICS |
| EXP_HISTOGRAM_DATA_POINTS | 14 | Exponential histogram data points | Yes | EXP_HISTOGRAM_DP_ATTRS, EXP_HISTOGRAM_DP_EXEMPLARS | METRICS |
| NUMBER_DP_ATTRS | 15 | Attributes for number data points | No | — | NUMBER_DATA_POINTS |
| SUMMARY_DP_ATTRS | 16 | Attributes for summary data points | No | — | SUMMARY_DATA_POINTS |
| HISTOGRAM_DP_ATTRS | 17 | Attributes for histogram data points | No | — | HISTOGRAM_DATA_POINTS |
| EXP_HISTOGRAM_DP_ATTRS | 18 | Attributes for exp histogram data points | No | — | EXP_HISTOGRAM_DATA_POINTS |
| NUMBER_DP_EXEMPLARS | 19 | Exemplars for number data points | Yes | NUMBER_DP_EXEMPLAR_ATTRS | NUMBER_DATA_POINTS |
| HISTOGRAM_DP_EXEMPLARS | 20 | Exemplars for histogram data points | Yes | HISTOGRAM_DP_EXEMPLAR_ATTRS | HISTOGRAM_DATA_POINTS |
| EXP_HISTOGRAM_DP_EXEMPLARS | 21 | Exemplars for exp histogram data points | Yes | EXP_HISTOGRAM_DP_EXEMPLAR_ATTRS | EXP_HISTOGRAM_DATA_POINTS |
| NUMBER_DP_EXEMPLAR_ATTRS | 22 | Exemplar attributes for number DPs | No | — | NUMBER_DP_EXEMPLARS |
| HISTOGRAM_DP_EXEMPLAR_ATTRS | 23 | Exemplar attributes for histogram DPs | No | — | HISTOGRAM_DP_EXEMPLARS |
| EXP_HISTOGRAM_DP_EXEMPLAR_ATTRS | 24 | Exemplar attributes for exp histogram DPs | No | — | EXP_HISTOGRAM_DP_EXEMPLARS |
| METRIC_ATTRS | 26 | Metric-level attributes | No | — | METRICS |
| RESOURCE_ATTRS | 1 | Resource attributes | No | — | METRICS |
| SCOPE_ATTRS | 2 | Scope attributes | No | — | METRICS |

#### 2.1.3 Traces Signal Tables

| Payload Type | Enum Value | Description | Id | Child Payload Types | Parent Payload Type |
|---|---|---|---|---|---|
| SPANS | 40 | Core span data (Root) | Yes | SPAN_ATTRS, SPAN_EVENTS, SPAN_LINKS, RESOURCE_ATTRS, SCOPE_ATTRS | — |
| SPAN_ATTRS | 41 | Span attributes | No | — | SPANS |
| SPAN_EVENTS | 42 | Span events | Yes | SPAN_EVENT_ATTRS | SPANS |
| SPAN_EVENT_ATTRS | 44 | Event attributes | No | — | SPAN_EVENTS |
| SPAN_LINKS | 43 | Span links | Yes | SPAN_LINK_ATTRS | SPANS |
| SPAN_LINK_ATTRS | 45 | Link attributes | No | — | SPAN_LINKS |
| RESOURCE_ATTRS | 1 | Resource attributes | No | — | SPANS |
| SCOPE_ATTRS | 2 | Scope attributes | No | — | SPANS |

The foreign key relationships between payload types are defined in Section 6.

---

## 3. Protocol Architecture

OTAP consists of three distinct layers:

1. **gRPC Layer**: Bi-directional streaming RPC services for each signal type
2. **OTAP Message Layer**: BatchArrowRecords and ArrowPayload protobuf messages
3. **Arrow IPC Layer**: Apache Arrow Interprocess Communication streams

### 3.1 gRPC Layer

The gRPC layer provides the transport mechanism and service definitions. It establishes the bi-directional streaming connections between clients and servers over HTTP/2. There is a single client message type, BatchArrowRecords, and a single server response message type BatchStatus.

Despite a single message type, OTAP defines three separate gRPC services (one per signal type) rather than a unified
service. The OTAP Message Layer places further restrictions on the contents of BatchArrowRecords per service.

#### 3.1.1 Service Definitions

OTAP defines three signal-specific gRPC services in the [protobuf definition](https://github.com/open-telemetry/otel-arrow/blob/main/proto/opentelemetry/proto/experimental/arrow/v1/arrow_service.proto):

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

Each service accepts a stream of BatchArrowRecords (BAR) messages from the client and returns a stream of BatchStatus acknowledgments. The bi-directional streaming pattern allows the client to continue sending BARs while waiting for acknowledgments, enabling high throughput with backpressure control.

#### 3.1.2 Connection Lifecycle

1. Client establishes gRPC connection to server
2. Client initiates bi-directional stream for appropriate signal type
3. Client sends stream of BatchArrowRecords (BAR) messages
4. Server processes each BAR and returns BatchStatus acknowledgments
5. Connection persists until explicitly closed or error occurs

### 3.2 OTAP Message Layer

The OTAP message layer places additional restrictions and requirements on the contents of a BAR, the
ArrowPayload, and BatchStatus messages. It defines which Payload Types are valid for which Services/Signals;
rules around Schema Evolution, Schema Resets, and Error Handling; and when it is allowable to omit payloads
entirely. These mechanics are further explained in Section 4.

### 3.3 Arrow IPC Layer

The Arrow IPC (Interprocess Communication) layer is the innermost layer where the telemetry data resides in Apache Arrow's columnar format. Arrow IPC defines how schemas and data are serialized into byte streams using a standardized format that can be read by any Arrow-compatible library. This layer enables:

- **Schema negotiation**: Dynamic schema definition without pre-compiled protobuf definitions
- **Dictionary/Delta Dictionary encoding**: Efficient representation of repeated string values
- **Zero-copy deserialization**: Data can be read directly from wire format without copying
- **Columnar layout**: Data organized by column rather than by row, enabling better compression and SIMD processing

**Stream Organization**: Each ArrowPayload within a BAR contains a buffer of bytes (the `record` field) that represents a slice of an Arrow IPC stream. These buffers may contain multiple Encapsulated Arrow IPC Messages—for example, a Schema message followed by DictionaryBatch messages and RecordBatch messages—all serialized consecutively in the Arrow IPC Streaming Format.

On the server side, each ArrowPayload is routed to a separate stream consumer based on its `type` and `schema_id`. This means that a single BAR may feed multiple independent Arrow IPC stream readers simultaneously:

- The LOGS payload goes to the logs stream consumer
- The LOG_ATTRS payload goes to the log attributes stream consumer
- The RESOURCE_ATTRS payload goes to the resource attributes stream consumer
- And so on for each table type

Each consumer maintains its own stateful Arrow IPC reader, tracking the current schema and dictionary state for its specific stream. This parallel consumption model allows efficient processing of the normalized table structure, where different tables can be decoded and processed independently while maintaining referential integrity through the foreign key relationships (via `id` and `parent_id` fields).

---

## 4. Transport Layer

The transport layer defines how OTAP packages telemetry data into messages suitable for transmission over gRPC. This layer bridges the Arrow IPC format with the gRPC streaming protocol.

### 4.1 BatchArrowRecords Message

The BatchArrowRecords (BAR) message is the fundamental unit of data transmission in OTAP. It represents a complete set of related telemetry tables for a single signal type, containing all the tables needed to reconstruct that signal (e.g., logs plus their attributes, or spans plus their events, links, and attributes).

Each BAR is assigned a unique identifier that allows the server to acknowledge receipt and report errors on a per-BAR basis. This enables reliable transmission with flow control—clients can send multiple BARs in flight while tracking which have been acknowledged.

```protobuf
message BatchArrowRecords {
  int64 batch_id = 1;                      // [REQUIRED]
  repeated ArrowPayload arrow_payloads = 2; // [REQUIRED]
  bytes headers = 3;                        // [OPTIONAL]
}
```

**Field Descriptions:**

- **batch_id**: A unique identifier for this BAR within the current gRPC stream. This ID is used by the server to send acknowledgments (BatchStatus messages) and by the client to correlate those acknowledgments with sent BARs. The ID space is scoped to a single gRPC stream connection.

- **arrow_payloads**: A collection of ArrowPayload messages, each containing the serialized Arrow IPC data for one table. For example, a logs BAR might contain four payloads: one for the LOGS table, and three for LOG_ATTRS, RESOURCE_ATTRS, and SCOPE_ATTRS tables. The primary signal table (LOGS, SPANS, or METRICS) SHOULD be listed first to simplify consumer processing.

- **headers**: An optional field for transmitting additional metadata alongside the BAR. When present, headers are encoded using HPACK compression. This field is typically used for authentication tokens, tracing context, or other out-of-band metadata. Servers MAY ignore this field if they do not require such metadata.

**Requirements:**

- **batch_id**:
  - MUST be unique within the gRPC stream
  - SHOULD be monotonically increasing for easier debugging and ordering
  - Used by server to acknowledge receipt via BatchStatus

- **arrow_payloads**:
  - MUST contain at least one payload
  - First payload MUST be the primary table (LOGS, SPANS, or UNIVARIATE_METRICS/MULTIVARIATE_METRICS)
  - Empty tables MAY be omitted from the BAR
  - Payloads SHOULD be ordered: primary table first, followed by related tables

- **headers**:
  - OPTIONAL field for additional metadata
  - If present, MUST be encoded using HPACK
  - Servers MAY ignore this field

### 4.2 ArrowPayload Message

An ArrowPayload encapsulates the serialized Arrow IPC data for a single table within a BAR. Each payload is tagged with a schema identifier and a type indicator so that consumers can correctly interpret and route the data.

The schema identifier is critical for OTAP's stateful protocol design. When a consumer sees a new schema_id for a given table type, it knows to reset its Arrow IPC reader and expect a new schema definition. This mechanism enables dynamic schema evolution—such as upgrading dictionary key sizes when cardinality grows—without breaking the connection.

```protobuf
message ArrowPayload {
  string schema_id = 1;          // [REQUIRED]
  ArrowPayloadType type = 2;     // [REQUIRED]
  bytes record = 3;              // [REQUIRED]
}
```

**Field Descriptions:**

- **schema_id**: A unique identifier for the Arrow schema used in this payload. The schema ID is derived from the schema structure (field names, types, and their ordering). When the client needs to change the schema—for example, to use a larger dictionary key type—it generates a new schema_id and includes a Schema message in the record bytes.

- **type**: An enum value identifying which table this payload represents (e.g., LOGS, SPAN_ATTRS, NUMBER_DATA_POINTS). This allows the consumer to route the data to the appropriate processing logic and validate that the schema matches expectations for that table type.

- **record**: The raw bytes containing one or more Apache Arrow Encapsulated IPC Messages. These messages follow the Arrow IPC Streaming Format and include Schema messages (for new schemas), DictionaryBatch messages (for dictionary state), and RecordBatch messages (for the actual data rows).

**Requirements:**

- **schema_id**:
  - MUST be a unique string identifier for the Arrow schema
  - Schema ID changes indicate a schema reset
  - Format is implementation-defined but SHOULD be deterministic based on schema structure
  - Recommended format: Compact representation of field names and types (see section 7.2)

- **type**:
  - MUST be a valid ArrowPayloadType enum value
  - MUST NOT be UNKNOWN (value 0)
  - Determines which table this payload represents

- **record**:
  - MUST contain one or more serialized Apache Arrow Encapsulated IPC Messages
  - Message format defined by Arrow IPC Streaming specification
  - See section 4.3 for message ordering requirements

### 4.3 Arrow IPC Message Ordering

The Apache Arrow IPC Streaming Format defines a stateful protocol where schemas and dictionaries must be established before data can be transmitted. Each ArrowPayload's `record` field contains a sequence of Encapsulated Arrow IPC Messages that must follow specific ordering rules to ensure the consumer can correctly interpret the data.

Understanding this ordering is critical: the first time a schema_id appears, the consumer needs to learn what the schema looks like and initialize any dictionaries. On subsequent uses of the same schema_id, the consumer only needs the data records themselves (and possibly dictionary updates). The ordering rules reflect these different scenarios.

#### 4.3.1 Initial Schema Transmission

When a new schema_id is introduced, the first ArrowPayload with that schema_id MUST contain messages in this order:

1. **Schema Message** (REQUIRED): Defines the Arrow schema
2. **DictionaryBatch Message(s)** (OPTIONAL): Initial dictionaries for dictionary-encoded columns
3. **RecordBatch Message(s)** (REQUIRED): Actual data records

#### 4.3.2 Subsequent Transmissions

After the initial schema transmission, subsequent ArrowPayloads with the same schema_id MAY contain:

1. **DictionaryBatch Message(s)** (OPTIONAL): Delta dictionaries for new values
2. **RecordBatch Message(s)** (REQUIRED): Actual data records

**Note**: Schema messages MUST NOT be repeated unless the schema_id changes.

### 4.4 BatchStatus Acknowledgment

The BatchStatus message provides feedback from server to client about the success or failure of processing a BAR. This acknowledgment mechanism enables reliable delivery—clients can track which BARs have been successfully processed and retry or handle failures for BARs that were rejected.

```protobuf
message BatchStatus {
  int64 batch_id = 1;
  StatusCode status_code = 2;
  string status_message = 3;
}
```

BatchStatus messages flow from server to client over the same bi-directional gRPC stream, allowing the server to acknowledge BARs as they are processed. A status code of OK indicates the BAR was successfully received, decoded, and accepted. Non-OK status codes indicate various error conditions (see section 8 for details).

**Field Descriptions:**

- **batch_id**: The identifier of the BAR being acknowledged. This matches the batch_id from the BatchArrowRecords message that was received.

- **status_code**: Indicates whether processing succeeded or failed, and if failed, what category of error occurred (e.g., invalid data, resource exhaustion, authentication failure).

- **status_message**: Human-readable error details. For OK status, this is typically empty. For errors, this provides context to help diagnose the issue (e.g., "dictionary key overflow in LOG_ATTRS table" or "unknown payload type: 99").

**Requirements:**

Servers MUST send BatchStatus messages to acknowledge received BARs:

- **batch_id**: MUST match the batch_id from the received BatchArrowRecords
- **status_code**: MUST be a valid StatusCode (see section 8.2)
- **status_message**: MAY provide additional context for non-OK statuses

---

## 5. Payload Specifications

This section defines the complete Arrow schema for all OTAP payload types, organized by signal category.

### 5.1 Common Payloads

#### 5.1.1 RESOURCE_ATTRS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| parent_id | UInt16 | UInt16 | No | Yes | Foreign key to parent table's `resource_id` |
| key | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | No | Yes | Attribute key name |
| type | UInt8 | UInt8 | No | Yes | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | Yes | No | String value (when type=1) |
| int | Int64 | Int64 | Yes | No | Integer value (when type=3) |
| double | Float64 | Float64 | Yes | No | Double value (when type=4) |
| bool | Boolean | Boolean | Yes | No | Boolean value (when type=2) |
| bytes | Binary | Binary | Yes | No | Bytes value (when type=5) |
| ser | Binary | Binary | Yes | No | CBOR-encoded Array or Map (when type=6 or 7) |

#### 5.1.2 SCOPE_ATTRS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| parent_id | UInt16 | UInt16 | No | Yes | Foreign key to parent table's `scope_id` |
| key | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | No | Yes | Attribute key name |
| type | UInt8 | UInt8 | No | Yes | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | Yes | No | String value (when type=1) |
| int | Int64 | Int64 | Yes | No | Integer value (when type=3) |
| double | Float64 | Float64 | Yes | No | Double value (when type=4) |
| bool | Boolean | Boolean | Yes | No | Boolean value (when type=2) |
| bytes | Binary | Binary | Yes | No | Bytes value (when type=5) |
| ser | Binary | Binary | Yes | No | CBOR-encoded Array or Map (when type=6 or 7) |

### 5.2 Logs Payloads

#### 5.2.1 LOGS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| id | UInt16 | UInt16 | Yes | Yes | Log record identifier (primary key) |
| resource_id | UInt16 | UInt16 | Yes | No | Foreign key to resource |
| resource_schema_url | Utf8 | Utf8 | Yes | No | Resource schema URL |
| resource_dropped_attributes_count | UInt32 | UInt32 | Yes | No | Number of dropped resource attributes |
| scope_id | UInt16 | UInt16 | Yes | No | Foreign key to scope |
| scope_name | Utf8 | Utf8 | Yes | No | Instrumentation scope name |
| scope_version | Utf8 | Utf8 | Yes | No | Instrumentation scope version |
| scope_dropped_attributes_count | UInt32 | UInt32 | Yes | No | Number of dropped scope attributes |
| schema_url | Utf8 | Utf8 | Yes | No | Log schema URL |
| time_unix_nano | Timestamp(Nanosecond) | Timestamp(Nanosecond) | No | Yes | Log timestamp in Unix nanoseconds |
| observed_time_unix_nano | Timestamp(Nanosecond) | Timestamp(Nanosecond) | No | Yes | Observation timestamp in Unix nanoseconds |
| trace_id | FixedSizeBinary(16) | FixedSizeBinary(16) | Yes | No | Trace ID for correlation |
| span_id | FixedSizeBinary(8) | FixedSizeBinary(8) | Yes | No | Span ID for correlation |
| severity_number | Int32 | Int32 | Yes | No | Numeric severity level |
| severity_text | Utf8 | Utf8 | Yes | No | Textual severity level |
| body_type | UInt8 | UInt8 | No | Yes | Body value type (same encoding as attribute type) |
| body_str | Utf8 | Utf8 | No | Yes | String body (may be empty) |
| body_int | Int64 | Int64 | Yes | No | Integer body (when body_type=3) |
| body_double | Float64 | Float64 | Yes | No | Double body (when body_type=4) |
| body_bool | Boolean | Boolean | Yes | No | Boolean body (when body_type=2) |
| body_bytes | Binary | Binary | Yes | No | Bytes body (when body_type=5) |
| body_ser | Binary | Binary | Yes | No | CBOR-encoded complex body (when body_type=6 or 7) |
| dropped_attributes_count | UInt32 | UInt32 | Yes | No | Number of dropped log attributes |
| flags | UInt32 | UInt32 | Yes | No | Trace flags |

#### 5.2.2 LOG_ATTRS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| parent_id | UInt16 | UInt16 | No | Yes | Foreign key to LOGS.id |
| key | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | No | Yes | Attribute key name |
| type | UInt8 | UInt8 | No | Yes | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | Yes | No | String value (when type=1) |
| int | Int64 | Int64 | Yes | No | Integer value (when type=3) |
| double | Float64 | Float64 | Yes | No | Double value (when type=4) |
| bool | Boolean | Boolean | Yes | No | Boolean value (when type=2) |
| bytes | Binary | Binary | Yes | No | Bytes value (when type=5) |
| ser | Binary | Binary | Yes | No | CBOR-encoded Array or Map (when type=6 or 7) |

### 5.3 Metrics Payloads

#### 5.3.1 UNIVARIATE_METRICS / MULTIVARIATE_METRICS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| id | UInt16 | UInt16 | No | Yes | Metric identifier (primary key) |
| resource_id | UInt16 | UInt16 | Yes | No | Foreign key to resource |
| resource_schema_url | Utf8 | Utf8 | Yes | No | Resource schema URL |
| resource_dropped_attributes_count | UInt32 | UInt32 | Yes | No | Number of dropped resource attributes |
| scope_id | UInt16 | UInt16 | Yes | No | Foreign key to scope |
| scope_name | Utf8 | Utf8 | Yes | No | Instrumentation scope name |
| scope_version | Utf8 | Utf8 | Yes | No | Instrumentation scope version |
| scope_dropped_attributes_count | UInt32 | UInt32 | Yes | No | Number of dropped scope attributes |
| schema_url | Utf8 | Utf8 | Yes | No | Metric schema URL |
| metric_type | UInt8 | UInt8 | No | Yes | Metric type enum (Gauge, Sum, Histogram, etc.) |
| name | Utf8 | Utf8 | No | Yes | Metric name |
| description | Utf8 | Utf8 | Yes | No | Metric description |
| unit | Utf8 | Utf8 | Yes | No | Metric unit |
| aggregation_temporality | Int32 | Int32 | Yes | No | Aggregation temporality enum |
| is_monotonic | Boolean | Boolean | Yes | No | Whether the metric is monotonic |

#### 5.3.2 NUMBER_DATA_POINTS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| id | UInt32 | UInt32 | No | Yes | Data point identifier (primary key) |
| parent_id | UInt16 | UInt16 | No | Yes | Foreign key to METRICS.id |
| start_time_unix_nano | Timestamp(Nanosecond) | Timestamp(Nanosecond) | No | Yes | Start time in Unix nanoseconds |
| time_unix_nano | Timestamp(Nanosecond) | Timestamp(Nanosecond) | No | Yes | Timestamp in Unix nanoseconds |
| int_value | Int64 | Int64 | No | Yes | Integer value |
| double_value | Float64 | Float64 | No | Yes | Double value |
| flags | UInt32 | UInt32 | Yes | No | Data point flags |

#### 5.3.3 SUMMARY_DATA_POINTS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| id | UInt32 | UInt32 | Yes | Yes | Data point identifier (primary key) |
| parent_id | UInt16 | UInt16 | No | Yes | Foreign key to METRICS.id |
| start_time_unix_nano | Timestamp(Nanosecond) | Timestamp(Nanosecond) | Yes | No | Start time in Unix nanoseconds |
| time_unix_nano | Timestamp(Nanosecond) | Timestamp(Nanosecond) | Yes | No | Timestamp in Unix nanoseconds |
| count | UInt64 | UInt64 | Yes | No | Count of observations |
| sum | Float64 | Float64 | Yes | No | Sum of observations |
| quantile | Float64 | Float64, List(Float64) | Yes | No | Quantile values |
| value | Float64 | Float64, List(Float64) | Yes | No | Quantile observation values |
| flags | UInt32 | UInt32 | Yes | No | Data point flags |

#### 5.3.4 HISTOGRAM_DATA_POINTS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| id | UInt32 | UInt32 | Yes | Yes | Data point identifier (primary key) |
| parent_id | UInt16 | UInt16 | No | Yes | Foreign key to METRICS.id |
| start_time_unix_nano | Timestamp(Nanosecond) | Timestamp(Nanosecond) | Yes | No | Start time in Unix nanoseconds |
| time_unix_nano | Timestamp(Nanosecond) | Timestamp(Nanosecond) | Yes | No | Timestamp in Unix nanoseconds |
| count | UInt64 | UInt64 | Yes | No | Count of observations |
| sum | Float64 | Float64 | Yes | No | Sum of observations |
| bucket_counts | UInt64 | List(UInt64) | Yes | No | Count per bucket |
| explicit_bounds | Float64 | List(Float64) | Yes | No | Histogram bucket boundaries |
| flags | UInt32 | UInt32 | Yes | No | Data point flags |
| min | Float64 | Float64 | Yes | No | Minimum value |
| max | Float64 | Float64 | Yes | No | Maximum value |

#### 5.3.5 EXP_HISTOGRAM_DATA_POINTS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| id | UInt32 | UInt32 | Yes | Yes | Data point identifier (primary key) |
| parent_id | UInt16 | UInt16 | No | Yes | Foreign key to METRICS.id |
| start_time_unix_nano | Timestamp(Nanosecond) | Timestamp(Nanosecond) | Yes | No | Start time in Unix nanoseconds |
| time_unix_nano | Timestamp(Nanosecond) | Timestamp(Nanosecond) | Yes | No | Timestamp in Unix nanoseconds |
| count | UInt64 | UInt64 | Yes | No | Count of observations |
| sum | Float64 | Float64 | Yes | No | Sum of observations |
| scale | Int32 | Int32 | Yes | No | Exponential histogram scale |
| zero_count | UInt64 | UInt64 | Yes | No | Count of zero values |
| positive_offset | Int32 | Int32 | Yes | No | Positive bucket offset |
| positive_bucket_counts | UInt64 | List(UInt64) | Yes | No | Positive bucket counts |
| negative_offset | Int32 | Int32 | Yes | No | Negative bucket offset |
| negative_bucket_counts | UInt64 | List(UInt64) | Yes | No | Negative bucket counts |
| flags | UInt32 | UInt32 | Yes | No | Data point flags |
| min | Float64 | Float64 | Yes | No | Minimum value |
| max | Float64 | Float64 | Yes | No | Maximum value |

#### 5.3.6 NUMBER_DP_ATTRS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| parent_id | UInt32 | UInt32 | No | Yes | Foreign key to NUMBER_DATA_POINTS.id |
| key | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | No | Yes | Attribute key name |
| type | UInt8 | UInt8 | No | Yes | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | Yes | No | String value (when type=1) |
| int | Int64 | Int64 | Yes | No | Integer value (when type=3) |
| double | Float64 | Float64 | Yes | No | Double value (when type=4) |
| bool | Boolean | Boolean | Yes | No | Boolean value (when type=2) |
| bytes | Binary | Binary | Yes | No | Bytes value (when type=5) |
| ser | Binary | Binary | Yes | No | CBOR-encoded Array or Map (when type=6 or 7) |

#### 5.3.7 SUMMARY_DP_ATTRS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| parent_id | UInt32 | UInt32 | No | Yes | Foreign key to SUMMARY_DATA_POINTS.id |
| key | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | No | Yes | Attribute key name |
| type | UInt8 | UInt8 | No | Yes | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | Yes | No | String value (when type=1) |
| int | Int64 | Int64 | Yes | No | Integer value (when type=3) |
| double | Float64 | Float64 | Yes | No | Double value (when type=4) |
| bool | Boolean | Boolean | Yes | No | Boolean value (when type=2) |
| bytes | Binary | Binary | Yes | No | Bytes value (when type=5) |
| ser | Binary | Binary | Yes | No | CBOR-encoded Array or Map (when type=6 or 7) |

#### 5.3.8 HISTOGRAM_DP_ATTRS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| parent_id | UInt32 | UInt32 | No | Yes | Foreign key to HISTOGRAM_DATA_POINTS.id |
| key | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | No | Yes | Attribute key name |
| type | UInt8 | UInt8 | No | Yes | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | Yes | No | String value (when type=1) |
| int | Int64 | Int64 | Yes | No | Integer value (when type=3) |
| double | Float64 | Float64 | Yes | No | Double value (when type=4) |
| bool | Boolean | Boolean | Yes | No | Boolean value (when type=2) |
| bytes | Binary | Binary | Yes | No | Bytes value (when type=5) |
| ser | Binary | Binary | Yes | No | CBOR-encoded Array or Map (when type=6 or 7) |

#### 5.3.9 EXP_HISTOGRAM_DP_ATTRS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| parent_id | UInt32 | UInt32 | No | Yes | Foreign key to EXP_HISTOGRAM_DATA_POINTS.id |
| key | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | No | Yes | Attribute key name |
| type | UInt8 | UInt8 | No | Yes | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | Yes | No | String value (when type=1) |
| int | Int64 | Int64 | Yes | No | Integer value (when type=3) |
| double | Float64 | Float64 | Yes | No | Double value (when type=4) |
| bool | Boolean | Boolean | Yes | No | Boolean value (when type=2) |
| bytes | Binary | Binary | Yes | No | Bytes value (when type=5) |
| ser | Binary | Binary | Yes | No | CBOR-encoded Array or Map (when type=6 or 7) |

#### 5.3.10 NUMBER_DP_EXEMPLARS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| id | UInt32 | UInt32 | Yes | Yes | Exemplar identifier (primary key) |
| parent_id | UInt32 | UInt32 | No | Yes | Foreign key to NUMBER_DATA_POINTS.id |
| time_unix_nano | Timestamp(Nanosecond) | Timestamp(Nanosecond) | Yes | No | Timestamp in Unix nanoseconds |
| int_value | Int64 | Int64 | Yes | No | Integer exemplar value |
| double_value | Float64 | Float64 | Yes | No | Double exemplar value |
| span_id | FixedSizeBinary(8) | FixedSizeBinary(8) | Yes | No | Associated span ID |
| trace_id | FixedSizeBinary(16) | FixedSizeBinary(16) | Yes | No | Associated trace ID |

#### 5.3.11 HISTOGRAM_DP_EXEMPLARS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| id | UInt32 | UInt32 | Yes | Yes | Exemplar identifier (primary key) |
| parent_id | UInt32 | UInt32 | No | Yes | Foreign key to HISTOGRAM_DATA_POINTS.id |
| time_unix_nano | Timestamp(Nanosecond) | Timestamp(Nanosecond) | Yes | No | Timestamp in Unix nanoseconds |
| int_value | Int64 | Int64 | Yes | No | Integer exemplar value |
| double_value | Float64 | Float64 | Yes | No | Double exemplar value |
| span_id | FixedSizeBinary(8) | FixedSizeBinary(8) | Yes | No | Associated span ID |
| trace_id | FixedSizeBinary(16) | FixedSizeBinary(16) | Yes | No | Associated trace ID |

#### 5.3.12 EXP_HISTOGRAM_DP_EXEMPLARS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| id | UInt32 | UInt32 | Yes | Yes | Exemplar identifier (primary key) |
| parent_id | UInt32 | UInt32 | No | Yes | Foreign key to EXP_HISTOGRAM_DATA_POINTS.id |
| time_unix_nano | Timestamp(Nanosecond) | Timestamp(Nanosecond) | Yes | No | Timestamp in Unix nanoseconds |
| int_value | Int64 | Int64 | Yes | No | Integer exemplar value |
| double_value | Float64 | Float64 | Yes | No | Double exemplar value |
| span_id | FixedSizeBinary(8) | FixedSizeBinary(8) | Yes | No | Associated span ID |
| trace_id | FixedSizeBinary(16) | FixedSizeBinary(16) | Yes | No | Associated trace ID |

#### 5.3.13 NUMBER_DP_EXEMPLAR_ATTRS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| parent_id | UInt32 | UInt32 | No | Yes | Foreign key to NUMBER_DP_EXEMPLARS.id |
| key | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | No | Yes | Attribute key name |
| type | UInt8 | UInt8 | No | Yes | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | Yes | No | String value (when type=1) |
| int | Int64 | Int64 | Yes | No | Integer value (when type=3) |
| double | Float64 | Float64 | Yes | No | Double value (when type=4) |
| bool | Boolean | Boolean | Yes | No | Boolean value (when type=2) |
| bytes | Binary | Binary | Yes | No | Bytes value (when type=5) |
| ser | Binary | Binary | Yes | No | CBOR-encoded Array or Map (when type=6 or 7) |

#### 5.3.14 HISTOGRAM_DP_EXEMPLAR_ATTRS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| parent_id | UInt32 | UInt32 | No | Yes | Foreign key to HISTOGRAM_DP_EXEMPLARS.id |
| key | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | No | Yes | Attribute key name |
| type | UInt8 | UInt8 | No | Yes | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | Yes | No | String value (when type=1) |
| int | Int64 | Int64 | Yes | No | Integer value (when type=3) |
| double | Float64 | Float64 | Yes | No | Double value (when type=4) |
| bool | Boolean | Boolean | Yes | No | Boolean value (when type=2) |
| bytes | Binary | Binary | Yes | No | Bytes value (when type=5) |
| ser | Binary | Binary | Yes | No | CBOR-encoded Array or Map (when type=6 or 7) |

#### 5.3.15 EXP_HISTOGRAM_DP_EXEMPLAR_ATTRS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| parent_id | UInt32 | UInt32 | No | Yes | Foreign key to EXP_HISTOGRAM_DP_EXEMPLARS.id |
| key | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | No | Yes | Attribute key name |
| type | UInt8 | UInt8 | No | Yes | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | Yes | No | String value (when type=1) |
| int | Int64 | Int64 | Yes | No | Integer value (when type=3) |
| double | Float64 | Float64 | Yes | No | Double value (when type=4) |
| bool | Boolean | Boolean | Yes | No | Boolean value (when type=2) |
| bytes | Binary | Binary | Yes | No | Bytes value (when type=5) |
| ser | Binary | Binary | Yes | No | CBOR-encoded Array or Map (when type=6 or 7) |

#### 5.3.16 METRIC_ATTRS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| parent_id | UInt16 | UInt16 | No | Yes | Foreign key to METRICS.id |
| key | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | No | Yes | Attribute key name |
| type | UInt8 | UInt8 | No | Yes | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | Yes | No | String value (when type=1) |
| int | Int64 | Int64 | Yes | No | Integer value (when type=3) |
| double | Float64 | Float64 | Yes | No | Double value (when type=4) |
| bool | Boolean | Boolean | Yes | No | Boolean value (when type=2) |
| bytes | Binary | Binary | Yes | No | Bytes value (when type=5) |
| ser | Binary | Binary | Yes | No | CBOR-encoded Array or Map (when type=6 or 7) |

### 5.4 Traces Payloads

#### 5.4.1 SPANS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| id | UInt16 | UInt16 | Yes | Yes | Span identifier (primary key) |
| resource_id | UInt16 | UInt16 | Yes | No | Foreign key to resource |
| resource_schema_url | Utf8 | Utf8 | Yes | No | Resource schema URL |
| resource_dropped_attributes_count | UInt32 | UInt32 | Yes | No | Number of dropped resource attributes |
| scope_id | UInt16 | UInt16 | Yes | No | Foreign key to scope |
| scope_name | Utf8 | Utf8 | Yes | No | Instrumentation scope name |
| scope_version | Utf8 | Utf8 | Yes | No | Instrumentation scope version |
| scope_dropped_attributes_count | UInt32 | UInt32 | Yes | No | Number of dropped scope attributes |
| schema_url | Utf8 | Utf8 | Yes | No | Span schema URL |
| start_time_unix_nano | Timestamp(Nanosecond) | Timestamp(Nanosecond) | No | Yes | Span start time in Unix nanoseconds |
| duration_time_unix_nano | Duration(Nanosecond) | Duration(Nanosecond) | No | Yes | Span duration in nanoseconds |
| trace_id | FixedSizeBinary(16) | FixedSizeBinary(16) | No | Yes | Trace ID |
| span_id | FixedSizeBinary(8) | FixedSizeBinary(8) | No | Yes | Span ID |
| trace_state | Utf8 | Utf8 | Yes | No | W3C trace state |
| parent_span_id | FixedSizeBinary(8) | FixedSizeBinary(8) | Yes | No | Parent span ID |
| name | Utf8 | Utf8 | No | Yes | Span name |
| kind | Int32 | Int32 | Yes | No | Span kind enum |
| dropped_attributes_count | UInt32 | UInt32 | Yes | No | Number of dropped span attributes |
| dropped_events_count | UInt32 | UInt32 | Yes | No | Number of dropped events |
| dropped_links_count | UInt32 | UInt32 | Yes | No | Number of dropped links |
| status_code | Int32 | Int32 | Yes | No | Span status code |
| status_status_message | Utf8 | Utf8 | Yes | No | Status message |

#### 5.4.2 SPAN_ATTRS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| parent_id | UInt16 | UInt16 | No | Yes | Foreign key to SPANS.id |
| key | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | No | Yes | Attribute key name |
| type | UInt8 | UInt8 | No | Yes | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | Yes | No | String value (when type=1) |
| int | Int64 | Int64 | Yes | No | Integer value (when type=3) |
| double | Float64 | Float64 | Yes | No | Double value (when type=4) |
| bool | Boolean | Boolean | Yes | No | Boolean value (when type=2) |
| bytes | Binary | Binary | Yes | No | Bytes value (when type=5) |
| ser | Binary | Binary | Yes | No | CBOR-encoded Array or Map (when type=6 or 7) |

#### 5.4.3 SPAN_EVENTS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| id | UInt32 | UInt32 | Yes | Yes | Event identifier (primary key) |
| parent_id | UInt16 | UInt16 | No | Yes | Foreign key to SPANS.id |
| time_unix_nano | Timestamp(Nanosecond) | Timestamp(Nanosecond) | Yes | No | Event timestamp in Unix nanoseconds |
| name | Utf8 | Utf8 | No | Yes | Event name |
| dropped_attributes_count | UInt32 | UInt32 | Yes | No | Number of dropped event attributes |

#### 5.4.4 SPAN_EVENT_ATTRS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| parent_id | UInt32 | UInt32 | No | Yes | Foreign key to SPAN_EVENTS.id |
| key | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | No | Yes | Attribute key name |
| type | UInt8 | UInt8 | No | Yes | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | Yes | No | String value (when type=1) |
| int | Int64 | Int64 | Yes | No | Integer value (when type=3) |
| double | Float64 | Float64 | Yes | No | Double value (when type=4) |
| bool | Boolean | Boolean | Yes | No | Boolean value (when type=2) |
| bytes | Binary | Binary | Yes | No | Bytes value (when type=5) |
| ser | Binary | Binary | Yes | No | CBOR-encoded Array or Map (when type=6 or 7) |

#### 5.4.5 SPAN_LINKS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| id | UInt32 | UInt32 | Yes | Yes | Link identifier (primary key) |
| parent_id | UInt16 | UInt16 | No | Yes | Foreign key to SPANS.id |
| trace_id | FixedSizeBinary(16) | FixedSizeBinary(16) | Yes | No | Linked trace ID |
| span_id | FixedSizeBinary(8) | FixedSizeBinary(8) | Yes | No | Linked span ID |
| trace_state | Utf8 | Utf8 | Yes | No | Linked trace state |
| dropped_attributes_count | UInt32 | UInt32 | Yes | No | Number of dropped link attributes |

#### 5.4.6 SPAN_LINK_ATTRS

| Name | Value Type | Valid Types | Nullable | Required | Description |
|------|------------|-------------|----------|----------|-------------|
| parent_id | UInt32 | UInt32 | No | Yes | Foreign key to SPAN_LINKS.id |
| key | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | No | Yes | Attribute key name |
| type | UInt8 | UInt8 | No | Yes | Value type: 0=None, 1=String, 2=Bool, 3=Int, 4=Double, 5=Bytes, 6=Array, 7=Map |
| str | Utf8 | Utf8, Dictionary(UInt8\|UInt16\|UInt32, Utf8) | Yes | No | String value (when type=1) |
| int | Int64 | Int64 | Yes | No | Integer value (when type=3) |
| double | Float64 | Float64 | Yes | No | Double value (when type=4) |
| bool | Boolean | Boolean | Yes | No | Boolean value (when type=2) |
| bytes | Binary | Binary | Yes | No | Bytes value (when type=5) |
| ser | Binary | Binary | Yes | No | CBOR-encoded Array or Map (when type=6 or 7) |

### 5.5 Dictionary Encoding

Dictionary encoding is a compression technique where repeated values are stored once in a separate dictionary, and the actual column contains only small integer keys that index into that dictionary. This is particularly effective for string columns with limited cardinality, such as attribute keys, metric names, or resource types.

For example, if 10,000 log records all have the attribute key "http.method", rather than storing the string "http.method" 10,000 times, we store it once in a dictionary and use small integer keys (0-10) in the actual column. This can reduce data size by 10x or more for high-repetition columns.

OTAP supports dictionary encoding through Arrow's built-in dictionary types. Dictionaries are stateful—once established, they persist across multiple BARs, allowing subsequent BARs to reference previously defined dictionary entries without retransmitting them. When new values appear, clients send delta dictionaries that add entries without replacing existing ones.

#### 5.5.1 Allowed Dictionary Key Types

Dictionary keys MUST use one of these unsigned integer types:
- **UInt8**: For dictionaries with ≤256 unique values
- **UInt16**: For dictionaries with ≤65,536 unique values
- **UInt32**: For dictionaries with ≤4,294,967,296 unique values

**Recommendation**: Producers SHOULD select the smallest key type that can accommodate expected cardinality, with appropriate overflow handling (see 7.3).

#### 5.5.2 Allowed Dictionary Value Types

Dictionary values MUST be one of:
- **Utf8**: For string values
- **Binary**: For byte array values

#### 5.5.3 Delta Dictionaries

Producers MAY send delta dictionaries to add new entries without resetting the schema. Delta dictionary rules:

1. Delta dictionaries MUST only add new key-value pairs
2. Delta dictionaries MUST NOT modify existing entries
3. Consumers MUST merge delta dictionaries with existing dictionary state
4. Key values in delta dictionaries MUST NOT conflict with existing keys

---

## 6. Id Columns

This section defines the identifier columns used to establish relationships between payload types in the OTAP data model.

### 6.1 Primary Keys and Foreign Keys

All parent-child relationships in the OTAP data model follow a uniform convention:

- **Parent tables** define an `id` column as their primary key
- **Child tables** define a `parent_id` column as a foreign key that always references their parent table's `id` column

This naming convention makes the data model self-documenting: every `parent_id` column references the `id` column of its parent table as specified in the payload type relationships (see Section 2.1).

**Example**: In the Logs signal:
- The LOGS table has an `id` column (UInt16)
- The LOG_ATTRS table has a `parent_id` column (UInt16) that references LOGS.`id`
- Each LOG_ATTRS row belongs to exactly one LOGS row via this foreign key

### 6.2 Id Column Types

Id columns use unsigned integer types sized according to expected cardinality:

| Table Category | Id Type | Parent Id Type | Max Entries |
|---|---|---|---|
| Root tables (LOGS, SPANS, METRICS) | UInt16 | — | 65,536 per BAR |
| Data points, events, links | UInt32 | UInt16 | 4.3B per BAR |
| Exemplars | UInt32 | UInt32 | 4.3B per BAR |
| Attributes (root-level) | — | UInt16 | Many per parent |
| Attributes (nested) | — | UInt32 | Many per parent |

**Rationale**: Root tables rarely exceed 65K items per BAR, but child tables (especially data points and their nested children) can have much higher cardinality, hence the UInt32 type.

### 6.3 Resource and Scope Identifiers

Resource and scope entities are **not** represented as separate payload types. Instead, they are embedded as struct fields within root tables (LOGS, SPANS, METRICS).

Each root table contains:
- `resource_id` (UInt16): Identifier for the resource
- `scope_id` (UInt16): Identifier for the instrumentation scope

These fields are commonly referenced as `resource.id` and `scope.id` in the context of struct field access.

**Key characteristics**:

1. **No dedicated tables**: There are no RESOURCE or SCOPE payload types. Resources and scopes are defined implicitly by their presence in root table rows.

2. **Many-to-many relationships**: Multiple Items (Logs, Data Points, or Spans) MAY share the same `resource_id` or `scope_id`. This is a many-to-many relationship.

3. **Attribute tables**: RESOURCE_ATTRS and SCOPE_ATTRS tables reference these identifiers via their `parent_id` columns:
   - RESOURCE_ATTRS.`parent_id` → root table's `resource_id`
   - SCOPE_ATTRS.`parent_id` → root table's `scope_id`

4. **No owning table**: Unlike other identifiers, `resource_id` and `scope_id` have no single table that "owns" them. Their definition is implicit across all Items that reference them.

**Example**: Consider a BAR containing 1000 logs from 3 services (resources):
- LOGS table: 1000 rows with `resource_id` values of 0, 1, or 2
- RESOURCE_ATTRS table: Rows with `parent_id` of 0, 1, or 2 defining attributes for each resource
- No separate RESOURCE table exists; the resource is defined by the combination of `resource_id` and its associated RESOURCE_ATTRS rows

**Design rationale**: Resources and scopes have minimal intrinsic properties (just schema_url, name, version, dropped_attributes_count) which are duplicated in root tables. Creating separate payload types would add complexity for little benefit, so OTAP embeds these fields directly in root tables.

### 6.4 Relationship DAG

The `id` and `parent_id` columns form a Rooted Directed Acyclic Graph (DAG) for each signal:

- **Root**: The root payload type (LOGS, SPANS, or METRICS) with `id` but no `parent_id`
- **Edges**: Each `parent_id` column creates an edge to the parent table's `id`
- **Leaf nodes**: Attribute tables with `parent_id` but no `id`
- **Intermediate nodes**: Tables with both `id` (primary key) and `parent_id` (foreign key)

This structure ensures:
- No circular references
- Clear parent-child hierarchies
- Efficient foreign key resolution

See Section 2.1 for the complete DAG structure of each signal type.

### 6.5 Transport Optimized Encodings

Beyond Arrow's built-in compression features, OTAP defines specialized column encodings that transform ID and parent_id columns before serialization to maximize compression efficiency during network transport. These transformations exploit patterns in telemetry data, such as sequential IDs and clustered foreign keys.

The key insight is that ID columns often exhibit strong sequential patterns:
- Primary IDs are often sequential (0, 1, 2, 3...)
- Foreign keys (`parent_id`) are often clustered (many attributes reference the same parent item)
- When sorted by `parent_id`, related records appear together

By encoding these patterns explicitly (e.g., storing deltas between values rather than absolute values), we create long runs of small integers and repeated values that compress extremely well with standard algorithms like LZ4 or Zstandard.

**Important**: These encodings apply **only to ID columns** (`id`, `parent_id`, `resource_id`, `scope_id`). Other columns use plain encoding or dictionary encoding as appropriate.

#### 6.5.1 PLAIN Encoding

**Encoding identifier**: `"plain"`

No transformation applied. Values are stored as-is in the Arrow array.

**Applicability**: All ID columns

**When to use**: Default encoding when no optimization is beneficial or when concatenating BARs.

#### 6.5.2 DELTA Encoding

**Encoding identifier**: `"delta"`

Stores the difference between consecutive values instead of absolute values. Used for columns that are sorted and contain sequential or near-sequential values.

**Algorithm**:
```
encoded[0] = original[0]
encoded[i] = original[i] - original[i-1]  (for i > 0)
```

**Applicability**: Unsigned integer columns that are sorted

**Typical use**: Primary `id` columns and `parent_id` columns in data point tables when sorted by parent

**Requirements**:
- Column MUST be sorted in ascending order
- Column type MUST support the delta operation
- For UInt types, deltas are stored as the same UInt type (no negatives)

#### 6.5.3 QUASI-DELTA Encoding

**Encoding identifier**: `"quasidelta"`

A hybrid encoding that applies delta encoding selectively. Parent IDs are delta-encoded only within "runs" of rows that share the same attribute key/value or other identifying columns.

**Algorithm for Attributes**:
```
For each row i:
  If row i matches row i-1 on (type, key, str, int, double, bool, bytes):
    encoded[i] = parent_id[i] - parent_id[i-1]
  Else:
    encoded[i] = parent_id[i]  (absolute value)
```

**Algorithm for Columnar Quasi-Delta**:
Similar, but matching based on specified column values (e.g., span event `name` field, exemplar `int_value`/`double_value`).

**Applicability**: `parent_id` columns in attribute and related tables

**Typical use**:
- Attribute table `parent_id` columns
- Span event/link `parent_id` columns
- Exemplar `parent_id` columns

#### 6.5.4 Field Metadata

Producers SHOULD include field metadata to indicate encoding:

```json
{
  "encoding": "delta" | "plain" | "quasidelta"
}
```

**Requirements**:
- If metadata is present, `encoding` field SHOULD indicate the applied encoding
- If metadata is absent, consumers SHOULD assume the column is encoded (for backward compatibility with implementations that don't write metadata)
- Consumers MUST handle both presence and absence of metadata

**Note**: The Rust implementation includes this metadata; the Go implementation may not.

#### 6.5.5 Schema Metadata

Producers MAY include schema-level metadata:

```json
{
  "sort_columns": "field1,field2,..."
}
```

This indicates the columns by which the record batch has been sorted, which is useful context for understanding applied encodings.

#### 6.5.6 Encoding Application by Payload Type

The following table specifies recommended encodings per payload type:

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

#### 6.5.7 Decoding Requirements

Consumers MUST:
1. Detect encoding type from field metadata or use heuristics if absent
2. Reverse the encoding to reconstruct original values before processing
3. Handle both encoded and plain data transparently

#### 6.5.8 Default Encoding

**Recommended defaults**:
- If transport optimization is not enabled: Use **PLAIN** encoding for all ID columns
- If transport optimization is enabled: Use encodings per section 6.5.6

---

## 7. Schema Management

One of OTAP's key features is dynamic schema management. Unlike protocols with fixed schemas (like OTLP's protobuf definitions), OTAP allows schemas to evolve during a connection's lifetime. This flexibility is essential for handling events like dictionary overflow, where a UInt16 dictionary exceeds 65,536 unique values and must be upgraded to UInt32.

Schema management in OTAP is coordinated through schema identifiers. Each unique schema configuration receives a unique ID, and consumers track which schemas they've seen. When a new schema_id appears, consumers know to reset their Arrow IPC readers and expect a new schema definition.

### 7.1 Schema Identification

Each Arrow schema for a given payload type is identified by a unique `schema_id` string. This identifier serves as a contract between producer and consumer: "the data in this payload conforms to the schema identified by this ID."

The schema_id allows both parties to recognize when a schema has changed without explicitly signaling "reset your state." If the consumer sees a schema_id it hasn't encountered before (for a given payload type), it knows the schema has changed.

**Requirements**:
- Schema IDs MUST be unique within a payload type
- Schema IDs MUST be deterministic: same schema structure produces same ID
- Schema IDs MAY be any string format, but SHOULD be compact

### 7.2 Schema ID Generation

**Recommended algorithm**:

1. Sort fields by name at each nesting level
2. Generate compact representation:
   - Field name
   - `:` separator
   - Type abbreviation (e.g., `U16` for UInt16, `Str` for Utf8, `Dic<U16,Str>` for Dictionary)
3. Concatenate fields with `,` separator

**Example**: `id:U16,parent_id:U16,key:Str,type:U8,str:Dic<U16,Str>`

**Note**: Implementations MAY use alternative deterministic algorithms (e.g., hash-based), but MUST ensure:
- Identical schemas produce identical IDs
- Different schemas produce different IDs with high probability

### 7.3 Schema Reset Triggers

A schema reset occurs when the producer needs to change the structure, types, or encoding of a table's schema. This is communicated by changing the schema_id and sending a new Schema message. Schema resets are necessary in several scenarios, all related to the schema no longer being adequate for the data being sent.

Producers MUST perform a schema reset (change schema_id) when:

1. **Dictionary overflow**: Dictionary key type is too small for cardinality
   - Example: UInt16 dictionary exceeds 65,536 unique values
   - Solution: Upgrade to UInt32 dictionary encoding

2. **Field addition/removal**: Schema structure changes
   - Example: Adding a new optional field
   - Solution: Create new schema with updated field list

3. **Type changes**: Field data type changes
   - Example: Changing from UInt16 to UInt32 for an ID field
   - Solution: Create new schema with updated type

4. **Encoding changes**: Dictionary encoding added or removed for a field
   - Example: Converting a plain string field to dictionary-encoded
   - Solution: Create new schema reflecting the encoding change

**Note**: Metadata-only changes (e.g., updating `encoding` metadata) do NOT require schema reset.

### 7.4 Schema Reset Procedure

When performing a schema reset:

1. **Determine new schema**: Create Arrow schema with necessary changes
2. **Generate new schema_id**: Compute unique identifier for new schema
3. **Send Schema message**: Include Schema message in next ArrowPayload with new schema_id
4. **Initialize dictionaries**: Send initial DictionaryBatch messages if needed
5. **Continue transmission**: Send RecordBatch messages using new schema

**Consumer behavior**:
- Upon receiving a new schema_id for a payload type, consumers MUST:
  1. Close any existing Arrow IPC readers for that payload type
  2. Discard dictionary state for old schema
  3. Create new Arrow IPC reader for the new schema
  4. Initialize new dictionary state as DictionaryBatch messages arrive

### 7.5 Schema Compatibility

OTAP does NOT require forward or backward schema compatibility. Consumers need only handle the specific schema identified by schema_id.

---

## 8. Error Handling

Robust error handling is critical for reliable telemetry collection. OTAP uses gRPC status codes to signal different error conditions, allowing clients to distinguish between transient failures (that should be retried) and permanent failures (that indicate bugs or misconfigurations).

Error handling in OTAP operates at two levels:

1. **BAR-level errors**: Reported via BatchStatus messages with non-OK status codes
2. **Stream-level errors**: Reported by closing the gRPC stream with an error status

Understanding which errors are retryable versus non-retryable is essential for implementing correct client behavior. Retrying non-retryable errors wastes resources, while failing to retry retryable errors can lead to data loss.

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

This section provides detailed semantics for fields in OTAP schemas, including which fields are required versus optional, special handling rules for attribute and body fields, and field metadata conventions.

Understanding field requirements is important for both producers (to ensure they send valid data) and consumers (to know which fields they can rely on being present). OTAP inherits most field semantics from OTLP but adapts them to the columnar model.

### 9.1 Required vs Optional Fields

Fields in OTAP schemas are categorized based on whether they must always have a value. Note that Arrow's type system represents nullable and non-nullable separately from semantic requirements—a field may be marked nullable in the Arrow schema for technical reasons but be semantically required to have a value.

Fields in OTAP schemas are categorized as:

#### 9.1.1 Required (Non-Nullable) Fields

These fields MUST always have a value (though Arrow arrays may have nulls for compatibility):

**All Signals**:
- Primary table `id` (though marked nullable in schema, semantically required)
- Attribute `parent_id`
- Attribute `key`
- Attribute `type`

**Logs**:
- `time_unix_nano`
- `observed_time_unix_nano`
- `body_type`
- `body_str` (even if empty string)

**Metrics**:
- Primary table `id`
- `metric_type`
- `name`
- Data point `id`
- Data point `parent_id`
- Data point timestamp fields

**Traces**:
- `start_time_unix_nano`
- `duration_time_unix_nano`
- `trace_id`
- `span_id`
- `name`

#### 9.1.2 Optional (Nullable) Fields

These fields MAY be null/absent:

- Most metadata fields: `description`, `unit`, `schema_url`
- Foreign keys: `resource_id`, `scope_id` (when not present)
- Trace correlation: `trace_id`, `span_id` in logs
- All attribute value fields except the one matching `type`
- Counter fields: `dropped_attributes_count`, `flags`

**Semantics**: Optional fields that are null/absent indicate the value is not present or not applicable.

### 9.2 Special Field Rules

#### 9.2.1 Attribute Value Fields

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

#### 9.2.2 Body Fields (Logs)

For logs, the `body_type` field determines which `body_*` field is populated, similar to attribute fields.

#### 9.2.3 Exemplar Value Fields

For exemplar tables, either `int_value` or `double_value` MUST be non-null (or both may be present with appropriate semantics).

### 9.3 Field Metadata

Fields MAY include metadata key-value pairs:

**Standard metadata keys**:

| Key | Values | Meaning |
|-----|--------|---------|
| `encoding` | `"plain"`, `"delta"`, `"quasidelta"` | Transport encoding applied |

**Custom metadata**:
- Implementations MAY define custom metadata keys
- Unknown metadata keys SHOULD be ignored by consumers

---

## 10. Compliance Requirements

This section defines what it means to be a compliant OTAP producer or consumer. Compliance ensures interoperability—any compliant producer should be able to communicate with any compliant consumer, regardless of implementation language or vendor.

Requirements are divided into MUST (mandatory for compliance) and SHOULD (strongly recommended but not strictly required). Following the SHOULD requirements improves efficiency, debuggability, and user experience, but violating them doesn't break protocol correctness.

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

### C.2 Debugging

1. **Schema ID mismatches**: Compare schema structures field-by-field
2. **Dictionary errors**: Verify DictionaryBatch sent before use
3. **Encoding issues**: Check field metadata and verify decoding logic
4. **Foreign key violations**: Validate parent_id references exist in parent table

### C.3 Testing

Implementations SHOULD test:

1. Schema reset scenarios (dictionary overflow, type changes)
2. Delta and quasi-delta encoding correctness
3. Empty table handling (omitted payloads)
4. Unknown field handling (forward compatibility)
5. Error code correctness (retryable vs non-retryable)
6. Memory limit enforcement
7. Round-trip fidelity (encode/decode produces identical data)

---

## Appendix D: Changes from OTLP

Major differences from OTLP:

1. **Format**: Columnar (Arrow) vs row-based (Protobuf)
2. **Compression**: Built-in Arrow compression vs external compression
3. **Schema evolution**: Dynamic schemas with schema_id vs fixed protobuf schema
4. **Dictionaries**: Stateful dictionary encoding vs no dictionary support
5. **Normalization**: Related tables vs nested messages
6. **Transport optimization**: Built-in encodings vs no optimization

---

## Appendix E: Load Balancing

OTAP's stateful, long-lived gRPC streams introduce load-balancing challenges that do not arise with stateless unary RPCs. Because gRPC multiplexes streams over a single HTTP/2 connection, L4 (TCP-level) load balancers distribute work at connection granularity, not per-stream. Combined with kernel `SO_REUSEPORT` hashing, too few client connections can pin traffic to a single backend core.

Key considerations include:

- **Connection fan-out**: Clients SHOULD open multiple gRPC channels (connections) to provide enough entropy for balanced distribution across backend listeners.
- **Stream lifetime management**: Periodically recycling OTAP streams bounds dictionary growth and allows downstream rebalancers to redistribute load, at the cost of resending schemas and dictionaries.
- **L7 load balancing**: An HTTP/2-aware proxy (e.g., Envoy, NGINX) can distribute individual gRPC streams across backends, which is the recommended approach for long-lived streaming RPCs.
- **Server-side enforcement**: Servers SHOULD enforce memory and dictionary size limits regardless of client behavior.

For a detailed treatment of challenges, solution techniques (client-side and server-side), and recommended baseline configurations, see [Load Balancing: Challenges & Solutions](rust/otap-dataflow/docs/load-balancing.md).

---

## Appendix F: Glossary

- **Client/Producer**: The sender of telemetry data
- **Server/Consumer**: The receiver of telemetry data
- **Signal**: One of logs, metrics, or traces
- **Payload**: An ArrowPayload containing serialized Arrow IPC messages
- **BAR**: Abbreviation for BatchArrowRecords, the client gRPC message
- **Items**: The item type of a signal e.g. Log, Data Point(s), or Span
- **Root Payload/Root Payload Type**: The root table in the Signal's DAG
- **Schema Reset**: The act of changing the Arrow schema for a payload type
- **OTLP Specification**: OpenTelemetry Protocol specification
- **Apache Arrow IPC Format**: https://arrow.apache.org/docs/format/Columnar.html#ipc-streaming-format
- **Payload Type**: Also referred to as ArrowPayloadType, this is equivalent to a distinct table in the OTAP data model
- **gRPC**: https://grpc.io/
- **OTEP 0156**: Columnar Encoding proposal

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-02-10 | Initial formal specification |

---

## Appendix G: References

1. Apache Arrow IPC Format: https://arrow.apache.org/docs/format/Columnar.html
2. OTLP Specification: https://opentelemetry.io/docs/specs/otlp/
3. gRPC Status Codes: https://grpc.io/docs/guides/status-codes/
4. OTEP 0156: https://github.com/open-telemetry/oteps/blob/main/text/0156-columnar-encoding.md
5. Reference Implementation (Go): [Producer](https://github.com/open-telemetry/otel-arrow/blob/main/pkg/otel/arrow_record/producer.go), [Consumer](https://github.com/open-telemetry/otel-arrow/blob/main/pkg/otel/arrow_record/consumer.go)
6. Rust Implementation: otap-dataflow/crates/pdata
7. RFC 2119: https://www.rfc-editor.org/rfc/rfc2119

