# OTel Arrow SQL Agent Receiver Architecture and Packaging Design

<!-- markdownlint-disable MD013 -->

| Document field | Value |
| --- | --- |
| Status | Draft for collaboration |
| Scope | Early SQL agent and reusable database polling receiver |
| Primary reviewer | Jitender Bisht |
| Last updated | September 11, 2026 |
| Classification | Internal working design |

## Document purpose

Define a thin, locally configured SQL agent that reuses OTel Arrow
infrastructure, isolates vendor drivers per build, and preserves the same
receiver implementation for future managed DF Engine hosting.

## Document conventions

| Marker | Meaning |
| --- | --- |
| E | Existing capability reused without a new component |
| P | Proposed design or extension requiring review |
| N | New implementation work |
| Future | Managed-product work outside the initial SQL-agent delivery |

## Related design inputs

| Document | Use in this design |
| --- | --- |
| Azure Monitor On-Prem Database Data Collection Connector | Document structure, product context, control-plane boundary, and managed target |
| Generic Database Receiver Design | Shared polling runtime, adapter boundary, progress semantics, bounds, security, observability, and conformance requirements |
| OTel Arrow database receiver architecture skeleton | Current crate proposal and link-time `ReceiverFactory` integration model |

## Accuracy and implementation status

This document combines verified repository behavior with proposed database
receiver behavior. The distinction is important:

| Claim | Verified status |
| --- | --- |
| OTel Arrow provides a controller, pdata pipeline, component factories, link-time factory inventories, backpressure, ACK/NACK routing, and OTLP gRPC exporter | Existing |
| The OTLP gRPC exporter is registered as `urn:otel:exporter:otlp_grpc` and forwards export-RPC completion as pipeline ACK/NACK feedback | Existing |
| `database-agent`, `database-scraper`, and the vendor receiver crates exist at their intended paths | Design skeleton only; excluded from the Cargo workspace |
| `database-agent` declares optional vendor features and feature-gated side-effect imports | Present in the skeleton |
| Exactly-one-vendor compile guards | Proposed; not implemented in the skeleton |
| Oracle factory registration and `urn:otel:receiver:oracle` | Declared in the skeleton; configuration validation and receiver creation remain `todo!()` |
| Shared polling, mapping, completion tracking, and checkpoint behavior | Interfaces and pseudocode only; runtime methods remain `todo!()` |
| Early OTLP export to a supplied Strato endpoint | Delivery-strategy assumption from the working discussion; endpoint authentication remains unresolved |
| DCR/AMCS-managed database receiver delivery | Future product integration; not implemented by this skeleton |

## Executive summary

This design proposes a small OTel Arrow-based SQL agent for early customer
validation before new DCR and AMCS integration is available. The agent reads a
constrained local contract from environment variables and mounted files, starts
the normal Dataflow controller, and is intended to send OTLP logs to a supplied
Strato OTLP endpoint.

The implementation separates a vendor-neutral polling runtime from
vendor-specific receiver crates. The proposed packaging model reuses one
`database-agent` source crate across vendors, while building each customer
artifact with exactly one mutually exclusive vendor feature. Under that
invariant, an Oracle artifact contains the Oracle receiver and driver but not
PostgreSQL, SQL Server, or MySQL dependencies.

The proposed managed integration later links the same vendor receiver crate
into the customer-cluster OTAP collector/DF Engine. The product design uses DCR,
AMCS, an in-cluster `dcr-config-agent`, and Strato reconciliation to materialize
managed collector configuration and workload resources. These control-plane
steps replace local bootstrap configuration; they do not replace the receiver,
polling runtime, mapping, checkpoint, or delivery behavior.

> **Key outcome:** One shared polling implementation, one receiver component
> per vendor, and one vendor driver per early-delivery artifact.

## Executive architecture

```text
ONE SOURCE CRATE                         VENDOR-ISOLATED ARTIFACTS

database-agent                          Oracle build
+-- optional receiver-oracle            +-- database-agent --features oracle
+-- optional receiver-postgresql        +-- receiver-oracle
+-- optional receiver-sql-server        +-- Oracle driver
+-- optional receiver-mysql             +-- shared polling runtime

                                        PostgreSQL build
                                        +-- database-agent --features postgresql
                                        +-- receiver-postgresql
                                        +-- PostgreSQL driver
                                        +-- shared polling runtime
```

