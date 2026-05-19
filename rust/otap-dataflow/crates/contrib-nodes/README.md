<!-- markdownlint-disable MD013 -->

# Contrib Nodes

This crate contains contrib receivers, processors, and exporters.

## Folder Layout

- `src/exporters/`
  - Contrib exporters
- `src/receivers/`
  - Contrib receivers
- `src/processors/`
  - Contrib processors

## Features

Feature flags are grouped into aggregate categories and individual node flags.
Aggregate flags enable all feature-gated nodes in their category. Nodes listed
as always enabled are registered whenever the `otap-df-contrib-nodes` crate is
linked.

### Receivers

- `contrib-receivers` (enables all feature-gated contrib receivers)

| Feature | Enables Node | Node URN | Module |
| ------- | ------------ | -------- | ------ |
| `user_events-receiver` | User Events receiver | `urn:otel:receiver:user_events` | `src/receivers/user_events_receiver/` |
| always enabled | STEF receiver | `urn:otel:receiver:stef` | `src/receivers/stef/` |

#### user_events_receiver

- Reads Linux `user_events` tracepoints through per-CPU perf sessions
- Supports single-tracepoint and multi-tracepoint configuration
- Supports tracefs structural decoding by default
- Supports EventHeader decoding when the `user_events-eventheader` feature is
  enabled

#### STEF receiver

- Accepts Collector-compatible STEF metrics streams over gRPC
- Decodes supported STEF metric records directly into OTAP Arrow records
- Currently metrics-only with support for numeric `Gauge` and `Sum` data points

### Exporters

- `contrib-exporters` (enables all feature-gated contrib exporters)

| Feature | Enables Node | Node URN | Module |
| ------- | ------------ | -------- | ------ |
| `geneva-exporter` | Geneva exporter | `urn:microsoft:exporter:geneva` | `src/exporters/geneva_exporter/` |
| `azure-monitor-exporter` | Azure Monitor exporter | `urn:microsoft:exporter:azure_monitor` | `src/exporters/azure_monitor_exporter/` |
| always enabled | STEF exporter | `urn:otel:exporter:stef` | `src/exporters/stef/` |

### Processors

- `contrib-processors` (enables all feature-gated contrib processors)

| Feature | Enables Node | Node URN | Module |
| ------- | ------------ | -------- | ------ |
| `condense-attributes-processor` | Condense Attributes processor | `urn:otel:processor:condense_attributes` | `src/processors/condense_attributes_processor/` |
| `recordset-kql-processor` | RecordSet KQL processor | `urn:microsoft:processor:recordset_kql` | `src/processors/recordset_kql_processor/` |
| `resource-validator-processor` | Resource Validator processor | `urn:otel:processor:resource_validator` | `src/processors/resource_validator_processor/` |

When feature-gated nodes are enabled in the top-level binary, their factories
are registered into the OTAP pipeline factory maps. Always-enabled nodes are
registered as soon as this crate is linked.
