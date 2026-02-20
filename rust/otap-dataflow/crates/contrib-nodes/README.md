# Contrib Nodes

This crate contains optional (feature-gated) contrib processors and exporters.

## Folder Layout

- `src/exporters/`
  - `azure_monitor_exporter/`
  - `geneva_exporter/`
- `src/processors/`
  - `condense_attributes_processor/`
  - `recordset_kql_processor/`
  - `resource_validator_processor/`

## Features

### Exporters

- `geneva-exporter`
- `azure-monitor-exporter`

### Processors

- `condense-attributes-processor`
- `recordset-kql-processor`
- `resource-validator-processor`

When these features are enabled in the top-level binary, their factories are
registered into the OTAP pipeline factory maps.