The proposed build policy permits exactly one vendor feature in each agent
artifact. The current skeleton does not yet enforce this policy.

## Key deliverable mapping

| Deliverable | Architecture mapping | Outcome |
| --- | --- | --- |
| Shared polling runtime | Vendor-neutral scheduling, bounds, mapping, ACK/checkpoint, retry, lifecycle, and telemetry | Consistent correctness across database vendors |
| Oracle receiver | Oracle adapter, driver, binds, cancellation, values, and `ReceiverFactory` | Oracle capability without unrelated vendor dependencies |
| Early SQL agent | Local bootstrap, exactly one vendor feature, and the existing OTLP exporter | Customer testing without new DCR/AMCS support |
| Managed reuse | Customer-cluster OTAP collector/`df_engine` links the same receiver factory | No second receiver or polling implementation |

## Part I - High-Level Design

## 1. Objective and scope

The initial objective is to deliver a database polling receiver through an OTel
Arrow-based binary that can run in a customer-controlled environment and export
through a supplied Strato OTLP endpoint without waiting for the complete
managed control-plane experience.

### In scope

- Local environment-variable and mounted-file bootstrap.
- One vendor receiver and driver per compiled customer artifact.
- Normal OTAP pipeline execution and OTLP export.
- Shared polling correctness, progress, and bounded-resource behavior.
- Reuse of the same receiver implementation by the managed DF Engine.

### Out of scope

- Portal UX, a new DCR schema, and AMCS translation.
- Runtime driver installation or dynamic native-library loading.
- CDC and transaction-log ingestion.
- Final managed deployment, ownership, and scaling policy.
- A second Dataflow runtime or a second OTLP exporter.

## 2. Design principles

| Principle | Design implication |
| --- | --- |
| Database-agnostic core, database-specific adapter | Scheduling, progress, mapping, and delivery rules are shared; connection, binds, cancellation, and native values remain vendor-specific |
| Static composition | A receiver must be compiled into the host before its URN can be selected at runtime |
| One vendor per early artifact | The proposed build policy permits exactly one Cargo vendor feature in a packaged `database-agent` artifact |
| Thin agent | The agent owns bootstrap and composition, not polling or export implementations |
| Bounded by construction | Rows, bytes, queries, connections, workers, pending ACKs, and admitted memory have explicit limits |
| Fail closed on uncertain progress | A checkpoint never advances after NACK, timeout, destination failure, or persistence failure |
| Reuse before duplication | Use the existing controller, pdata pipeline, backpressure, factory registry, and OTLP exporter |

## 3. Component architecture

```text
database-agent (one source crate, exactly one vendor feature)
    |
    +-- receiver-oracle
    |       +-- Oracle driver and vendor behavior
    |       +-- database-scraper (shared polling runtime)
    |
    +-- normal Dataflow controller
    +-- existing OTLP exporter

managed df_engine
    +-- same receiver-oracle
            +-- same database-scraper
```

| Component | Responsibility | Dependency rule | State |
| --- | --- | --- | --- |
| `database-scraper` | Shared polling policy and progress state machine | No vendor driver and no dependency on either host | N |
| `receiver-oracle` | Oracle connection, binds, cancellation, values, and factory composition | Depends on `database-scraper` and the Oracle driver only | N |
| Other vendor receivers | Equivalent adapter and factory boundary for each vendor | Each owns only its corresponding driver | Future |
| `database-agent` | Local input validation, constrained pipeline construction, and controller startup | Exactly one optional vendor receiver is enabled per build | N |
| Dataflow controller and pdata pipeline | Pipeline validation, lifecycle, wiring, and backpressure | Reused; not reimplemented by the agent | E |
| OTLP exporter | Sends produced logs to the supplied Strato endpoint | Existing component; packaging inventory requires review | E/P |
| Customer-cluster OTAP collector/managed `df_engine` | Future host for the same receiver factories | Depends directly on receiver crates, never on `database-agent` | Future |

## 4. Proposed architecture decisions

These decisions are the current recommendation. D3 remains subject to explicit
agreement on whether one feature-selected source crate satisfies the packaging
boundary.

