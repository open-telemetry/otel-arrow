# Example: record and replay OpenTelemetry Protocol data

First, obtain a collector that includes the OTAP components -- see
[BUILDING](../../BUILDING.md) (for example, run `make otelarrowcol` in the
top-level directory to extract a `./bin/otelarrowcol` binary).

To execute the data recorder, with the `collector` function defined as
described in the [examples README](../README.md):

```shell
collector --config target.yaml
```

During this phase, data received is copied through a "loopback" Arrow pipeline
that encodes and decodes the data and should second, identical copies of the
inputs.

When enough data has been collected to `first.traces.json` and
`first.metrics.json`, you may stop the target collector.

Note this configuration exercises Arrow in the loopback pipeline, and if Arrow
is failing for any reason the sender will receive errors. When this is
happening, modify the forwarding exporter to disable Arrow in the recording
step, e.g.,:

```yml
exporters:
  otelarrow/forward:
    arrow:
      enabled: false
```

Then, to re-exercise the same data though an Arrow pipeline using data recorded
in the first step, run:

```shell
collector --config replay.yaml
```

Note that this example only supports traces and metrics.  Logs are not supported
in these configurations.
