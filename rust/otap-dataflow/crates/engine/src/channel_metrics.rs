// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Channel-oriented metrics for the OTAP engine.
//!
//! Metrics are split by channel type (control vs pdata) and by endpoint
//! role (sender vs receiver). All metrics are scoped using channel endpoint
//! attributes and can be correlated using `channel.id`.

use otap_df_telemetry::instrument::{Counter, Gauge, ObserveUpDownCounter};
use otap_df_telemetry_macros::metric_set;

#[metric_set(name = "control.channel.sender")]
#[derive(Debug, Default, Clone)]
pub struct ControlSenderMetrics {
    /// Count of messages successfully sent to the channel.
    #[metric(name = "send.count", unit = "{message}")]
    pub send_count: Counter<u64>,
    /// Total time spent awaiting send capacity during the interval.
    #[metric(name = "send.wait_time", unit = "{s}")]
    pub send_wait_time: Counter<f64>,
    /// Count of send failures due to a full channel.
    #[metric(name = "send.error_full", unit = "{1}")]
    pub send_error_full: Counter<u64>,
    /// Count of send failures due to a closed channel.
    #[metric(name = "send.error_closed", unit = "{1}")]
    pub send_error_closed: Counter<u64>,
    /// Total bytes successfully sent (when message size is known).
    #[metric(name = "send.bytes", unit = "{By}")]
    pub send_bytes: Counter<u64>,
    /// Count of messages dropped (if overflow policy drops).
    #[metric(name = "drop.count", unit = "{message}")]
    pub drop_count: Counter<u64>,
}

#[metric_set(name = "control.channel.receiver")]
#[derive(Debug, Default, Clone)]
pub struct ControlReceiverMetrics {
    /// Count of messages successfully received from the channel.
    #[metric(name = "recv.count", unit = "{message}")]
    pub recv_count: Counter<u64>,
    /// Total time spent awaiting messages during the interval.
    #[metric(name = "recv.wait_time", unit = "{s}")]
    pub recv_wait_time: Counter<f64>,
    /// Count of receive attempts when the channel was empty.
    #[metric(name = "recv.error_empty", unit = "{1}")]
    pub recv_error_empty: Counter<u64>,
    /// Count of receive attempts after the channel was closed.
    #[metric(name = "recv.error_closed", unit = "{1}")]
    pub recv_error_closed: Counter<u64>,
    /// Total bytes successfully received (when message size is known).
    #[metric(name = "recv.bytes", unit = "{By}")]
    pub recv_bytes: Counter<u64>,
    /// Current number of buffered messages.
    #[metric(name = "queue.depth", unit = "{message}")]
    pub queue_depth: Gauge<u64>,
    /// Maximum channel capacity (buffer size).
    #[metric(name = "capacity", unit = "{message}")]
    pub capacity: Gauge<u64>,
    /// Count of channel close events.
    #[metric(name = "close.count", unit = "{1}")]
    pub close_count: Counter<u64>,
}

#[metric_set(name = "pdata.channel.sender")]
#[derive(Debug, Default, Clone)]
pub struct PdataSenderMetrics {
    /// Count of messages successfully sent to the channel.
    #[metric(name = "send.count", unit = "{message}")]
    pub send_count: Counter<u64>,
    /// Total time spent awaiting send capacity during the interval.
    #[metric(name = "send.wait_time", unit = "{s}")]
    pub send_wait_time: Counter<f64>,
    /// Count of send failures due to a full channel.
    #[metric(name = "send.error_full", unit = "{1}")]
    pub send_error_full: Counter<u64>,
    /// Count of send failures due to a closed channel.
    #[metric(name = "send.error_closed", unit = "{1}")]
    pub send_error_closed: Counter<u64>,
    /// Total bytes successfully sent (when message size is known).
    #[metric(name = "send.bytes", unit = "{By}")]
    pub send_bytes: Counter<u64>,
    /// Count of messages dropped (if overflow policy drops).
    #[metric(name = "drop.count", unit = "{message}")]
    pub drop_count: Counter<u64>,
}

#[metric_set(name = "pdata.channel.receiver")]
#[derive(Debug, Default, Clone)]
pub struct PdataReceiverMetrics {
    /// Count of messages successfully received from the channel.
    #[metric(name = "recv.count", unit = "{message}")]
    pub recv_count: Counter<u64>,
    /// Total time spent awaiting messages during the interval.
    #[metric(name = "recv.wait_time", unit = "{s}")]
    pub recv_wait_time: Counter<f64>,
    /// Count of receive attempts when the channel was empty.
    #[metric(name = "recv.error_empty", unit = "{1}")]
    pub recv_error_empty: Counter<u64>,
    /// Count of receive attempts after the channel was closed.
    #[metric(name = "recv.error_closed", unit = "{1}")]
    pub recv_error_closed: Counter<u64>,
    /// Total bytes successfully received (when message size is known).
    #[metric(name = "recv.bytes", unit = "{By}")]
    pub recv_bytes: Counter<u64>,
    /// Current number of buffered messages.
    #[metric(name = "queue.depth", unit = "{message}")]
    pub queue_depth: Gauge<u64>,
    /// Maximum channel capacity (buffer size).
    #[metric(name = "capacity", unit = "{message}")]
    pub capacity: Gauge<u64>,
    /// Count of channel close events.
    #[metric(name = "close.count", unit = "{1}")]
    pub close_count: Counter<u64>,
}