| ID | Decision | Rationale | Alternative rejected |
| --- | --- | --- | --- |
| D1 | Use a shared database polling runtime beneath vendor receiver crates | Preserves one correctness model while isolating drivers and dialect behavior | Independent polling implementation per vendor |
| D2 | Keep each public receiver vendor-specific | Configuration, driver behavior, value decoding, and compatibility are vendor-owned | One public generic SQL receiver with all drivers |
| D3 | Use one thin `database-agent` source crate with exactly one vendor feature per build | Avoids duplicate wrappers while producing driver-isolated artifacts | One all-vendor binary or several nearly identical wrappers |
| D4 | Select the vendor at build time and configure the instance at runtime | Rust drivers are statically linked; endpoints, queries, and secrets vary by deployment | Runtime selection or download of missing drivers |
| D5 | Use link-time `ReceiverFactory` registration and receiver URNs | Factory inventories and URN lookup already exist in OTel Arrow and support managed reuse | Custom agent-only receiver startup |
| D6 | Advance checkpoints only after downstream ACK and durable commit | Prevents false progress and data loss after failed delivery | Checkpoint after channel enqueue or before delivery |
| D7 | Provide at-least-once delivery | Crash timing can replay accepted rows before checkpoint persistence | Claim exactly-once without an end-to-end transaction |

## 5. Shared database polling runtime

The shared runtime is the common polling policy that becomes the foundation for
database receiver mode in OTel Arrow. The working crate name is
`database-scraper`; the final name remains an open decision.

### 5.1 Responsibilities

- Validate shared configuration and coordinate receiver lifecycle.
- Schedule independent queries with no overlap and optional startup jitter.
- Enforce admission limits for queries, connections, blocking workers, pages,
  bytes, and pending acknowledgements.
- Normalize database rows and map one source row to one OTLP `LogRecord`.
- Load committed cursors, retain candidate progress, correlate ACK/NACK
  feedback, and commit checkpoints.
- Propagate backpressure, enforce deadlines, coordinate cancellation, drain,
  shutdown, and emit common telemetry.

### 5.2 Vendor adapter boundary

| Shared runtime owns | Vendor adapter owns |
| --- | --- |
| Scheduling and no-overlap policy | Connection and pool implementation |
| Logical typed bind values | Vendor parameter-marker and bind syntax |
| Page row and byte limits | Efficient database-side row limits and bounded fetch |
| Deadline and cancellation intent | Driver/server cancellation and connection cleanup |
| Neutral row and `CellValue` model | Vendor metadata and native value decoding |
| Cursor and checkpoint state machine | Dialect-correct keyset predicates and ordering |
| Common error classes and retry policy | Vendor error classification and connection health |
| Conformance contract and stable metrics | Database, driver, and native-client compatibility matrix |

> **Boundary rule:** The shared runtime represents intent and correctness
> policy. It must not synthesize vendor SQL, cancellation behavior, or native
> value decoding.

### 5.3 Row and value model

Every vendor value is normalized into a closed, database-neutral `CellValue`
representation before OTLP conversion. The baseline categories are null,
Boolean, signed and unsigned integers, decimal, floating point, string, bytes,
date/time, interval, JSON, and UUID.

- One database row maps to one OTLP `LogRecord`.
- Decimal and other non-lossless numeric values default to strings.
- Unsigned values map to OTLP integers only when representable.
- Bytes map to base64 strings.
- Unsupported values and invalid encodings are explicit conversion errors.
- Null behavior is policy-controlled.
- Values are never silently truncated.
- Column names remain unique after normalization and configured renaming.

JSON-compatible bodies are the safest initial baseline. Structured map bodies
remain subject to exporter and processor compatibility review.

### 5.4 Cursor and query contract

Incremental polling uses deterministic keyset pagination. A scalar watermark is
valid only when it uniquely and monotonically orders the source range;
otherwise a stable composite cursor, typically timestamp plus key, is required.

- Resume strictly after the committed cursor and return every cursor column.
- Require non-null, losslessly represented cursor values.
- Require the last cursor of a non-empty page to advance.
- Apply client-side hard row and byte bounds even when vendor SQL limits rows.
- Do not use offset pagination as the reliability baseline.
- Define how late commits are handled: commit-ordered marker, bounded overlap
  with deduplication, or CDC when neither polling contract is safe.

### 5.5 ACK-gated checkpoint state machine

