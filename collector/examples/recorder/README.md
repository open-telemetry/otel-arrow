First, run `make otelarrowcol` in the top-level directory.

To execute the data recorder, for example, where GOOS=darwin and GOARCH=arm64:

```
../../../bin/otelarrowcol_darwin_arm64 --config examples/replay/target.yaml
```

During this phase, data received is copied through a "loopback" Arrow
pipeline that encodes and decodes the data and should second,
identical copies of the inputs.

When enough data has been collected to `first.traces.json` and
`first.metrics.json`, you may stop the target collector.

Note this configuration exercises Arrow in the loopback pipeline, and
if Arrow is failing for any reason the sender will receive errors.
When this is happening, modify the forwarding exporter to disable
Arrow in the recording step, e.g.,:

```
exporters:
  otelarrow/forward:
    arrow:
      enabled: false
```

Then, to re-exercise the same data though an Arrow pipeline using data
recorded in the first step, run:

```
../../../bin/otelarrowcol_darwin_arm64 --config examples/replay/replay.yaml
```

Note that this example only supports traces and metrics.  Logs are not
supported in these configurations.
