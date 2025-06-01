// SPDX-License-Identifier: Apache-2.0

//! Traits and structs defining the local (!Send) version of receivers, processors, and exporters.

use serde_json::Value;

pub mod exporter;
pub mod processor;
pub mod receiver;

/// A factory for creating local receivers.
#[derive(Clone)]
pub struct LocalReceiverFactory<PData> {
    /// The name of the receiver.
    pub name: &'static str,
    /// A function that creates a new local receiver instance.
    pub create: fn(config: &Value) -> Box<dyn receiver::Receiver<PData>>,
}

/// A factory for creating local processors.
#[derive(Clone)]
pub struct LocalProcessorFactory<PData> {
    /// The name of the processor.
    pub name: &'static str,
    /// A function that creates a new local processor instance.
    pub create: fn(config: &Value) -> Box<dyn processor::Processor<PData>>,
}

/// A factory for creating local exporter.
#[derive(Clone)]
pub struct LocalExporterFactory<PData> {
    /// The name of the receiver.
    pub name: &'static str,
    /// A function that creates a new local exporter instance.
    pub create: fn(config: &Value) -> Box<dyn exporter::Exporter<PData>>,
}
