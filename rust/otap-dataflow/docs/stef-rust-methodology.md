# STEF Rust Compatibility Methodology

This document applies the
[oracle-based reimplementation methodology](./oracle-based-reimplementation.md)
to the Rust STEF receiver, exporter, encoder, and decoder in the OTAP Dataflow
Engine.

It is intentionally STEF-specific. The general method is defined once in the
shared methodology document; this document records the STEF sources, design
constraints, current status, and remaining validation plan.

## STEF-Specific Method

The STEF Rust implementation follows the general **spec-constrained oracle
method**.

For STEF, the normative sources are the STEF wire specification and SDL. The
behavioral oracle is the Go STEF implementation and its OpenTelemetry Collector
integration. The Rust implementation must remain explainable by the spec while
also interoperating with Go-generated and Go-consumed STEF streams.

The STEF-specific compatibility contract is:

- Rust-encoded STEF must be accepted by the Go reader, Go receiver, and Go
  Collector integration for the declared supported scope.
- Go-encoded STEF must be accepted by the Rust decoder and receiver for the
  declared supported scope.
- The decoded telemetry must be semantically equivalent after conversion to the
  relevant OpenTelemetry or OTAP representation.
- Byte-for-byte equality is required only for stable protocol elements such as
  fixed headers, schema identity, first-message rules, and gRPC message
  sequencing.
- Data frames may differ byte-for-byte when both streams decode to equivalent
  telemetry and remain accepted by the reference implementation.
- Unsupported STEF schemas, OpenTelemetry data types, and protocol options must
  fail explicitly rather than silently dropping data.

## STEF Sources of Truth

- **Normative wire source:** the STEF specification, including stream headers,
  VarHeader, data frames, codecs, dictionaries, and STEF/gRPC protocol
  behavior.
- **Normative schema source:** the STEF SDL and the OpenTelemetry STEF schema
  used by the Go implementation.
- **Behavioral oracle:** the Go OTel STEF implementation, the otelcol STEF
  receiver/exporter integration, Go tests, and Go fuzz corpora.
- **Local architecture source:** the OTAP Dataflow Engine's OTAP Arrow records,
  OTLP passthrough model, receiver/exporter node model, backpressure,
  ack/nack, telemetry, and memory behavior.

## Current Rust Scope

The current Rust implementation targets the high-performance metrics path used
by the OTAP Dataflow Engine:

- STEF metrics receiver over gRPC
- STEF metrics exporter over gRPC
- direct STEF-to-OTAP decoding
- direct OTAP-to-STEF encoding
- direct OTLP protobuf bytes view to STEF encoding
- supported metric data: gauge and sum number data points
- supported attribute values: the current subset implemented by the Rust STEF
  codec

Full functional parity with every STEF schema feature, every OpenTelemetry
metric data type, logs, and traces is intentionally outside the first completed
scope. Those areas are part of the remaining compatibility plan.

## Already Applied

- [x] Identified the Go STEF implementation as the executable compatibility
  oracle for OpenTelemetry STEF behavior.
- [x] Reviewed the STEF wire model: stream header, VarHeader, data frames,
  columns, modified-field masks, codecs, and dictionaries.
- [x] Reviewed the STEF/gRPC protocol model: first message, `stef_bytes`,
  `is_end_of_chunk`, chunk assembly, and stream-level behavior.
- [x] Reviewed the OTel STEF schema shape used by the Go-generated OTel STEF
  code.
- [x] Defined the first Rust compatibility scope around metrics, gauge and sum
  number data points, and OTAP-native dataflow.
- [x] Implemented a Rust STEF metrics encoder from OTLP metric views.
- [x] Implemented a Rust STEF metrics encoder from direct OTAP Arrow records.
- [x] Implemented a Rust STEF metrics decoder directly into OTAP Arrow records.
- [x] Implemented the Rust STEF gRPC receiver node.
- [x] Implemented the Rust STEF gRPC exporter node.
- [x] Validated Go-to-Rust decoding with Go-generated STEF fixtures.
- [x] Validated Rust-to-Go decoding with Rust-generated STEF fixtures.
- [x] Validated e2e interoperability between a Go Collector and the Rust OTAP
  Dataflow Engine over STEF.
- [x] Compared Go and Rust codec performance on direct encode/decode paths.
- [x] Optimized the Rust STEF decoder around direct OTAP builders and attribute
  reuse.
- [x] Optimized the Rust STEF encoder around OTAP direct encoding and OTLP view
  traversal.
- [x] Compared conversion scenarios at 100k and 200k metric points:
  `STEF -> STEF`, `OTLP -> STEF`, `OTAP -> STEF`, `STEF -> OTLP`, and
  `STEF -> OTAP`.
