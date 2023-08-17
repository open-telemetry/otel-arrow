This example shows how to use the telemetry-generator with or without
the obfuscation processor to record synthetic data for replay.

First, run `make otelarrowcol` in the top-level directory.

To run this example:

```
../../../bin/otelarrowcol_darwin_arm64 --config ./record.yaml
```

This generates zstd-compressed JSON lines in `recorded_metrics.json` and `recorded_traces.json`.

To replay the same data:

```
../../../bin/otelarrowcol_darwin_arm64 --config ./replay.yaml
```

This generates uncompressed copies of the generated data as JSON lines
in `replayed_metrics.json` and `replayed_traces.json`.

The same JSON files may be used in the `pkg/benchmark/dataset` package
in this repository.
