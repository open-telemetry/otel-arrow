// SPDX-License-Identifier: Apache-2.0

//! Async Pipeline Engine

use serde_json::Value;

use crate::{
    config::{ExporterConfig, ProcessorConfig, ReceiverConfig},
    exporter::ExporterWrapper,
    processor::ProcessorWrapper,
    receiver::ReceiverWrapper,
};

pub mod error;
pub mod exporter;
pub mod message;
pub mod processor;
pub mod receiver;

pub mod config;
mod effect_handler;
pub mod local;
pub mod pipeline;
pub mod runtime_config;
pub mod shared;

pub mod testing;

/// A factory for creating receivers.
#[derive(Clone)]
pub struct ReceiverFactory<PData> {
    /// The name of the receiver.
    pub name: &'static str,
    /// A function that creates a new receiver instance.
    pub create: fn(config: &Value, receiver_config: &ReceiverConfig) -> ReceiverWrapper<PData>,
}

/// A factory for creating processors.
#[derive(Clone)]
pub struct ProcessorFactory<PData> {
    /// The name of the processor.
    pub name: &'static str,
    /// A function that creates a new processor instance.
    pub create: fn(config: &Value, processor_config: &ProcessorConfig) -> ProcessorWrapper<PData>,
}

/// A factory for creating exporter.
#[derive(Clone)]
pub struct ExporterFactory<PData> {
    /// The name of the receiver.
    pub name: &'static str,
    /// A function that creates a new exporter instance.
    pub create: fn(config: &Value, exporter_config: &ExporterConfig) -> ExporterWrapper<PData>,
}
