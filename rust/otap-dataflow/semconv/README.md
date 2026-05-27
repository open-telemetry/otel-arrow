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
    event.<name>.yaml
  triggers/
    <name>.sh        # exercises the matching event in live-check CI
```

Each event is a Weaver `group` with `type: event`; the group's `name:`
field is the OpenTelemetry `event.name` value passed as the first
argument to `otel_info!` / `otel_warn!` / `otel_debug!` /
`otel_error!` / `otel_event!` (see
[`crates/telemetry/src/internal_events.rs`][events]).

## Adding a new event

1. Pick a stable, dot-segmented name (`component.action[.detail]`).
2. Add a new YAML file under `groups/` with a single `type: event`
   group; reuse attributes from upstream OTel semconv (`error.type`,
   `server.address`, ...) where they exist, and add project-specific
   attributes under the `otap.*` namespace otherwise.
3. Emit the event with the matching name from the appropriate
   `otel_*!` macro.
4. Add `triggers/<name>.sh` that drives df_engine to emit the event;
   the live-check job iterates this directory and fails if any
   declared event receives zero samples.

[weaver]: https://github.com/open-telemetry/weaver
[events]: ../crates/telemetry/src/internal_events.rs
