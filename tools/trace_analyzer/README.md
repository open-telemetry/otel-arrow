# Trace Analyzer

This tool is designed for trace analysis and the generation of statistical
reports. Its primary function is to ascertain how a given trace dataset is
encoded using the OTel Arrow Protocol.

To view statistics for all .zst files, use the command below. These files should
be in JSON format, compressed with ZSTD and bearing the .zst extension. Batches
are configured to handle 2,000 spans.

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