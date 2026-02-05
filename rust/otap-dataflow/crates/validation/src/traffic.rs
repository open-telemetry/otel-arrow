// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module defines structs to describe the traffic being created and captured for validation

use serde::{Deserialize, Serialize};
/// Helps distinguish between the message types
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// otlp type
    Otlp,
    /// otap type
    Otap,
}

/// Configuration describing how the traffic generator should emit signals.
#[derive(Debug, Deserialize, Serialize)]
pub struct Generator {
    /// Type to use for the system-under-validation exporter.
    #[serde(default = "default_message_type")]
    pub suv_exporter_type: MessageType,
    /// Endpoint the system-under-validation exporter should target.
    #[serde(default = "default_suv_endpoint")]
    pub suv_endpoint: String,
    /// Type to use for the control path exporter.
    #[serde(default = "default_message_type")]
    pub control_exporter_type: MessageType,
    /// Endpoint the control exporter should target.
    #[serde(default = "default_control_endpoint")]
    pub control_endpoint: String,
    /// Maximum number of signals the load generator should emit.
    #[serde(default = "default_max_signal_count")]
    pub max_signal_count: usize,
    /// Maximum batch size emitted by the load generator.
    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: usize,
    /// Signal emission rate (per second) for the load generator.
    #[serde(default = "default_signals_per_second")]
    pub signals_per_second: usize,
}

/// Configuration describing how validation receivers capture generated traffic.
#[derive(Debug, Deserialize, Serialize)]
pub struct Capture {
    /// Type to use for the system-under-validation receiver.
    #[serde(default = "default_message_type")]
    pub suv_receiver_type: MessageType,
    /// Listening address for the system-under-validation receiver.
    #[serde(default = "default_suv_addr")]
    pub suv_listening_addr: String,
    /// Type to use for the control receiver.
    #[serde(default = "default_message_type")]
    pub control_receiver_type: MessageType,
    /// Listening address for the control receiver.
    #[serde(default = "default_control_addr")]
    pub control_listening_addr: String,
    /// Whether pipeline transformation is expected (flips validation expectation).
    #[serde(default = "default_transformative")]
    pub transformative: bool,
}

fn default_message_type() -> MessageType {
    MessageType::Otlp
}

fn default_suv_addr() -> String {
    "127.0.0.1:4318".to_string()
}

fn default_suv_endpoint() -> String {
    "http://127.0.0.1:4317".to_string()
}

fn default_max_signal_count() -> usize {
    2000
}

fn default_max_batch_size() -> usize {
    100
}

fn default_signals_per_second() -> usize {
    100
}

fn default_control_addr() -> String {
    "127.0.0.1:4316".to_string()
}

fn default_control_endpoint() -> String {
    "http://127.0.0.1:4316".to_string()
}

fn default_transformative() -> bool {
    false
}