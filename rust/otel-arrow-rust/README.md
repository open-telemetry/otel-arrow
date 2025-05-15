# OTEL Arrow Protocol Implementation in Rust

The Rust implementation for [OTEL Arrow
protocol](https://github.com/open-telemetry/otel-arrow).

- Decoding Arrow IPC record batches to Opentelemetry data structures.
  - :construction: Metrics
    - :white_check_mark: Univariate metrics
    - [ ] Multivariate metrics
  - :white_check_mark: Logs
  - [ ] Traces
- Encoding Opentelemetry data structures to Arrow IPC record batches.
  - [ ] Metrics
  - :construction: Logs
  - [ ] Traces

## Build

```bash
git clone https://github.com/open-telemetry/otel-arrow.git
cd rust/otel-arrow-rust && git submodule update --init --recursive
cargo build --release
```
