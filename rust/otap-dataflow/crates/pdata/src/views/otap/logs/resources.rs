// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Resource-level OTAP logs views.

mod decoded;
mod view;

pub use decoded::DecodedOtapLogsResources;
pub use view::OtapLogsResourcesView;
