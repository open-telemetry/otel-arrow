// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Experimental control-aware bounded channel primitives.
//!
//! This crate is intentionally not integrated into the engine runtime yet. It
//! exists to prototype a bounded, policy-aware control channel that can
//! reserve lifecycle delivery, batch completion traffic, and coalesce
//! best-effort control work.

mod core;
pub mod local;
pub mod shared;
mod types;

pub use types::{
    AckMsg, CompletionMsg, ConfigError, ControlChannelConfig, ControlChannelStats, ControlClass,
    ControlCmd, ControlEvent, DelayedDataMsg, DrainIngressMsg, NackMsg, Phase, SendError,
    SendOutcome, ShutdownMsg, TelemetrySourceId, TimerSourceId,
};

#[cfg(test)]
mod tests;
