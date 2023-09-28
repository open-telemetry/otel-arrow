# Trace Analyzer

This tool can be used to analyze traces and generate reports.

The following command displays all the statistics for all the zst files. The
expected format for the data file is JSON (compressed with ZSTD, extension .zst).
The size of the batch is configured 2000 spans.

```shell
go run tools/traces_analyzer/main.go --format json --batch-size 2000 --schema-stats --schema-updates --producer-stats --record-stats *.zst
```

## Supported flags

By default, there is no flag enabled. All these flags are cumulative.

| Flag             | Description                            |
|------------------|----------------------------------------|
| --schema-stats   | Display Arrow schema statistics        |
| --record-stats   | Display Arrow record statistics        |
| --schema-updates | Display Arrow schema updates           |
| --producer-stats | Display OTel Arrow producer statistics |

## Supported formats

| Format | Description                                                |
|--------|------------------------------------------------------------|
| json   | JSON format (compressed with ZSTD and with extension .zst) |
| proto  | Protobuf format                                            |