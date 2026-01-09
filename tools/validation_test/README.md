### Adding pipelines to the validation process

using the yaml configuration below to define additional pipelines
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

