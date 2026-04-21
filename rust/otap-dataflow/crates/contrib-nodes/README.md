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
Aggregate flags enable all nodes in their category.

### Receivers

| Node | URN | Module |
| ---- | --- | ------ |
| userevents_receiver | `urn:otel:receiver:userevents` | `src/receivers/userevents_receiver/` |

#### userevents_receiver

- Reads Linux `user_events` tracepoints through per-CPU perf sessions
- Supports single-tracepoint and multi-tracepoint configuration
- Supports the `common_schema_otel_logs` decode format for supported log payloads

### Exporters

- `contrib-exporters` (enables all contrib exporters)

| Feature | Enables Node | Node URN | Module |
| ------- | ------------ | -------- | ------ |
| `geneva-exporter` | Geneva exporter | `urn:microsoft:exporter:geneva` | `src/exporters/geneva_exporter/` |
| `azure-monitor-exporter` | Azure Monitor exporter | `urn:microsoft:exporter:azure_monitor` | `src/exporters/azure_monitor_exporter/` |

### Processors

- `contrib-processors` (enables all contrib processors)

| Feature | Enables Node | Node URN | Module |
| ------- | ------------ | -------- | ------ |
| `condense-attributes-processor` | Condense Attributes processor | `urn:otel:processor:condense_attributes` | `src/processors/condense_attributes_processor/` |
| `recordset-kql-processor` | RecordSet KQL processor | `urn:microsoft:processor:recordset_kql` | `src/processors/recordset_kql_processor/` |
| `resource-validator-processor` | Resource Validator processor | `urn:otel:processor:resource_validator` | `src/processors/resource_validator_processor/` |

When these features are enabled in the top-level binary, their factories are
registered into the OTAP pipeline factory maps.
