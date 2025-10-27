# Geneva Exporter (Experimental)

**Status:** ALPHA (Functional scaffold with trait implementation)

## Overview

The Geneva Exporter is designed for Microsoft products to send telemetry data to Microsoft's Geneva monitoring backend. It is not meant to be used outside of Microsoft products and is open sourced to demonstrate best practices and to be transparent about what is being collected.

## Current State

This is a functional scaffold that implements the full `Exporter<OtapPdata>` trait and can be integrated with the OTAP dataflow engine. The message loop is functional but encoding/upload logic is still no-op.

**Implemented:**
- ✅ Configuration struct with serde deserialization
- ✅ `Exporter<OtapPdata>` trait implementation
- ✅ Distributed slice registration (`linkme`)
- ✅ Message loop with shutdown handling
- ✅ Metrics registration (placeholder)
- ✅ Can be discovered by `df_engine` binary

**Not Yet Implemented:**
- ❌ Arrow RecordBatch encoding
- ❌ Geneva Bond format encoding
- ❌ LZ4 compression
- ❌ HTTP upload to Geneva
- ❌ Authentication (certificate, managed identity)

## Building

### Standalone Build

```bash
cargo build
cargo test
```

### Integration with OTAP Dataflow

To integrate with the `df_engine` binary:

**1. Add to workspace** (`../../otap-dataflow/Cargo.toml`):
```toml
[workspace]
members = [
    "benchmarks",
    "crates/*",
    "xtask",
    "../experimental/gigwarm_exporter",  # ← Add this
]
```

**2. Import in otap crate** (`../../otap-dataflow/crates/otap/src/lib.rs`):
```rust
// Import to trigger distributed_slice registration
extern crate gigwarm_exporter;
```

**3. Build df_engine**:
```bash
cd ../../otap-dataflow
cargo build --release
```

**4. Verify registration**:
```bash
./target/release/df_engine --help
# Should show: "urn:otel:geneva:exporter" in Exporters list
```

## Usage

### Example YAML Configuration

See `configs/geneva-example.yaml` for a complete example:


### Running

```bash
./target/release/df_engine --pipeline configs/geneva-example.yaml --num-cores 4
```

## Current Behavior

The exporter currently:
- ✅ Accepts configuration from YAML
- ✅ Starts successfully and logs startup
- ✅ Receives PData messages
- ✅ Logs receipt of data (no-op - discards data)
- ✅ Handles shutdown gracefully
- ❌ Does NOT encode or upload to Geneva (placeholder)


## License

Apache 2.0
