# Shared scraper infrastructure

This crate is the shared, database-neutral home for OTAP receiver scraping.
This layer adds database-neutral configuration, query, value, cursor, page, and
driver contracts. It does not implement polling, encode OTLP, persist state,
open database connections, register a receiver, or enable new behavior in
`df_engine`.

## Dependency boundary

- Database receiver modules in `contrib-nodes` may depend on shared scraper
  contracts and their own optional database drivers.
- Shared scraper code must not depend on a vendor receiver or database driver.
- The executable composes registered components and owns application startup.
- Helm charts, container images, and installation scripts are deployment assets,
  not dependencies of the shared runtime.
- Runtime integration reuses the existing engine, telemetry, and pdata APIs.
  Local async contracts preserve the engine's thread-per-core model.

## Follow-on changes

Add durable filesystem checkpoints and source ownership next, followed by
polling, delivery and OTLP mapping, then the optional Oracle adapter. The polling
controller uses the concrete checkpoint store and source lease directly; no
additional persistence or ownership interface is needed for the review split.

Database authentication through extension capabilities is a separate follow-up,
not a new credential mechanism introduced by this skeleton.

See [the database receiver RFC](https://github.com/open-telemetry/otel-arrow/issues/3918)
for the broader design. This crate scaffold does not claim that the full RFC is
implemented.
