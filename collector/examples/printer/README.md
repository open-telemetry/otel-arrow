# Example: OpenTelemetry Protocol headers and data printer

To execute the printer, which writes a summary of OpenTelemetry export requests
to the console,

```shell
go run .
```

or

```shell
docker build . -t printer
docker run printer
```

Optionally, add `--verbose` to print the data in JSON format.
