# Traffic Generator -- SUT Saturation Benchmarks

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `receiver:traffic_generator` (`urn:otel:receiver:traffic_generator`)
- Feature gate: `dev-tools`
- Stability: Experimental

## Overview

The traffic generator receiver emits synthetic, semantic-convention-derived,
or codec-generated telemetry for tests, demos, and benchmark scenarios. It is
compiled only when the `dev-tools` feature is enabled.

## Getting Started

Generate synthetic logs with pregenerated payloads:

```yaml
type: receiver:traffic_generator
config:
  data_source: synthetic
  generation_strategy: pre_generated
  traffic_config:
    production_mode: open
    max_batch_size: 1000
    metric_weight: 0
    trace_weight: 0
    log_weight: 1
    log_body_size_bytes: 1024
```

## Configuration

```yaml
type: receiver:traffic_generator
config:
  # Source of generated telemetry shape (default: semantic_conventions).
  data_source: synthetic

  # Semantic convention registry path when data_source is semantic_conventions.
  registry_path: ./semantic-conventions

  # Batch generation strategy: "fresh" or "pre_generated" (default: fresh).
  generation_strategy: pre_generated

  # Enables generated pdata ACK/NACK interests and waits for outstanding
  # completions during graceful drain (default: false).
  enable_ack_nack: false

  # Resource attributes rotated across batches (default: []).
  resource_attributes:
    - service.name: load-generator

  # Fixed or generated transport headers (default: {}).
  transport_headers:
    x-tenant-id: demo

  # Signal rate, batch size, and signal weights (required).
  traffic_config:
    production_mode: open
    signals_per_second: null
    max_signal_count: null
    max_batch_size: 1000
    metric_weight: 0
    trace_weight: 0
    log_weight: 1
    log_body_size_bytes: 1024
```

Transport-header names preserve their configured spelling. Names that differ
only by case remain distinct stored context entries.

### Codec-owned generation

Set `data_source: codec` to generate the selected codec's encoded payloads
instead of constructing OTLP signals. The codec owns format-specific profiles,
framing, and allocation limits; the receiver still owns signal weights, rate,
batch size, total signal limits, and fresh/pre-generated strategies.

```text
trafficgen pacing -> codec generator -> encoded payload -> downstream nodes
                                                           |
                                            decode when native records are needed
```

Configuration template: replace the encoding and profile placeholders with
values advertised by a linked codec that supports generation.

```yaml
type: receiver:traffic_generator
config:
  data_source: codec
  generation_strategy: pre_generated
  codec:
    encoding: your-encoding
    profile: your-profile
    max_batch_bytes: 2097152
  traffic_config:
    signals_per_second: 1000
    max_batch_size: 100
    log_weight: 1
```

`codec.encoding` selects a registered encoding, honoring pipeline `codecs.bindings`,
and `codec.profile` selects one of that implementation's advertised generation
profiles. Codecs opt in to generation separately from ordinary encoding support.
Structural settings are validated at config load time. Unknown encodings,
unavailable profiles, unsupported signals, and codecs without a generator fail
producer setup, after pipeline bindings are available and before traffic is emitted.

The receiver seeds each codec generator internally.
By default, each generated batch receives the current time. Fresh
mode generates new contents and timestamps; pre-generated mode caches a run of
batches at startup and replays their bytes, including timestamps, unchanged.
Set `codec.timestamp_unix_nano` to a nonnegative integer for fixed timestamps
in either strategy. Timestamp precision and interpretation depend on the codec.

`codec.max_batch_bytes` defaults to 2 MiB. The codec validates its own limits
and worst-case allocation size at startup; the shared generation wrapper also
checks each emitted payload's encoding, signal, item count, and encoded size.
Reduce `traffic_config.max_batch_size` if a batch cannot fit the byte limit.
Synthetic-only body-size, attribute, and trace-context settings are rejected
with codec generation; use codec profiles to define generated contents.

Generation injects encoded pdata directly, without a client or network
transport. Payloads retain their encoded representation until a downstream node
requires native records, exercising the real decoder rather than embedding
format-looking text in OTLP bodies. This does not test socket handling or
transport framing.