- [x] Documented the Rust STEF receiver and exporter configuration, current
  limits, and future expansion areas.
- [x] Removed the initial legacy implementation and kept the fastest committed
  Rust path.
- [x] Refactored the Rust STEF codec into smaller modules with explicit module
  responsibilities.
- [x] Ran the full Rust workspace validation with `cargo xtask check`.

## Remaining Plan

### 1. Pin Sources and Build a Traceability Matrix

- [ ] Pin the exact STEF specification version or commit used for validation.
- [ ] Pin the exact SDL document version used for schema interpretation.
- [ ] Pin the exact Go STEF modules and otelcol reference commits used as
  compatibility oracles.
- [ ] Record the exact Go fuzz corpus commit used for Rust seed corpora.
- [ ] Build a STEF traceability matrix from spec sections to Rust modules and
  tests.
- [ ] Build an OTel STEF schema coverage matrix:
  implemented, partially implemented, explicitly unsupported, and not yet
  validated.

### 2. Expand Golden Corpus Compatibility Tests

- [ ] Generate canonical Go STEF fixtures for supported metric shapes.
- [ ] Generate canonical Rust STEF fixtures for the same metric shapes.
- [ ] Decode every Go fixture with Rust and assert semantic OTAP equivalence.
- [ ] Decode every Rust fixture with Go and assert semantic OTLP equivalence.
- [ ] Include fixtures with high and low string cardinality.
- [ ] Include fixtures with dictionary reuse, dictionary reset, and frame
  restart cases.
- [ ] Include fixtures that split logical chunks across many gRPC messages.
- [ ] Include malformed fixtures and assert compatible rejection behavior.
- [ ] Include empty streams, empty frames, empty metric batches, and unsupported
  metric data types.

### 3. Add Differential Roundtrip Tests

- [ ] Test `Go encode -> Rust decode -> Rust encode -> Go decode`.
- [ ] Test `Rust encode -> Go decode -> Go encode -> Rust decode`.
- [ ] Assert semantic equivalence after each full roundtrip.
- [ ] Assert stable protocol invariants: header, root struct name, schema
  identity, first-message rules, and end-of-chunk rules.
- [ ] Assert unsupported inputs fail explicitly and do not silently drop data.
- [ ] Run the differential tests for both OTLP bytes input and OTAP Arrow input
  when the path exists in Rust.

### 4. Add Fuzzing

- [ ] Import the Go `FuzzMetricsReader` corpus as seeds for Rust decoder
  fuzzing.
- [ ] Add `cargo-fuzz` targets for STEF bytes to Rust OTAP decode.
- [ ] Add a fuzz target for STEF/gRPC chunk assembly.
- [ ] Add a fuzz target for VarHeader and schema parsing.
- [ ] Add mutation strategies that preserve valid stream structure while
  varying dictionaries, frame boundaries, modified masks, strings, and numeric
  codecs.
- [ ] Add negative fuzzing for malformed frames, invalid varints, truncated
  columns, invalid UTF-8, unknown flags, and unsupported schemas.
- [ ] Add differential fuzzing where minimized Rust inputs are replayed through
  the Go decoder when practical.
- [ ] Add CI or nightly jobs that run the fuzz corpus in bounded time.
- [ ] Store every discovered crash or semantic mismatch as a regression
  fixture.

### 5. Validate STEF/gRPC Protocol Compatibility

- [ ] Test Go exporter to Rust receiver over gRPC.
- [ ] Test Rust exporter to Go receiver over gRPC.
- [ ] Test Go Collector to Rust OTAP Dataflow Engine over STEF.
- [ ] Test Rust OTAP Dataflow Engine to Go Collector over STEF.
- [ ] Validate first-message schema negotiation and root struct handling.
- [ ] Validate arbitrary `stef_bytes` fragmentation and reassembly.
- [ ] Validate `is_end_of_chunk` handling with empty and non-empty final
  messages.
- [ ] Validate dictionary limit advertisements and behavior at limit
  boundaries.
- [ ] Validate receiver behavior for unexpected first messages, unknown roots,
  malformed chunks, and unsupported compression.
- [ ] Validate shutdown, cancellation, backpressure, and nack/ack behavior in
  the OTAP runtime.
- [ ] Validate configured message-size limits and oversized chunk rejection.

### 6. Complete Functional Coverage

- [ ] Extend metrics support toward Go parity for histograms.
- [ ] Extend metrics support toward Go parity for exponential histograms.
- [ ] Extend metrics support toward Go parity for summaries.
- [ ] Extend attribute value support to cover the OTel STEF schema.
- [ ] Decide whether logs and spans are in scope for this Rust milestone.
- [ ] If logs are in scope, repeat the full oracle method for logs.
- [ ] If spans are in scope, repeat the full oracle method for spans.
- [ ] Document every intentionally unsupported schema feature and its failure
  mode.

