# System Under Test

The pipeline being tested -eg: OTLP or OTAP pipeline.

- Starts with the latest OTel Collector image and evolves to support
  - Custom Collector builds and config variations
  - Multi-instance topologies via Docker Compose or Kubernetes