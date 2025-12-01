# Quiver (Experimental) - Arrow-Based Persistence for OTAP Dataflow - README

The source for the experimental crate now lives under
`rust/otap-dataflow/crates/quiver`. This directory keeps the architecture notes
and high level design documents, while the shared `otap-dataflow` workspace
hosts the actual Rust code so the project inherits the same lint, fmt, and test
infrastructure.

The crate currently exposes configuration scaffolding, placeholder engine APIs,
and Criterion bench harness stubs. No bytes are persisted yet; every ingest
call intentionally returns `QuiverError::Unimplemented` until Phase 1 wires in
the WAL.

Integration with the `otap-df` binary is opt-in via the Cargo feature
`quiver-persistence`. The feature is *disabled by default*, so release builds
never pull in the experimental persistence code path unless the flag is
explicitly enabled.

## Status

**Not production-ready** This crate is being developed based on the
specifications in `ARCHITECTURE.md` (which may be updated as development
proceeds). It is not yet stable or suitable for taking a dependency on.

## Quick start

```bash
cd rust/otap-dataflow
cargo test -p otap-df-quiver      # unit tests + doc tests
cargo bench -p otap-df-quiver     # opt-in Criterion bench stub
# Enable the downstream integration (still a stub) when needed
cargo test -p otap-df --features quiver-persistence
```

The bench currently measures the placeholder ingest path so we have a home for
future perf instrumentation once real I/O lands.
