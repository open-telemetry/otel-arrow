// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Syslog/CEF parsing, Arrow materialization, and internal batch framing.

/// Arrow records encoder for parsed syslog and CEF messages.
pub mod arrow_records_encoder;
mod codec;
mod json;
/// Parsers for RFC 3164, RFC 5424, and raw or syslog-wrapped CEF.
pub mod parser;

#[cfg(feature = "bench")]
pub use codec::bench_support;
#[cfg(feature = "testing")]
pub use codec::full_sized_test_message_capacity;
pub use codec::{
    MAX_BATCH_BYTES, MAX_BATCH_ITEMS, SyslogBatch, SyslogBatchBuilder, SyslogFramingError,
    receipt_local_year, resolve_syslog,
};
pub use json::{MAX_JSON_INPUT_BYTES, SYSLOG_JSON_ENCODING};

/// Maximum size of one syslog or CEF message in an encoded batch.
pub const MAX_MESSAGE_SIZE: usize = 16 * 1024;
