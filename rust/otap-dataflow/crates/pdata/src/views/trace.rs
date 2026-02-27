// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! **Backend-agnostic, zero-copy view traits for OTLP Traces.**
//! ```text
//! TracesView
//! └─ ResourceSpansView
//!    │  resource::ResourceView
//!    └─ ScopeSpansView
//!       │  common::InstrumentationScopeView
//!       └─ SpanView
//!          ├─ common::AttributeView
//!          │  └─ common::AnyValueView
//!          ├─ EventView
//!          │  └─ common::AttributeView
//!          │     └─ common::AnyValueView
//!          ├─ LinkView
//!          │  └─ common::AttributeView
//!          │     └─ common::AnyValueView
//!          └─ StatusView
//! ```

pub use otap_pdata_views::views::trace::{
    EventView, LinkView, ResourceSpansView, ScopeSpansView, SpanView, StatusView, TracesView,
};