```text
load committed cursor C0
        |
        v
query one bounded page -> candidate cursor C1
        |
        v
normalize rows and send one bounded batch
        |
        v
wait for terminal downstream ACK/NACK
        |
        +-- NACK / timeout / shutdown -> retain C0 and replay later
        |
        +-- ACK -> durably compare-and-set checkpoint to C1
                         |
                         +-- commit succeeds -> C1 becomes active
                         +-- commit fails    -> retain C0 and stop progress
```

The proposed initial reliable profile permits one in-flight batch per
independently checkpointed query or partition. This is not implemented in the
current skeleton. Multiple in-flight batches require ordered completion
tracking and commitment of only the largest contiguous ACKed prefix.

A crash after destination acceptance but before checkpoint persistence can
replay rows. The delivery contract is therefore at least once; consumers should
use stable source keys where deduplication is required.

The exact meaning of terminal ACK is still open. OTel Arrow can route completion
feedback and the OTLP gRPC exporter reports export-RPC completion, but the
product must decide whether that boundary is sufficient for the customer-facing
checkpoint guarantee.

### 5.6 Scheduling, bounds, and backpressure

| Invariant | Required behavior |
| --- | --- |
| No overlap | Do not start a second execution for the same logical cursor while query, delivery, or checkpoint work remains unresolved |
| Missed ticks | Use delay-style handling; do not create overlapping catch-up timers |
| Backpressure | A blocked downstream send pauses that query or partition before the next page |
| Connections | One active query per physical connection; query concurrency cannot exceed connection capacity |
| Memory | Bound rows, normalized bytes, fetch size, active pages, pending ACKs, and total admitted memory |
| Oversize values | Fail explicitly or use a reviewed oversize policy; never silently truncate |
| Retry | Retry only classified transient failures with bounded backoff and jitter |

### 5.7 Deadlines, cancellation, and lifecycle

The runtime owns whole-request deadlines, cancellation propagation, and the
decision to stop progress when connection state is uncertain. The adapter owns
the strongest vendor-supported cancellation mechanism and the cleanup required
before a connection can be reused.

- Include admission wait, prepare, execute, fetch, and decode in the deadline.
- Never advance a checkpoint after timeout.
- Discard or retire a connection whose state cannot be proven safe.
- Stop new queries during drain and resolve in-flight feedback within the
  remaining deadline.
- Preserve the last durable checkpoint for unresolved work during shutdown.

## 6. Agent build and packaging model

### 6.1 Build-time versus runtime selection

| Build time | Runtime |
| --- | --- |
| Selects the single database vendor feature | Supplies endpoint, service/database, query, cursor, limits, secret references, checkpoint path, and OTLP endpoint |
| Compiles and links one receiver and driver | Builds and validates a constrained `OtelDataflowSpec` |
| Registers the selected `ReceiverFactory` | Resolves the already-linked factory by URN |
| Produces a vendor-specific artifact | Starts polling; it cannot add a missing driver |

> **Important:** The runtime URN selects an already-linked factory. It cannot
> determine which Rust dependencies were compiled into the executable.

### 6.2 Exactly-one-vendor enforcement

The current skeleton declares all vendor dependencies as optional and leaves
the default feature set empty:

```toml
[features]
default = []
oracle = ["dep:otel-arrow-dfe-receiver-oracle"]
postgresql = ["dep:otel-arrow-dfe-receiver-postgresql"]
sql-server = ["dep:otel-arrow-dfe-receiver-sql-server"]
mysql = ["dep:otel-arrow-dfe-receiver-mysql"]
```

Compile-time guards enforce:

```text
zero vendor features     -> build fails
one vendor feature       -> build succeeds
multiple vendor features -> build fails
```

The guards above are proposed and do not yet exist in `database-agent`.

Vendor builds must run in separate Cargo invocations because Cargo feature
unification is additive. Two repository workflows also require a decision
before the crate joins the workspace:

- a zero-feature guard would fail normal workspace builds that compile
  `database-agent` with its empty default feature set; and
- a multi-feature guard would intentionally fail an `--all-features` build.

Options include a default early vendor, packaging-only enforcement plus CI
dependency checks, a dedicated packaging target, or separate vendor composition
crates. The selected option must preserve normal workspace validation while
still making a mixed-driver customer artifact impossible.

### 6.3 Oracle package example

```console
cargo build --release \
  -p otel-arrow-dfe-database-agent \
  --no-default-features \
  --features oracle
```

