# Rust Dataflow Engine

## Usage

From the workspace root directory:

```bash
# Build the engine
cargo build --release

# Run with default configuration
cargo run --release

# Run with custom configuration file
cargo run --release -- -p configs/otlp-otlp.json

# Run with custom core count
cargo run --release -- --num-cores 4

# Run with both custom config and cores
cargo run --release -- -p configs/otlp-otlp.json --num-cores 8

# Get help
cargo run --release -- --help
```
