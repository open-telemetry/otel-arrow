## Validation Tests

The validation test validates a OTLP message after
experiencing various processes such as encoding/decoding
or going through a pipeline.

### Encoding/Decoding Validation

To validate whether encoding/decoding is working properly
we comparing the input and output to check that they are equal

### Pipeline Validation

To validate pipelines we create a pipeline and check
the input and output, to confirm that the data has not
been altered, this assumes that the pipeline doesn't
use processors that can alter the data. Soon we plan
to support these processors in the validation test

####  Adding pipelines to the validation process

Use the yaml configuration below to define additional
pipelines just define non transformative processors to
the defintion and save the configuration under the
`validation_pipelines` directory

```yaml
settings:
  default_pipeline_ctrl_msg_channel_size: 100
  default_node_ctrl_msg_channel_size: 100
  default_pdata_channel_size: 100

nodes:
  receiver:
    kind: receiver
    plugin_urn: "urn:otel:otlp:receiver"
    out_ports:
      out_port:
        destinations:
          - {insert starting processsor}
        dispatch_strategy: round_robin
    config:
      listening_addr: "127.0.0.1:4317"

  {DEFINE PROCESSORS HERE}

  exporter:
    kind: exporter
    plugin_urn: "urn:otel:otlp:exporter"
    config:
      grpc_endpoint: "http://127.0.0.1:4318"
```