The Oracle artifact contains:

```text
database-agent
+-- receiver-oracle
|   +-- Oracle driver
|   +-- database-scraper
+-- normal Dataflow controller
+-- existing OTLP exporter
```

With only the Oracle feature enabled, the PostgreSQL, SQL Server, and MySQL
receiver crates are absent from the Cargo dependency graph. The packaged native
library inventory must be checked separately. The release pipeline should use
an explicit artifact name such as `otel-arrow-database-agent-oracle`.

### 6.4 Runtime input contract

The early agent reads documented non-secret values and mounted-file references.
Credentials do not belong directly in environment variables or generated
configuration.

| Input group | Illustrative values | Validation |
| --- | --- | --- |
| Source | Endpoint, service/database, read-only session options | Scheme, length, supported options, and vendor compatibility |
| Query | Mounted query file and cursor-column mapping | Protected file, deterministic order, returned cursor fields, and bind contract |
| Limits | Interval, timeout, rows/bytes, connections, workers | Non-zero bounds and consistency with connection capacity |
| State | Checkpoint directory and source identity | Writable storage, compatible fingerprint, and non-regressing state |
| Destination | Strato OTLP endpoint and credential-provider reference | TLS, endpoint, bounded timeout, and approved authentication |

The selected receiver URN should be derived from the compile-time feature. The
current skeleton still models a runtime `ReceiverSelection`; implementation
must either remove that field or validate it against the compiled vendor and
reject a mismatch.

## Part II - End-to-End Behavior

## 7. Early-delivery flow

```text
Package build: --features oracle
        |
        v
Oracle-only database-agent binary
        |
        v
environment variables + mounted files
        |
        v
constrained OtelDataflowSpec
        |
        v
type: urn:otel:receiver:oracle
        |
        v
Oracle ReceiverFactory -> receiver-oracle --uses--> shared polling runtime
        |
        v
OTAP pdata pipeline -> existing OTLP gRPC exporter -> supplied Strato endpoint
```

The agent is a host and bootstrap layer. It does not schedule queries, connect
to the database, map rows, manage checkpoints, implement ACK tracking, or
export OTLP itself. Receiver construction, pipeline construction, and the
Strato authentication binding remain unimplemented design work.

## 8. Managed DF Engine reuse

```text
Portal / ARM / Bicep
        |
        v
DCR -> AMCS -> in-cluster dcr-config-agent
        |
        v
Strato DcrReconciler
        |
        v
collector ConfigMap + secret mounts + PVC + StatefulSet
        |
        v
customer-cluster OTAP collector / df_engine with Oracle receiver
        |
        v
same Oracle ReceiverFactory -> same receiver-oracle
        |
        +-- uses same shared polling runtime
        |
        v
same OTAP data pipeline -> transform / batch / durable buffer
        |
        v
Azure Monitor exporter -> DCR stream -> Log Analytics destination
```

The managed host depends directly on selected receiver crates and never on
`database-agent`. Moving to managed delivery changes configuration ownership
and product lifecycle; it does not require rewriting the Oracle receiver or
polling runtime.

The managed product design does not describe Strato as the final data
destination. It uses the Azure Monitor exporter and a DCR stream to reach Log
Analytics. The early OTLP-to-Strato endpoint is a short-term delivery path, not
the managed exporter contract.

## 9. Failure behavior

| Failure class | Required behavior |
| --- | --- |
| Configuration or query contract | Fail startup with a specific redacted error |
| Authentication | Report clearly; retry only under an explicit credential-rotation policy |
| Transient transport | Reconnect with bounded backoff and jitter |
| Timeout or cancellation | Do not advance; cancel or retire the connection and retry according to policy |
| Conversion or oversize value | Fail the batch by default; do not truncate or skip silently |
| Downstream NACK or timeout | Retain the committed cursor and replay |
| Checkpoint persistence or CAS conflict | Stop affected progress; never assume a new position |
| Corrupt or incompatible checkpoint | Fail closed and require an explicit recovery action |

## 10. Security and trust boundaries

- Use a dedicated read-only database principal restricted to required schemas,
  views, or procedures.
- Bind cursor and static values as typed parameters.
- Never concatenate untrusted values into SQL.
- Require server-identity-verifying TLS in production.
- Treat SQL, row values, connection strings, credentials, and cursors as
  sensitive.
