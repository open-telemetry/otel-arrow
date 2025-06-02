// SPDX-License-Identifier: Apache-2.0

//! Traits and structs defining the shared (Send) version of receivers, processors, and exporters.

use serde_json::Value;

pub mod exporter;
pub mod processor;
pub mod receiver;

/// A factory for creating shared receivers.
#[derive(Clone)]
pub struct SharedReceiverFactory<PData> {
    /// The name of the receiver.
    pub name: &'static str,
    /// A function that creates a new shared receiver instance.
    pub create: fn(config: &Value) -> Box<dyn receiver::Receiver<PData>>,
}

/// A factory for creating shared processors.
#[derive(Clone)]
pub struct SharedProcessorFactory<PData> {
    /// The name of the processor.
    pub name: &'static str,
    /// A function that creates a new shared processor instance.
    pub create: fn(config: &Value) -> Box<dyn processor::Processor<PData>>,
}

/// A factory for creating shared exporter.
#[derive(Clone)]
pub struct SharedExporterFactory<PData> {
    /// The name of the receiver.
    pub name: &'static str,
    /// A function that creates a new shared exporter instance.
    pub create: fn(config: &Value) -> Box<dyn exporter::Exporter<PData>>,
}
