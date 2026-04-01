// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Standalone control-aware bounded channel primitives.
//!
//! This crate is intentionally not integrated into the engine runtime yet. It
//! provides a bounded, policy-aware control channel that can
//! reserve lifecycle delivery, batch completion traffic, and coalesce
//! best-effort control work.

mod channel;
mod core;
mod types;

pub use channel::{
    NodeControlReceiver, NodeControlSender, ReceiverControlReceiver, ReceiverControlSender,
    node_channel, node_channel_with_meta, receiver_channel, receiver_channel_with_meta,
};
pub use types::{
    AckMsg, AdmissionClass, CompletionMsg, ConfigError, ControlChannelConfig, ControlChannelStats,
    ControlCmd, DrainIngressMsg, LifecycleSendResult, NackMsg, NodeControlEvent, Phase,
    ReceiverControlEvent, SendError, SendOutcome, ShutdownMsg, TrySendError,
};

#[cfg(test)]
mod tests;
