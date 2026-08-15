// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded JSON Lines framing for the file exporter.
//!
//! The shared pdata serializers write unframed OTLP JSON to any synchronous writer. This module
//! wraps those serializers with an exporter-owned byte limit, reuses the caller's buffer, and
//! appends a newline only after a complete document has been encoded successfully.

use otap_df_pdata::otlp::json::{
    JsonEncodeError, write_logs_json, write_metrics_json, write_traces_json,
};
use otap_df_pdata_views::views::logs::LogsDataView;
use otap_df_pdata_views::views::metrics::MetricsView;
use otap_df_pdata_views::views::trace::TracesView;
use std::io::{self, Write};

/// Failure while encoding one bounded OTLP JSON Lines frame.
#[derive(Debug, thiserror::Error)]
pub enum FrameEncodeError {
    /// The encoded JSON document and newline exceed the configured frame limit.
    #[error("OTLP JSON frame exceeds the configured {max_frame_bytes} byte limit")]
    FrameTooLarge {
        /// Maximum allowed frame size, including the newline delimiter.
        max_frame_bytes: usize,
    },
    /// The pdata view could not be serialized as OTLP JSON.
    #[error(transparent)]
    Json(#[from] JsonEncodeError),
}

struct BoundedWriter<'a> {
    output: &'a mut Vec<u8>,
    max_document_bytes: usize,
    limit_exceeded: bool,
}

impl Write for BoundedWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.output.len().saturating_add(bytes.len()) > self.max_document_bytes {
            self.limit_exceeded = true;
            return Err(io::Error::other("OTLP JSON frame limit exceeded"));
        }
        self.output.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn encode_frame(
    output: &mut Vec<u8>,
    max_frame_bytes: usize,
    encode: impl FnOnce(&mut BoundedWriter<'_>) -> Result<(), JsonEncodeError>,
) -> Result<(), FrameEncodeError> {
    output.clear();
    let mut writer = BoundedWriter {
        output,
        max_document_bytes: max_frame_bytes.saturating_sub(1),
        limit_exceeded: false,
    };
    let result = encode(&mut writer);
    if writer.limit_exceeded {
        writer.output.clear();
        return Err(FrameEncodeError::FrameTooLarge { max_frame_bytes });
    }
    if let Err(error) = result {
        writer.output.clear();
        return Err(FrameEncodeError::Json(error));
    }
    writer.output.push(b'\n');
    Ok(())
}

/// Encodes one logs view as a bounded OTLP JSON Lines frame.
pub fn encode_logs<L: LogsDataView>(
    logs: &L,
    output: &mut Vec<u8>,
    max_frame_bytes: usize,
) -> Result<(), FrameEncodeError> {
    encode_frame(output, max_frame_bytes, |writer| {
        write_logs_json(logs, writer)
    })
}

/// Encodes one metrics view as a bounded OTLP JSON Lines frame.
pub fn encode_metrics<M: MetricsView>(
    metrics: &M,
    output: &mut Vec<u8>,
    max_frame_bytes: usize,
) -> Result<(), FrameEncodeError> {
    encode_frame(output, max_frame_bytes, |writer| {
        write_metrics_json(metrics, writer)
    })
}

/// Encodes one traces view as a bounded OTLP JSON Lines frame.
pub fn encode_traces<T: TracesView>(
    traces: &T,
    output: &mut Vec<u8>,
    max_frame_bytes: usize,
) -> Result<(), FrameEncodeError> {
    encode_frame(output, max_frame_bytes, |writer| {
        write_traces_json(traces, writer)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_pdata::proto::opentelemetry::logs::v1::LogsData;

    /// Scenario: An encoded document and its newline exactly reach the configured frame limit.
    /// Guarantees: The frame bound includes the delimiter and permits an exact fit.
    #[test]
    fn exact_frame_limit_is_accepted() {
        let mut output = vec![b'x'];
        encode_logs(&LogsData::default(), &mut output, 3).unwrap();
        assert_eq!(output, b"{}\n");
    }

    /// Scenario: The JSON document fits but adding the required newline would exceed the limit.
    /// Guarantees: The encoder reports a typed limit error and removes every partial byte.
    #[test]
    fn frame_limit_counts_newline_and_clears_output() {
        let mut output = vec![b'x'];
        let error = encode_logs(&LogsData::default(), &mut output, 2).unwrap_err();
        assert!(matches!(error, FrameEncodeError::FrameTooLarge { .. }));
        assert!(output.is_empty());
    }
}
