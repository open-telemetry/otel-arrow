# Geneva Exporter

**Status:** ALPHA (Functional scaffold with trait implementation)

The Geneva Exporter is designed for Microsoft products to send telemetry data to Microsoft's Geneva monitoring backend. It is not meant to be used outside of Microsoft products and is open sourced to demonstrate best practices and to be transparent about what is being collected.


# Build df_engine with Geneva Exporter

From the `otap-dataflow` directory:

```bash
cargo build --release --features experimental-geneva
```

## Verify the exporter is registered

```bash
./target/release/df_engine --help
```

You should see `urn:otel:geneva:exporter` in the Exporters list.

## Usage

### Example YAML Configuration (DRAFT)

TODO

### Authentication Options

TODO

### Running

```bash
./target/release/df_engine --pipeline config.yaml --num-cores 4
```

## License

Apache 2.0