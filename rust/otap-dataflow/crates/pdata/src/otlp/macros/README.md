# otel-arrow-dfe-pdata-otlp-macros

This crate is currently pre-1.0. Its public API may evolve between minor
releases.

## Overview

Procedural macros used to generate builders and OTLP-specific metadata for the
OTAP pdata model.

## Macros

- `Message`: derives OTLP builder methods for a generated Prost message
- `qualified`: records the message's fully qualified protobuf type name for
  code generation

This is an implementation crate for `otel-arrow-dfe-pdata`. Most applications
should depend on that crate instead of using these macros directly.
