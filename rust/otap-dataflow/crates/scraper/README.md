# Scraper Runtime

This crate is currently pre-1.0. Its public API may evolve between minor
releases.

The `otel-arrow-dfe-scraper` crate contains vendor-neutral runtime support for
receivers that periodically scrape external systems. It sits between the OTAP
engine contracts and concrete receiver nodes:

```text
engine -> otap -> scraper -> contrib receiver nodes
```

The crate owns:

- the engine-facing scrape controller and Ack/Nack state machine;
- durable, revisioned checkpoints installed only after downstream Ack;
- exclusive source leases and durable ownership generations;
- bounded, low-cardinality scraper telemetry; and
- database adapter, query, cursor, row, page, and row-to-OTAP contracts.

Concrete receivers remain in `contrib-nodes`. They own public configuration,
vendor drivers, adapter implementations, construction, and component
registration. This crate must not depend on any vendor database driver.

## Database Polling Guarantees

The current database runtime supports composite timestamp and tie-breaker
watermarks. It allows one active query and one in-flight page per checkpoint
source. A matching Ack advances the checkpoint only after crash-safe
persistence succeeds. Nack, stale feedback, timeout, cancellation, send
failure, checkpoint failure, and process termination do not advance it.

Database work and checkpoint I/O run outside the engine's local async core.
Configured row, normalized-memory, and serialized-batch limits bound each
poll. The initial unpartitioned mode still requires one pipeline core per
query/source range; future partition scheduling can build on the lease and
ownership primitives in this crate.
