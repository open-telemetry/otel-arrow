# Ingestion Service / Backend

A destination to which telemetry is exported to ultimately.

- Options include:
  - Null Sink (i.e., drop everything)
  - A fake service that adds configurable latency and tracks requests received.
  - A real backend for validating full pipeline integrity (vendor forks may
    leverage this)
