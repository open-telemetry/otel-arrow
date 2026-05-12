# Rust Dataflow Engine

## Usage

From the workspace root directory:

```bash
# Build the engine
cargo build --release

# Run with a config file (bare path)
cargo run --release -- --config configs/otlp-otlp.yaml

# Explicit file: URI (same result)
cargo run --release -- --config file:configs/otlp-otlp.yaml

# Load config from an environment variable
export MY_CONFIG=$(cat configs/otlp-otlp.yaml)
cargo run --release -- --config env:MY_CONFIG

# Short form
cargo run --release -- -c configs/otlp-otlp.yaml

# Validate configuration without starting the engine
cargo run --release -- --config configs/otlp-otlp.yaml --validate-and-exit

# Omit --config to fall back to config.yaml in the current directory
cargo run --release --

# Get help
cargo run --release -- --help
```

### Config URI schemes

The `--config` argument accepts a URI that specifies the config source:

| URI form | Behavior |
| --- | --- |
| `file:/path/to/config.yaml` | Read config from a local file |
| `env:MY_VAR` | Read the full config from an environment variable |
| `/path/to/config.yaml` | Bare path, treated as `file:` |
| `./relative/config.yaml` | Relative path, treated as `file:` |

If `--config` is omitted, the engine looks for `config.yaml` in the
current working directory. Both `.yaml` and `.json` files are supported.
