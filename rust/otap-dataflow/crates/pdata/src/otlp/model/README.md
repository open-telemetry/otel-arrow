# otel-arrow-dfe-pdata-otlp-model

This crate is currently pre-1.0. Its public API may evolve between minor
releases.

## Overview

Build-time OTLP model metadata used by the OTAP pdata code generator.

## Types

- `ParamConfig`: required and ignored fields for generated constructors
- `OneofCase` and `OneofMapping`: protobuf `oneof` variant metadata
- `EnumField`: protobuf enum representation overrides
- `REQUIRED_PARAMS`, `ONEOF_MAPPINGS`, and `FIELD_TYPE_OVERRIDES`: metadata
  tables consumed during generation
- `add_type_attributes`: configures a tonic/prost builder with the OTLP macros

This is an implementation crate for `otel-arrow-dfe-pdata` and its procedural
macros. Most applications should depend on `otel-arrow-dfe-pdata` instead.
