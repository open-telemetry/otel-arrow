# otel-arrow-contrib-data-engine-recordset

> **Experimental, pre-1.0.** This crate has no stability guarantees. Types,
> APIs, and evaluation behavior may change or be removed without notice,
> including in patch releases. It is published so other pre-1.0 data-engine
> crates can depend on a versioned registry package; it is not yet
> recommended for production use.

## Overview

`otel-arrow-contrib-data-engine-recordset` is a record-set evaluation engine for
[`otel-arrow-contrib-data-engine-expressions`](https://crates.io/crates/otel-arrow-contrib-data-engine-expressions)
`PipelineExpression`s. It evaluates a parsed pipeline (produced by a parser
such as
[`otel-arrow-contrib-data-engine-kql-parser`](https://crates.io/crates/otel-arrow-contrib-data-engine-kql-parser))
against a batch of caller-supplied records, applying transform, filter, and
summary expressions and producing included, dropped, and summarized results
with optional diagnostics.

Records are consumed through the crate's `Record` trait, so any owned or
borrowed data model can be adapted for evaluation without copying into a
crate-specific representation up front. This folder also contains the
design for hierarchical, time-windowed summaries described below.

## Principal types

- `RecordSetEngine`, `RecordSetEngineOptions` - the entry point; configure
  the diagnostic level, summary cardinality limit, and external function
  callbacks, then evaluate pipelines against batches of records.
- `RecordSetEngineBatch` - a single evaluation batch: push records with
  `push_records`, then `flush` to obtain results.
- `RecordSetEngineCounters`, `RecordSetEngineDiagnostic`,
  `RecordSetEngineDiagnosticLevel` - the execution counters and diagnostics
  produced by a batch.
- `OwnedValue`, `ResolvedValue`, `MapValueStorage` and related primitives -
  the value model used to represent and mutate record data during
  evaluation.

## Usage

```rust,ignore
use otel_arrow_contrib_data_engine_kql_parser::{KqlParser, Parser};
use otel_arrow_contrib_data_engine_recordset::{RecordSet, RecordSetEngine};

// `MyRecords` implements the crate's `RecordSet<MyRecord>` trait, where
// `MyRecord` implements `Record`, adapting your data model for evaluation.
fn evaluate(records: &mut MyRecords, query: &str) {
    let parsed = KqlParser::parse(query).expect("valid query");

    let engine = RecordSetEngine::new();
    let mut batch = engine
        .begin_batch(&parsed.pipeline)
        .expect("batch initializes");

    let dropped_at_push = batch.push_records(records);
    let results = batch.flush();

    println!(
        "included={} dropped={} summarized={}",
        results.included_records.len(),
        results.dropped_records.len() + dropped_at_push.len(),
        results.counters.summarized,
    );
}
```

## Summary Design

The current implementation contains a design for summaries. It is likely to
change partially or wholly but this is how it works at the moment.

### Goals

- A summary should include record counts based on some grouping criteria for
  some time window.

- Summaries should support late-arriving data and be combinable.

- A configurable sample\set of raw logs may also be included otherwise records
  are dropped.

### Example

Consider the following set up:

```text
pipeline: [
 summarize_by
 (
  predicate: (
   equal_to(
    left: resolve("@resource:attributes:'service.name'"),
    right: static(StringValue("MyService")))
   )
   &&
   equal_to(
    left: resolve("@instrumentation_scope:name"),
    right: static(StringValue("CategoryNameB")))
  ),
  window: Timestamp("00:05:00"),
  values: [
   resolve("@attributes[event_id]")
  ],
  reservoir: SimpleReservoir(2)
 )
]
```

User has asked for a 5min summary over `Timestamp` of records with
`resource.attributes['service.name'] == 'MyService'` and
`instrumentation_scope.name == 'CategoryNameB'` grouped by the
`attributes['event_id']` values. User has configured a `SimpleReservoir` with a
size of 2.

Let's say the engine is sent 6 records:

```text
resourceMyService { attributes: { service.name: MyService } }
resourceOtherService { attributes: { service.name: OtherService } }

scopeA { name: CategoryNameA }
scopeB { name: CategoryNameB }

log1 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:01am, attributes: { event_id: 1 } }
log2 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:02am, attributes: { event_id: 1 } }
log3 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:02am, attributes: { event_id: 1 } }
log4 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:02am, attributes: { event_id: 2 } }
log5 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:03am, attributes: { event_id: 1 } }
log6 [ resourceOtherService, scopeA ] { timestamp: 6/1/2025 10:03am, attributes: { event_id: 1 } }
```

The engine will output:

```text
included_records: [
 log1 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:01am, attributes: { event_id: 1 }, summary_id: 9aa6 }
 log4 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:02am, attributes: { event_id: 2 }, summary_id: 7aeb }
 log5 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:03am, attributes: { event_id: 1 }, summary_id: 9aa6 }
 log6 [ resourceOtherService, scopeA ] { timestamp: 6/1/2025 10:03am, attributes: { event_id: 1 } }
]

dropped_records: [
 log2 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:02am, attributes: { event_id: 1 } }
 log3 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:02am, attributes: { event_id: 1 } }
]

summaries: [
 {
  id: 9aa6,
  observed_timestamp: 6/1/2025 10:10am,
  window_type: Timestamp(00:05:00),
  window_start: 6/1/2025 10:00am,
  window_end: 6/1/2025 10:05am,
  grouping: [
   {
    key: resource.attributes[service.name],
    value: MyService
   },
   {
    key: instrumentation_scope.name,
    value: CategoryNameB
   },
   {
    key: attributes[event_id],
    value: 1
   }
  ],
  included_count: 2,
  total_count: 4
 },
 {
  id: 7aeb,
  observed_timestamp: 6/1/2025 10:10am,
  window_type: Timestamp(00:05:00),
  window_start: 6/1/2025 10:00am,
  window_end: 6/1/2025 10:05am,
  grouping: [
   {
    key: resource.attributes[service.name],
    value: MyService
   },
   {
    key: instrumentation_scope.name,
    value: CategoryNameB
   },
   {
    key: attributes[event_id],
    value: 2
   }
  ],
  included_count: 1,
  total_count: 1
 }
]
```

The engine outputs the summary information separately from the records. For this
run 2 summaries were created because `event_id` had 2 distinct values. The user
asked for summaries on `event_id` but resource and scope information was
automatically included due to the predicate set in place for the summary
expression. Each summary is given an id. The id is a SHA256 hash of all the
information in the summary. This is meant to make them deterministic.

Individual log records are either dropped or included. If a record is included,
it is assigned the summary id.

Let's look at late-arriving data. Say the engine receives this data the next
day:

```text
resourceMyService { attributes: { service.name: MyService } }
resourceOtherService { attributes: { service.name: OtherService } }

scopeA { name: CategoryNameA }
scopeB { name: CategoryNameB }

log1 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:01am, attributes: { event_id: 1 } }
log2 [ resourceMyService, scopeB ] { timestamp: 6/2/2025 11:02am, attributes: { event_id: 1 } }
log3 [ resourceMyService, scopeB ] { timestamp: 6/2/2025 11:02am, attributes: { event_id: 1 } }
log4 [ resourceMyService, scopeB ] { timestamp: 6/2/2025 11:02am, attributes: { event_id: 2 } }
log5 [ resourceMyService, scopeB ] { timestamp: 6/2/2025 11:03am, attributes: { event_id: 1 } }
```

`log1` in this case is late-arriving. It has a timestamp from the previous day.

The engine will output:

```text
included_records: [
 log1 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:01am, attributes: { event_id: 1 }, summary_id: 9aa6 }
 log2 [ resourceMyService, scopeB ] { timestamp: 6/2/2025 11:02am, attributes: { event_id: 1 }, summary_id: 1bce }
 log4 [ resourceMyService, scopeB ] { timestamp: 6/2/2025 11:02am, attributes: { event_id: 2 }, summary_id: f0ae }
 log5 [ resourceMyService, scopeB ] { timestamp: 6/2/2025 11:03am, attributes: { event_id: 1 }, summary_id: 1bce }
]

dropped_records: [
 log3 [ resourceMyService, scopeB ] { timestamp: 6/2/2025 11:02am, attributes: { event_id: 1 } }
]

summaries: [
 {
  id: 9aa6,
  observed_timestamp: 6/1/2025 10:10am,
  window_type: Timestamp(00:05:00),
  window_start: 6/1/2025 10:00am,
  window_end: 6/1/2025 10:05am,
  grouping: [
   {
    key: resource.attributes[service.name],
    value: MyService
   },
   {
    key: instrumentation_scope.name,
    value: CategoryNameB
   },
   {
    key: attributes[event_id],
    value: 1
   }
  ],
  included_count: 1,
  total_count: 1
 },
 {
  id: 1bce,
  observed_timestamp: 6/2/2025 11:10am,
  window_type: Timestamp(00:05:00),
  window_start: 6/2/2025 11:00am,
  window_end: 6/2/2025 11:05am,
  grouping: [
   {
    key: resource.attributes[service.name],
    value: MyService
   },
   {
    key: instrumentation_scope.name,
    value: CategoryNameB
   },
   {
    key: attributes[event_id],
    value: 1
   }
  ],
  included_count: 2,
  total_count: 3
 },
 {
  id: f0ae,
  observed_timestamp: 6/2/2025 11:10am,
  window_type: Timestamp(00:05:00),
  window_start: 6/2/2025 11:00am,
  window_end: 6/2/2025 11:05am,
  grouping: [
   {
    key: resource.attributes[service.name],
    value: MyService
   },
   {
    key: instrumentation_scope.name,
    value: CategoryNameB
   },
   {
    key: attributes[event_id],
    value: 2
   }
  ],
  included_count: 1,
  total_count: 1
 }
]
```

For this run 3 summaries were output. The late-arriving record was included in
its own summary with the same id as the previous run. This is due to the
deterministic generation of summary ids.

The idea here is the late-arriving summary can be forwarded along and can be
combined downstream to achieve eventual consistency.

### Forwarding

The engine may be run in multiple locations. Perhaps a collector close to where
telemetry is originating sends to a collector for a region which sends to a
collector in a cloud. Summaries are designed to be forwarded and combined.

```text
records: [
 log1 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:01am, attributes: { event_id: 1 }, summary_id: 9aa6 }
 log2 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:03am, attributes: { event_id: 1 }, summary_id: 9aa6 },
 log3 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:04am, attributes: { event_id: 1 } }
]

summaries: [
 {
  id: 9aa6,
  observed_timestamp: 6/1/2025 10:10am,
  window_type: Timestamp(00:05:00),
  window_start: 6/1/2025 10:00am,
  window_end: 6/1/2025 10:05am,
  grouping: [
   {
    key: resource.attributes[service.name],
    value: MyService
   },
   {
    key: instrumentation_scope.name,
    value: CategoryNameB
   },
   {
    key: attributes[event_id],
    value: 1
   }
  ],
  included_count: 2,
  total_count: 4
 },
]
```

In this case the engine received two logs which were already summarized and a
new log.

The output will look something like this:

```text
included_records: [
 log1 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:01am, attributes: { event_id: 1 }, summary_id: 9aa6 }
 log3 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:04am, attributes: { event_id: 1 }, summary_id: 9aa6 }
]

dropped_records: [
 log2 [ resourceMyService, scopeB ] { timestamp: 6/1/2025 10:03am, attributes: { event_id: 1 }, summary_id: 9aa6 },
]

summaries: [
 {
  id: 9aa6,
  observed_timestamp: 6/1/2025 10:10am,
  window_type: Timestamp(00:05:00),
  window_start: 6/1/2025 10:00am,
  window_end: 6/1/2025 10:05am,
  grouping: [
   {
    key: resource.attributes[service.name],
    value: MyService
   },
   {
    key: instrumentation_scope.name,
    value: CategoryNameB
   },
   {
    key: attributes[event_id],
    value: 1
   }
  ],
  included_count: 2,
  total_count: 5
 },
]
```

What happened here is the new log was combined with the external summary. Only a
single summary is returned with a new `total_count`.

### Unknowns

There isn't really a place in the OpenTelemetry data model to store these
summaries. The engine does not seek to solve this. Because the engine really is
agnostic to the fact that it is running over OpenTelemetry data. It just returns
the data and lets the caller decide what to do.

In something like the OpenTelemetry Collector summaries _could_ just be sent as
logs with a well-known scope.

Perhaps they could be converted to metrics.

Long-term perhaps there should be a spot for them in the data model. This is
something which needs to be discussed.
