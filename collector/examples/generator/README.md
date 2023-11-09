# Telemetry generator example configuration

The collector configuration in this directory produces synthetic
telemetry and exports to the standard port (4317) used by OTLP and
OpenTelemetry Protocol with Apache Arrow.

To start the generator, starting from this directory:

```
docker run -v `pwd`:/config -w /config otelarrowcol --config /config/generator.yaml
```

or

```
../../../bin/otelarrowcol --config generator.yaml
```

With this running, you can run the other examples such as a
[bridge](../bridge/README.md).
