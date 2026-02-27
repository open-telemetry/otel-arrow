// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! **Backend-agnostic, zero-copy view traits for OTLP Logs.**
//! ```text
//! LogsView
//! └─ ResourceLogsView
//!    │  resource::ResourceView
//!    └─ ScopeLogsView
//!       │  InstrumentationScopeView
//!       └─ LogRecordView
//!          └─ common::AttributeView
//!             └─ common::AnyValueView
//! ```

pub use otap_pdata_views::views::logs::{
    LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
};