### 7. Stabilize Performance Confidence

- [ ] Convert the temporary STEF benchmark harness into committed benchmarks.
- [ ] Add focused benchmarks for:
  `STEF -> STEF`, `OTLP -> STEF`, `OTAP -> STEF`, `STEF -> OTLP`, and
  `STEF -> OTAP`.
- [ ] Run each benchmark at 100k and 200k metric points.
- [ ] Add benchmark variants for batch size, chunk size, attribute cardinality,
  metric cardinality, and dictionary reuse.
- [ ] Record allocations per point and bytes allocated per point.
- [ ] Profile CPU with `perf` for each hot scenario.
- [ ] Track p50, p95, and p99 latency for e2e gRPC scenarios.
- [ ] Define performance acceptance targets against Go and against current Rust
  baselines.
- [ ] Add a reproducible benchmark report format with fixture metadata,
  machine metadata, commit SHAs, and command lines.
- [ ] Decide which performance checks belong in CI and which remain manual or
  scheduled.

### 8. Align with OTAP Dataflow Engine Semantics

- [ ] Keep the Rust hot path aligned with OTAP Arrow records rather than Prost
  objects.
- [ ] Preserve the OTLP passthrough model where it is the correct engine-level
  representation.
- [ ] Avoid unnecessary OTLP protobuf materialization in STEF receiver paths.
- [ ] Avoid unnecessary OTAP materialization in STEF exporter paths when OTLP
  bytes are already available and supported.
- [ ] Validate transport-optimized ID handling before and after STEF
  conversion.
- [ ] Validate payload preservation for ack/nack and retry paths.
- [ ] Validate memory limiter behavior under sustained STEF input.
- [ ] Add STEF receiver/exporter telemetry for accepted records, rejected
  records, bytes, chunks, decode errors, encode errors, and unsupported input.
- [ ] Document configuration behavior in the core-node docs and examples.

### 9. Automate Compatibility in CI

- [ ] Add a Go compatibility test helper that can be run from Rust CI.
- [ ] Cache or generate golden fixtures deterministically.
- [ ] Run Go-to-Rust and Rust-to-Go compatibility tests in CI.
- [ ] Run corpus regression tests in CI.
- [ ] Run bounded fuzz corpus replay in CI.
- [ ] Run full fuzzing in nightly or scheduled jobs.
- [ ] Add a clear failure report that identifies whether a mismatch is spec,
  schema, Go oracle, Rust codec, or gRPC protocol related.

### 10. Finalize Documentation and Release Criteria

- [ ] Publish the STEF compatibility matrix.
- [ ] Publish the STEF performance matrix.
- [ ] Publish the supported STEF feature matrix.
- [ ] Publish the known limitations and unsupported input behavior.
- [ ] Document the exact STEF oracle methodology in the receiver and exporter
  docs.
- [ ] Document how to regenerate fixtures and rerun compatibility tests.
- [ ] Define release criteria for declaring STEF metrics support compatible.
- [ ] Define separate release criteria for future logs and spans support.

## STEF Release Criteria

STEF metrics support can be considered compatible for the declared scope when:

- implemented wire behavior is traceable to the STEF specification
- implemented schema behavior is traceable to the SDL and OTel STEF schema
- Go and Rust accept each other's supported STEF output
- Go and Rust reject malformed or unsupported STEF input in compatible ways
- fuzz-generated failures are captured as permanent regression tests
- performance is measured with reproducible fixtures and stable command lines
- the hot path preserves OTAP Dataflow Engine semantics and avoids unnecessary
  format conversion
- receiver/exporter behavior is validated both as local codecs and as real
  gRPC nodes in end-to-end pipelines

## References

- [Oracle-based reimplementation methodology](./oracle-based-reimplementation.md)
- [STEF specification][stef-spec]
- [STEF SDL][stef-sdl]
- [Splunk STEF otelcol reference integration][stef-otelcol]
- [Splunk Go OTel STEF implementation and schema][stef-go-otel]
- [Go OTel STEF fuzz corpus][stef-fuzz]

[stef-spec]: https://github.com/splunk/stef/blob/main/stef-spec/specification.md
[stef-sdl]: https://www.stefdata.net/sdl.html
[stef-otelcol]: https://github.com/splunk/stef/tree/main/otelcol
[stef-go-otel]: https://github.com/splunk/stef/tree/main/go/otel
[stef-fuzz]: https://github.com/splunk/stef/tree/beadbb9b0a9143fc08aacdb8dda613bd451d143a/go/otel/otelstef/testdata/fuzz
