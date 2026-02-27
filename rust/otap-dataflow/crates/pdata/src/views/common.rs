// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains backend-agnostic, zero-copy view traits for common OTLP message types
//! such as `AnyValue`, `KeyValue`, and `InstrumentationScope`.
//!
//! It also contains common helper types and structs such as `ValSlice` and `Str` which do not have
//! an analogous proto message, but are available as common return types for other View trait
//! implementations

pub use crate::schema::{SpanId, TraceId};
pub use otap_pdata_views::views::common::{
    AnyValueView, AttributeView, InstrumentationScopeView, Str, ValueType,
};
