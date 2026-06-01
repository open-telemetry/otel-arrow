# Basic Operators

You can filter the things using `where`

## Filter (`where`)

The `where` operator can be used to filter telemetry data. Any fields telemetry
item that matches the logical expression will be kept.

```
logs | where severity_text == "ERROR"
```

## Assign (`set`)