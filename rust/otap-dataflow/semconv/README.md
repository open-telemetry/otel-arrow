# OTAP Dataflow internal semantic conventions

This directory holds the [Weaver][weaver]-format semantic-conventions
registry describing the internal events, metrics, and attributes
emitted by the OTAP Dataflow engine (`df_engine`) for its own
instrumentation.

The registry is consumed by CI to validate that what the engine
actually emits at runtime matches the conventions declared here.
See the `df-engine-events-weaver-live-check-mvp` job in
`.github/workflows/rust-ci.yml`.

Tracking issue:
<https://github.com/open-telemetry/otel-arrow/issues/1613>

## Layout

```text
semconv/
  manifest.yaml      # registry root + upstream semconv dependency
  groups/
    event.tls.handshake.failed.yaml
```

Each event is a Weaver `group` with `type: event`; the group's `name:`
field is the OpenTelemetry `event.name` value passed as the first
argument to `otel_info!` / `otel_warn!` / `otel_debug!` /
`otel_error!` / `otel_event!` (see
[`crates/telemetry/src/internal_events.rs`][events]).

## Scope today (MVP)

This PR introduces **one** event, `tls.handshake.failed`, end-to-end:

- declared here as a `type: event` group,
- emitted at `crates/otap/src/tls_utils.rs` when rustls rejects a
  client handshake,
- routed through the `internal_telemetry` receiver to a
  `weaver registry live-check` listener in CI, which asserts the
  emitted event is known to this registry and produces no
  `violation`-level findings.

The remaining ~140 distinct event names emitted by df_engine will be
backfilled in follow-up PRs once the mechanism is proven green in CI.

## Adding a new event

1. Pick a stable, dot-segmented name (`component.action[.detail]`).
2. Add a new YAML file under `groups/` with a single `type: event`
   group; reuse attributes from upstream OTel semconv (`error.type`,
   `server.address`, ...) where they exist, and add project-specific
   attributes under the `otap.*` namespace otherwise.
3. Emit the event with the matching name from the appropriate
   `otel_*!` macro.
4. Open the PR -- the live-check CI job will exercise the new path.

[weaver]: https://github.com/open-telemetry/weaver
[events]: ../crates/telemetry/src/internal_events.rs
