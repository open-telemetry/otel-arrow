// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Standalone control-aware bounded channel primitives.
//!
//! This crate is intentionally not integrated into the engine runtime yet. It
//! provides a bounded, policy-aware control channel that can
//! reserve lifecycle delivery, batch completion traffic, and coalesce
//! best-effort control work.

mod core;
pub mod local;
pub mod shared;
mod types;

pub use types::{
    AckMsg, AdmissionClass, CompletionMsg, ConfigError, ControlChannelConfig, ControlChannelStats,
    ControlCmd, DrainIngressMsg, LifecycleSendResult, NackMsg, NodeControlEvent, Phase,
    ReceiverControlEvent, SendError, SendOutcome, ShutdownMsg, TrySendError,
};

#[cfg(test)]
mod tests;
