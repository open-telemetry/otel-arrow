# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

- Remove two deprecated fields, both concurrent batch processor `max_in_flight_bytes`
  and otelarrow receiver `memory_limit` fields have corresponding `_mib` field names
  for consistency.

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
