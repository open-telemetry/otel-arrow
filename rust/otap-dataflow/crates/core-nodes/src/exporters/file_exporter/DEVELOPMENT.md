# File Exporter Development Note

## Capability Scope

The experimental `file` exporter provides bounded local capture and replay for
OTAP logs, metrics, and traces. It accepts OTLP protobuf bytes or OTAP Arrow
records and writes one newline-delimited OTLP JSON object per non-empty pdata
batch. One component owns at most three signal-specific file handles and one
bounded reusable encoding buffer.

## Reference Evidence

The design reviewed the OpenTelemetry Collector contrib `fileexporter` at
release `v0.157.0` and commit
`21196c805ba7091d0928434ec9ca145ed0386cab`, plus the OpenTelemetry Protocol
File Exporter specification. The review covered configuration, multi-signal
formatting, writer lifecycle, buffering, compression, grouping, rotation, and
reported startup, shutdown, cardinality, containment, and CPU issues.

## Finding Classifications

- Preserve: multi-signal OTLP JSON, one object per batch, one signal per file,
  optional directory creation, and composition with pipeline retry behavior.
- Improve: append by default, explicit durability, bounded crash-tail repair,
  lazy per-signal readiness probes, exclusive runtime paths, and deterministic
  writer ownership.
- Simplify: one frame in flight, one reusable buffer, and no flush task, LRU,
  or exporter-owned queue.
- Compose: retry and persistent buffering remain responsibilities of the retry
  and durable-buffer processors.
- Avoid or reject: attribute-derived paths, arbitrary encoders, per-message
  compression, unbounded grouped writers, and mixed-signal files.
- Investigate later: rotation, retention, standard file-level compression, and
  framed protobuf paired with a reader.

## OTAP Architecture

The exporter is a local `Exporter<OtapPdata>` registered as
`urn:otel:exporter:file`. Existing backend-agnostic pdata views feed the shared
unframed OTLP JSON writers in `otap_df_pdata`; an exporter-local adapter applies
the frame bound and newline. No protobuf object is materialized for either input
representation. The local run loop serializes all signal writes and relies on
the bounded exporter inbox for backpressure.

Paths require signal, core, and deployment-generation tokens. This matches the
thread-per-core share-nothing model and keeps rolling generations independent.
The run loop exclusively owns file handles and path leases, so shutdown cannot
race a background writer.

## Intentional Behavior Changes

Compared with the Go reference, append is the default, every path visibly owns
one core and generation, dynamic grouping is absent, and readiness is proved
per signal without creating unused signal files. ACK durability is explicit.
Failures use the engine ACK/NACK path instead of an exporter-local sending
queue.

## Unsupported Behavior

Profiles, rotation, retention, compression, non-JSON encodings, templates,
record flattening, telemetry-derived paths, cross-process leases, and
exactly-once semantics are outside the initial scope.

## Validation Status

Unit and component tests cover typed defaults and invalid relationships, path
rendering and normalized collisions, exact frame bounds, representative OTLP
JSON for all three signals, append preservation and bounded tail repair,
directory creation, path leases, lazy empty-pdata behavior, and alternating
multi-signal file output. The component-focused formatting, lint, and test
checks pass.

Collector replay fixtures, fault-injected short writes and rollback failures,
fuzzing, and dedicated throughput/allocation benchmarks remain useful follow-up
coverage before promoting the component beyond experimental stability.
