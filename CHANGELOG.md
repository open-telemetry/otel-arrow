# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

- Update to OTel-Collector v0.105.0, which includes the OTel-Arrow components.
- Remove the primary exporter/receiver components, update references and documentation. [#230](https://github.com/open-telemetry/otel-arrow/pull/230)
- Update to Arrow v17. [#231](https://github.com/open-telemetry/otel-arrow/pull/231)

## [0.24.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.24.0) - 2024-06-05

- Jitter is applied to once per process, not once per stream. [#199](https://github.com/open-telemetry/otel-arrow/pull/199)
- Network statistics tracing instrumentation simplified. [#201](https://github.com/open-telemetry/otel-arrow/pull/201)
- Protocol includes use of more gRPC codes. [#202](https://github.com/open-telemetry/otel-arrow/pull/202)
- Receiver concurrency bugfix. [#205](https://github.com/open-telemetry/otel-arrow/pull/205)
- Concurrent batch processor size==0 bugfix. [#208](https://github.com/open-telemetry/otel-arrow/pull/208)
- New integration testing. [#210](https://github.com/open-telemetry/otel-arrow/pull/210)
- Use gRPC Status codes in the Arrow exporter. [#211](https://github.com/open-telemetry/otel-arrow/pull/211)
- Fix stream-shutdown race in Arrow receiver. [#212](https://github.com/open-telemetry/otel-arrow/pull/212)
- Avoid work for already-canceled requests. [#213](https://github.com/open-telemetry/otel-arrow/pull/213)
- Call IPCReader.Err() after reader loop. [#215](https://github.com/open-telemetry/otel-arrow/pull/215)
- Update to Arrow-Go v16.1.0. [#218](https://github.com/open-telemetry/otel-arrow/pull/218)
- Update to OpenTelemetry Collector v0.102.x. [#219](https://github.com/open-telemetry/otel-arrow/pull/219)

## [0.23.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.23.0) - 2024-05-09

- Remove the OTel-Arrow exporter FIFO prioritizer.  Let "leastloaded" imply least-loaded
  over all streams and use this behavior by default. [#186](https://github.com/open-telemetry/otel-arrow/pull/186)

- Fix concurrentbatchproccessor bug to correctly propagate metadataKeys for multi shard batching. [#184](https://github.com/open-telemetry/otel-arrow/pull/184)

- Refactor otelarrowreceiver to do stream.Recv, request processing, and stream.Send in separate goroutines. [#181](https://github.com/open-telemetry/otel-arrow/pull/181)

- Add a semaphore package to limit bytes admitted and total number of waiters. [#174](https://github.com/open-telemetry/otel-arrow/pull/174)

## [0.22.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.22.0) - 2024-04-16

- Add load prioritization mechanism and "leastloaded" policy. [#178](https://github.com/open-telemetry/otel-arrow/pull/178)

## [0.21.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.21.0) - 2024-04-10

- Bump versions to latest 0.98.0 1.25.0 [#175](https://github.com/open-telemetry/otel-arrow/pull/175)

- Update examples; add "shutdown", remove generator [#171](https://github.com/open-telemetry/otel-arrow/pull/171)

- Improve logging at Arrow stream shutdown; avoid the explicit Canceled message at stream lifetime [#170](https://github.com/open-telemetry/otel-arrow/pull/170)

- More lint from opentelemetry-collector-contrib PR 32015 [#168](https://github.com/open-telemetry/otel-arrow/pull/168)

- Lint fixes for OTel-Arrow receiver in OTel-Collector-Contrib [#167](https://github.com/open-telemetry/otel-arrow/pull/167)

- Enable ci/cd build and test [#166](https://github.com/open-telemetry/otel-arrow/pull/166)


## [0.20.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.20.0) - 2024-03-27

- Lint fixes, renames, and validation update to follow conventions of otel-collector-contrib repository. [#163](https://github.com/open-telemetry/otel-arrow/pull/163)

- Update Otel Collector core dependency from `v0.96.0` to `v0.97.0` [#164](https://github.com/open-telemetry/otel-arrow/pull/164)

## [0.19.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.19.0) - 2024-03-26

- Fix arrow log encoder panic caused by empty attribute values. [#159](https://github.com/open-telemetry/otel-arrow/pull/159)

- Remove deprecated obsreporttest package. [#161](https://github.com/open-telemetry/otel-arrow/pull/161)

## [0.18.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.18.0) - 2024-03-06

- Update Otel Collector core dependency from `v0.94.1` to `v0.96.0`. [#155](https://github.com/open-telemetry/otel-arrow/pull/155)

## [0.17.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.17.0) - 2024-02-13

- Otel-Arrow receiver cleanups to help migrate component to `opentelemetry-collector-contrib` repo. [#146](https://github.com/open-telemetry/otel-arrow/pull/146)

- Add span attributes for uncompressed request size as part of netstats package. [#149](https://github.com/open-telemetry/otel-arrow/pull/149)

- Add metrics to record Otel-Arrow receiver in-flight bytes and items. [#150](https://github.com/open-telemetry/otel-arrow/pull/150)

- Cleanup deprecated telemetry object. [#151](https://github.com/open-telemetry/otel-arrow/pull/151)

- Update Otel Collector core dependency from `v0.92.0` to `v0.94.1`. [#153](https://github.com/open-telemetry/otel-arrow/pull/153)

## [0.16.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.16.0) - 2024-01-19

- Use go-1.20 in CI/CD and go.mods. [#144](https://github.com/open-telemetry/otel-arrow/pull/144)

## [0.15.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.15.0) - 2024-01-17

- Remove unused `mixed_signals` feature and associated protocol elements, regenerate mocks using go.uber.org/mock@v0.4.0, repair CI/CD pipeline. [#135](https://github.com/open-telemetry/otel-arrow/pull/135), [#136](https://github.com/open-telemetry/otel-arrow/pull/136).
- Add tracing support to the OTel-Arrow exporter and receiver. [#137](https://github.com/open-telemetry/otel-arrow/pull/139)

## [0.14.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.14.0) - 2024-01-11

- Remove two deprecated fields, both concurrent batch processor `max_in_flight_bytes`
  and otelarrow receiver `memory_limit` fields have corresponding `_mib` field names
  for consistency.
- OTel-Arrow exporter: Do not treat PartialSuccess as errors (see https://github.com/open-telemetry/opentelemetry-collector/issues/9243). [#130](https://github.com/open-telemetry/otel-arrow/pull/130)
- Use OTel Collector v0.92.0. [#131](https://github.com/open-telemetry/otel-arrow/pull/131)
- Use Apache Arrow v14.0.2 dependencies. [#132](https://github.com/open-telemetry/otel-arrow/pull/132)

## [0.13.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.13.0) - 2023-12-20

- Concurrent batch processor: Fail fast for large batch sizes. [#126](https://github.com/open-telemetry/otel-arrow/pull/126)
- Add the core OTLP receiver to the otelarrowcol build, for its HTTP support. [#125](https://github.com/open-telemetry/otel-arrow/pull/125)
- Let span context propagate through the concurrent batch processor. [#123](https://github.com/open-telemetry/otel-arrow/pull/123)
- Lint: use `max_in_flight_size_mib` as the configuration for maximum in-flight-bytes. [#121](https://github.com/open-telemetry/otel-arrow/pull/121)
- Remove HTTP support from the OTel-Arrwo receiver. [#120](https://github.com/open-telemetry/otel-arrow/pull/120)
- Remove the stream_unique metric attribute. [#119](https://github.com/open-telemetry/otel-arrow/pull/119)

## [0.12.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.12.0) - 2023-12-04

- Update to OpenTelemetry Collector v0.90.1 dependencies. (#116)
- Bug-fix for `_in_flight_bytes` metric. (#115)

## [0.11.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.11.0) - 2023-11-28

- "concurrentbatchprocessor" supports two new metrics, names ending
  `_in_flight_bytes` and `_batch_send_latency`. (#111, #112)

## [0.10.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.10.0) - 2023-11-17

- "concurrentbatchprocessor" component is ready for production testing. (#93)

## [0.9.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.9.0) - 2023-11-15

- No changes, testing release process.

## [0.8.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.8.0) - 2023-11-15

- No changes, testing release process.

## [0.7.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.7.0) - 2023-11-10

- New "concurrentbatchprocessor" component under development (#71, #79, #90)
- Update examples, add Docker build (#92)
- Collector v0.88.0 dependency updates (#76)
- Network statistics correctness (#70)
- Zstd compression level is configurable (#81)
- Instrumentation on schema changes (#77)
- Compression rate degradation fix (#82)

## [0.6.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.6.0) - 2023-10-17

- Collector v0.87.0 dependency updates, CI/minor memory optimization (#65). 

## [0.5.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.5.0)

- Memory leak fixes, new instrumentation.  See #47, #52, #53, #54, #55, #56, and #57.

## [0.4.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.4.0) - 2023-09-96

- No code changes from v0.3.0, working on go module relationship issues.

## [0.3.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.3.0) - 2023-08-31

- This release includes a BREAKING CHANGE 🛑 🛑 🛑 🛑 🛑, see #27.  Also includes #24, #29, #30, #36.

## [0.2.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.2.0) - 2023-08-25

- Includes fixes for stream lifetime, payload compression option, README updates: #13, #20, #21, #22, and #23.

## [0.1.0](https://github.com/open-telemetry/otel-arrow/releases/tag/v0.1.0) - 2023-08-17

- This is the first tagged release of the OTel Arrow project.
