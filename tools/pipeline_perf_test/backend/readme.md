# Backend Service

The backend service acts as a destination for exported telemetry data. It
currently supports OTLP/gRPC on port `5317`, counts the logs it receives, and
exposes these count at the `:5000/metrics` endpoint.

## Planned Enhancements

- A Null Sink to discard all incoming data.
- A mock service that introduces configurable latency and tracks incoming
  requests.
- A fully functional backend to validate end-to-end pipeline integrity (allowing
  vendor-specific forks to extend functionality).