See the [codec generation contract](../../../../pdata-codec/README.md#synthetic-generation-for-trafficgen)
for implementing profiles.

## Purpose

Verify that a **single traffic-gen sender core can saturate a single
`df_engine` (SUT) core** over OTLP gRPC, so that performance tests
don't need multiple load-generator cores.

```text
  sender (1 core)           SUT (1 core)            backend (1 core)
  traffic-gen -> OTLP-export --> OTLP-recv -> OTLP-export --> OTLP-recv -> noop
       :8080                     :8081                      :8082
```

## Configs (`bench/`)

| File                        | Role        | Pipeline                           |
|-----------------------------|-------------|------------------------------------|
| `sender-static-fresh.yaml`  | Load gen    | traffic-gen(fresh) -> OTLP export  |
| `sender-static-pregen.yaml` | Load gen    | traffic-gen(pregen) -> OTLP export |
| `sut-otlp-forward.yaml`     | SUT         | OTLP recv -> OTLP export           |
| `backend-noop.yaml`         | Backend     | OTLP recv -> noop                  |

## Quick start

```bash
cd rust/otap-dataflow
cargo build --release
bash crates/dev-nodes/src/receivers/traffic_generator/bench/bench.sh
```

---

## Results

Apple M3 Pro (14 cores, 24 GB), 1 pipeline core per process, release
build, logs only. Averaged over 4 runs.

| Config              | Throughput       | Sender core | SUT core     |
|---------------------|------------------|-------------|--------------|
| static/fresh 1KB    | ~480 K logs/sec  | ~82 %       | **~97 %**    |
| static/pregen 1KB   | ~500 K logs/sec  | ~66 %       | **~95 %**    |

---

## Conclusion

**Use `static` + `pre_generated` + `log_body_size_bytes: 1024`.**

This configuration saturates the SUT to ~95 % while consuming only
~66 % of a single sender core. `pre_generated` uses ~16 % less sender
CPU than `fresh` (~66 % vs ~82 %) at similar throughput, leaving more
headroom. One load-gen core is sufficient!

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `receiver.traffic_generator`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.traffic_generator.smooth.runs.terminations` | `{run}` | `outcome` | Number of smooth-mode production runs terminated, dimensionalized by outcome (`on_time`, `late`, `cancelled`). |
| `receiver.traffic_generator.smooth.payload.sends.attempts` | `{attempt}` | `attempt`, `outcome` | Number of smooth-mode payload send attempts, dimensionalized by kind of attempt (`initial`, `retry`) and outcome (`success`, `refused`, `failure`). |
| `receiver.traffic_generator.smooth.runs_started` | `{run}` | | Number of smooth-mode production runs started. |
| `receiver.traffic_generator.smooth.schedule_deadline_misses` | `{miss}` | | Number of times smooth mode detects a missed scheduling boundary. |
| `receiver.traffic_generator.completion.pending` | `{batch}` | | Number of subscribed batches waiting for Ack/Nack completion. |
| `receiver.traffic_generator.completion.drain_deadline_forced` | `{drain}` | | Number of drains forced to finish with unresolved batches at the deadline. |
| `receiver.traffic_generator.smooth.late_remaining_batches` | `{batch}` | | Number of batches remaining when smooth mode detects that a run is late. |
| `receiver.traffic_generator.smooth.late_remaining_items` | `{item}` | | Number of signal items remaining when smooth mode detects that a run is late. |
| `receiver.traffic_generator.smooth.run_batches` | `{batch}` | | Smooth-mode configured batches per one-second run. |
| `receiver.traffic_generator.smooth.batch_interval` | `ns` | | Smooth-mode configured interval between batches. |
| `receiver.traffic_generator.smooth.batch_tick_lateness_duration` | `s` | | Lateness of smooth-mode batch ticks relative to their scheduled instant. |
| `receiver.traffic_generator.smooth.payload_generate_duration` | `s` | | Wall-clock time spent generating or cloning one smooth-mode payload. |
| `receiver.traffic_generator.smooth.payload_send_duration` | `s` | | Wall-clock time spent sending one smooth-mode payload into the downstream channel. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `traffic_generator.smooth_run_behind` | `warn` | Smooth-mode generation did not finish before the next run tick. |
| `traffic_generator.open_run_behind` | `debug` | Open-loop generation remained behind schedule. |
| `traffic_generator.drain_ingress` | `info` | Receiver ingress drain started. |
| `traffic_generator.completion_drain.deadline_reached` | `warn` | Ack/Nack drain deadline expired with unresolved batches. |
| `traffic_generator.shutdown` | `info` | Receiver shutdown completed. |
| `traffic_generator.smooth_fallback_open` | `warn` | Smooth mode fell back to open-loop behavior. |

## Limits

- This receiver is available only with the `dev-tools` feature.
- `semantic_conventions` data source requires access to the configured registry
  path at startup.
- `pre_generated` mode repeats timestamps and IDs from pregenerated batches.
- `production_mode: open` can produce as fast as the runtime allows.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Core node catalog](../../../README.md)
