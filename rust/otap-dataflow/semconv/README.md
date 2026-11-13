# OTAP Dataflow internal semantic conventions

This directory holds the [Weaver][weaver]-format semantic-conventions
registry describing the internal events, metrics, and attributes
emitted by the OTAP Dataflow engine (`df_engine`) for its own
instrumentation.

CI uses this registry to validate that what the engine actually emits
at runtime matches the conventions declared here.

Tracking issue:
<https://github.com/open-telemetry/otel-arrow/issues/1613>

## Layout

```text
semconv/
  manifest.yaml      # registry root + upstream semconv dependency
  groups/
    attributes.metrics.yaml
    event.<name>.yaml
    metric.engine.yaml
  triggers/
    *.sh             # optional: scripts to run before validation
                     # (only needed for events that can't be exercised
                     # purely by the engine config)
```

Each event is a Weaver `group` with `type: event`; the group's `name:`
field is the OpenTelemetry `event.name` value passed as the first
argument to `otel_info!` / `otel_warn!` / `otel_debug!` /
`otel_error!` / `otel_event!` (see
[`crates/telemetry/src/internal_events.rs`][events]).

Each metric is a Weaver `group` with `type: metric` whose `metric_name`
is `<metric_set_name>.<field_name>`, synchronized with Rust `#[metric_set]`
and `#[attribute_set]` declarations via `cargo xtask telemetry-schema`.

## Adding or updating a metric

1. Declare or update the `#[metric_set]` and `#[attribute_set]` structs
   in the appropriate crate.
2. Run `cargo xtask telemetry-schema --generate-semconv core` (or `all`)
   to update `semconv/groups/attributes.metrics.yaml` and
   `semconv/groups/metric.*.yaml` and refresh generated Markdown tables.
3. Verify with `cargo xtask telemetry-schema --check` and
   `weaver registry check -r rust/otap-dataflow/semconv`.

## Adding a new event

1. Pick a stable, dot-segmented name (`component.action[.detail]`).
2. Add a new YAML file under `groups/` with a single `type: event`
   group; reuse attributes from upstream OTel semconv (`error.type`,
   `server.address`, ...) where they exist, and add project-specific
   attributes under the `otap.*` namespace otherwise.
3. Emit the event with the matching name from the appropriate
   `otel_*!` macro.
4. Exercise the event in CI. Many events fire from the engine
   config alone (e.g. startup events fire when a receiver is
   configured; failure events fire when traffic is routed at an
   unreachable endpoint). Adjust `configs/internal-events-otlp.yaml`
   as needed. For events that need active network traffic or other
   runtime stimulus, drop a script in `triggers/`; everything in
   that directory is run before validation. The live-check job
   fails if any declared event receives zero samples.
5. If neither the engine config nor a trigger can drive the event
   (for example, it fires only when an internal channel is closed),
   cover it with unit tests and add an `annotations.live_check.exempt`
   entry to its group that says why. The live-check job then allows
   zero samples for it, but still validates any samples it receives.

[weaver]: https://github.com/open-telemetry/weaver
[events]: ../crates/telemetry/src/internal_events.rs
