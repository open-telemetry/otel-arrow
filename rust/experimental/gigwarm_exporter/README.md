# Geneva Exporter (Experimental)

**Status:** PRE-ALPHA (Scaffold Only)

## Overview

The Geneva Exporter is designed for Microsoft products to send telemetry data to Microsoft's Geneva monitoring backend. It is not meant to be used outside of Microsoft products and is open sourced to demonstrate best practices and to be transparent about what is being collected.

## Current State

This is an initial no-op scaffold with zero functionality. It establishes the API surface for incremental development.

- ✅ Configuration struct and builder pattern
- ✅ No-op method stubs
- ❌ No encoding, upload, or authentication logic

## Usage Example

```rust
use gigwarm_exporter::geneva_exporter;

let exporter = geneva_exporter()
    .with_endpoint("https://geneva.microsoft.com")
    .with_environment("production")
    .with_account("my-account")
    .with_namespace("my-namespace")
    .with_region("westus2")
    .with_tenant("my-tenant")
    .with_role("my-role", "instance-1")
    .build();

// All operations are no-ops in this scaffold
exporter.export(b"data");
exporter.flush();
exporter.shutdown();
```

## Building and Testing

```bash
cargo build
cargo test
cargo clippy
```

## Development

This crate intentionally has zero dependencies. Functionality will be added incrementally:

1. Configuration and validation
2. Arrow RecordBatch processing
3. Geneva Bond encoding and LZ4 compression
4. Authentication (certificate, managed identity)
5. HTTP upload with retry logic
6. Integration with OTAP pipeline

## License

Apache 2.0
