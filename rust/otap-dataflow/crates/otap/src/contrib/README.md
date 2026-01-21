# Contributed Components

This directory contains contributed components that are not fully supported but are related to project goals mentioned in the [OTel-Arrow Project Phases](../../../../../../docs/project-phases.md).

## Structure

Components are organized by type:

- `exporter/` - Contributed exporters (e.g., Geneva, Azure Monitor)
- `processor/` - Contributed processors (e.g., Condense Attributes, Recordset KQL)
- `receiver/` - Contributed receivers (currently empty, reserved for future use)

## Status

These components were previously located in the `experimental` module. The renaming to `contrib` better indicates their status as contributed components that may be promoted to core in the future or moved to a separate `otel-arrow-contrib` repository.

## Usage

Contributed components are gated behind feature flags. See the main crate documentation for details on enabling specific components.
