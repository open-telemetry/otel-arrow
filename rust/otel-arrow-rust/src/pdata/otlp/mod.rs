// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module contains traits and utilities for OTLP (OpenTelemetry Protocol) message types.

// Re-export derive macro (required for generated code)
pub use otlp_derive::Message;

// Include tests
#[cfg(test)]
mod tests;

// Message trait to be implemented by future OTLP message types
/// Message is a trait for OTLP protocol buffer message types.
pub trait Message {
    /// The associated builder type for this message
    type Builder;

    /// Creates a new builder for this message type
    fn builder() -> Self::Builder;
}

// Shared traits for message builders
/// Trait for OTLP types that have a name field
pub trait HasName {
    /// Sets the name field
    fn with_name<S: AsRef<str>>(self, name: S) -> Self;
}

/// Trait for OTLP types that have a description field
pub trait HasDescription {
    /// Sets the description field
    fn with_description<S: AsRef<str>>(self, description: S) -> Self;
}

/// Trait for OTLP types that have attributes
pub trait HasAttributes {
    /// Sets the attributes field
    fn with_attributes(self, attributes: Vec<crate::proto::opentelemetry::common::v1::KeyValue>) -> Self;
}

/// Trait for OTLP types that have a start time field
pub trait HasStartTime {
    /// Sets the start time field (nanoseconds since Unix epoch)
    fn with_start_time_unix_nano(self, start_time: u64) -> Self;
}

/// Trait for OTLP types that have an end time field
pub trait HasEndTime {
    /// Sets the end time field (nanoseconds since Unix epoch)
    fn with_end_time_unix_nano(self, end_time: u64) -> Self;
}

/// Trait for OTLP types that have a timestamp field
pub trait HasTimeUnixNano {
    /// Sets the timestamp field (nanoseconds since Unix epoch)
    fn with_time_unix_nano(self, time: u64) -> Self;
}
