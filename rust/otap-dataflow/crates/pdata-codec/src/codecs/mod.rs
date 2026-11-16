// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod otlp;
#[cfg(feature = "syslog-cef")]
pub mod syslog;
