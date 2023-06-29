This example shows how to use the telemetry-generator with or without
the obfuscation processor to record synthetic data for replay.  

To run this example:

```
go run ../../cmd/otelarrowcol --config ./record.yaml
```

This generates zstd-compressed JSON lines in `recorded_metrics.json` and `recorded_traces.json`.

To replay the same data:

```
go run ../../cmd/otelarrowcol --config ./replay.yaml
```

This generates uncompressed copies of the generated data as JSON lines
in `replayed_metrics.json` and `replayed_traces.json`.

The same JSON files may be used in the `pkg/benchmark/dataset` package
in this repository.
