# OTEL Arrow Protocol Implementation in Rust

The Rust implementation for [OTEL Arrow protocol](https://github.com/open-telemetry/otel-arrow).

- Decoding Arrow IPC record batches to Opentelemetry data structures.
    - 🚧 Metrics
    - [ ] Logs
    - [ ] Traces
- Encoding Opentelemetry data structures to Arrow IPC record batches.
    - [ ] Metrics
    - [ ] Logs
    - [ ] Traces

## Build

```bash
git clone https://github.com/open-telemetry/otel-arrow.git
cd rust/otel-arrow-rust && git submodule update --init --recursive
cargo build --release
```