- Do not emit sensitive values in logs, metrics, errors, debug configuration,
  or administrative APIs.
- Use mounted secret files or an approved credential-provider capability.
- Initiate outbound database and OTLP connections; do not require software on
  the database host or an inbound database receiver port.

## 11. Observability

The proposed shared metric namespace is `receiver.database`. Final metric names
must follow OTel Arrow component naming conventions. Dimensions must remain
bounded and must not include endpoint, SQL, table names, cursor values, raw
errors, or row data.

| Signal group | Required signals |
| --- | --- |
| Polling | Polls started/completed/failed, duration, rows, and normalized bytes |
| Resources | Active queries, connections, workers, admitted memory, and admission rejection |
| Delivery | Batches sent, pending ACKs, ACKs, NACKs, and backpressure duration |
| Checkpoint | Commit success/failure, latency, age, and optional source lag |
| Recovery | Retry class, reconnects, timeout/cancellation outcome, and invalid cursor/query-contract failures |

## 12. Validation strategy

| Layer | Required coverage |
| --- | --- |
| Shared runtime unit tests | Scheduling, bounds, row mapping, cursor ordering, ACK/NACK state, checkpoint CAS, retry, and shutdown |
| Adapter conformance | Metadata, nulls, binds, conversion, bounded fetch, timeout, cancellation, reconnect, and connection health |
| Real database integration | Supported versions, connection loss, malformed values, deadlines, and restart replay |
| Pipeline integration | Backpressure, durable buffering, OTLP interoperability, destination failure, and checkpoint invariants |
| Packaging | Exactly one factory, disabled crates absent from `cargo tree`, expected native libraries, and artifact inventory |
| Security | Secret redaction, TLS verification, injection resistance, least privilege, and protected files |
| Performance | Bounded allocations, throughput, poll latency, cancellation latency, and source database load |

## 13. Delivery plan

| Phase | Content | Exit criteria |
| --- | --- | --- |
| 1. Design agreement | Runtime boundary, feature packaging, input contract, and open-decision owners | Architecture and package invariants approved |
| 2. Shared runtime foundation | Adapter contract, row model, bounds, scheduling, ACK/checkpoint state, and fake adapter tests | Conformance suite passes with bounded resources |
| 3. First vendor receiver | First approved adapter, driver, mapping, cancellation, and `ReceiverFactory`; examples in this document use Oracle | Real database and failure-path tests pass |
| 4. Early agent distribution | Env/file bootstrap, one-vendor build, OTLP pipeline, packaging, and deployment guidance | Dependency and native inventory prove a vendor-isolated artifact |
| 5. Managed integration | `df_engine` dependency, registration, managed translation, and destination integration | The same receiver passes managed end-to-end validation |
| 6. Additional vendors | Add receiver crates and independent feature builds | Each adapter passes the shared conformance suite |

## 14. Open decisions

1. Confirm whether one source crate with exactly-one-vendor enforcement
   satisfies the packaging requirement, or separate vendor agent crates are
   required.
2. Finalize the shared runtime name and public adapter API.
3. Define the terminal downstream ACK that permits checkpoint advancement.
4. Select the production checkpoint backend, identity/fingerprint contract,
   and ownership model.
5. Decide whether one interval reads one page or continues bounded paging until
   caught up.
6. Finalize secret delivery and Strato exporter authentication.
7. Determine whether linking `core-nodes` is acceptable or the existing OTLP
   exporter should move to a narrower crate.
8. Resolve the source-design sequencing inconsistency and confirm the first
   vendor: Oracle is described as a proving implementation, while another MVP
   section identifies PostgreSQL first.
9. Align exactly-one-feature enforcement with repository-wide
   `--all-features` workflows.

## 15. Review focus

- Does the shared runtime contain only vendor-neutral correctness and
  orchestration behavior?
- Does each vendor receiver own its complete driver, cancellation, metadata,
  and value-conversion boundary?
- Can an Oracle artifact be proven free of all other vendor dependencies and
  native libraries?
- Does the receiver preserve bounded work and backpressure under slow databases
  and slow destinations?
- Can the same `ReceiverFactory` execute unchanged in the early agent and
  managed `df_engine`?
- Are progress, replay, failure, and security guarantees explicit and testable?
