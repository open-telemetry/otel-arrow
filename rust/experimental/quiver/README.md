# Quiver (Experimental) - Arrow-Based Persistence for OTAP Dataflow - README

This crate hosts the experimental Arrow-based persistence components described
in [ARCHITECTURE.md](./ARCHITECTURE.md). It provides a standalone persistence
library which embed into `otap-dataflow` or other telemetry pipelines that need durable
buffering of Arrow payloads.

## Status

**Not production-ready** This crate is being prototyped based on the specifications
below (which may be updated as development proceeds). It is not yet stable or suitable
for taking a dependency on.
