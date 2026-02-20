# Contrib Nodes

This crate contains optional (feature-gated) contrib processors and exporters.

## Folder Layout

- `src/exporters/`
  - Contrib exporters
- `src/processors/`
  - Contrib processors

## Features

Feature flags are grouped into aggregate categories and individual node flags.
Aggregate flags enable all nodes in their category.

### Exporters

- `contrib-exporters` (enables all contrib exporters)

| Feature | Enables Node | Node URN | Module |
| ------- | ------------ | -------- | ------ |
| `geneva-exporter` | Geneva exporter | `urn:microsoft:geneva:exporter` | `src/exporters/geneva_exporter/` |
| `azure-monitor-exporter` | Azure Monitor exporter | `urn:microsoft_azure:monitor:exporter` | `src/exporters/azure_monitor_exporter/` |

### Processors

- `contrib-processors` (enables all contrib processors)

| Feature | Enables Node | Node URN | Module |
| ------- | ------------ | -------- | ------ |
| `condense-attributes-processor` | Condense Attributes processor | `urn:otel:condense_attributes:processor` | `src/processors/condense_attributes_processor/` |
| `recordset-kql-processor` | RecordSet KQL processor | `urn:microsoft:recordset_kql:processor` | `src/processors/recordset_kql_processor/` |
| `resource-validator-processor` | Resource Validator processor | `urn:otel:resource_validator:processor` | `src/processors/resource_validator_processor/` |

When these features are enabled in the top-level binary, their factories are
registered into the OTAP pipeline factory maps.
