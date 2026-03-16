# Rust Dataflow Engine

## Usage

From the workspace root directory:

```bash
# Build the engine
cargo build --release

# Run with a single pipeline configuration
cargo run --release -- -p configs/otlp-otlp.yaml

# Run with custom core count
cargo run --release -- -p configs/otlp-otlp.yaml --num-cores 4

# Run with an engine configuration (multiple pipeline groups)
cargo run --release -- -c configs/engine-conf/continuous_benchmark.yaml

# Validate configuration without starting the engine
cargo run --release -- -p configs/otlp-otlp.yaml --validate-and-exit

# Get help
cargo run --release -- --help
```

See the [configs README](../configs/README.md) for the difference between
pipeline (`-p`) and engine (`-c`) configuration formats.
