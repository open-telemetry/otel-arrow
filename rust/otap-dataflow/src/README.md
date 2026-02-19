# Rust Dataflow Engine

## Usage

From the workspace root directory:

```bash
# Build the engine
cargo build --release

# Run with custom configuration file
cargo run --release -- --config configs/otlp-otlp.yaml

# Short form
cargo run --release -- -c configs/otlp-otlp.yaml

# Get help
cargo run --release -- --help
```
