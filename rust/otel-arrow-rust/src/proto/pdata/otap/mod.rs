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

//! This module contains traits and utilities for OTAP (OpenTelemetry Arrow Protocol) message types.

// Re-export the derive macros from otap-derive
pub use otap_derive::Message;
pub use otap_derive::MessageBuilder;

// Include the builders module
pub mod builders;

// Include tests
#[cfg(test)]
mod tests;

/// Message is a trait for OTAP protocol buffer message types.
/// It extends the functionality provided by prost::Message with
/// OTAP-specific methods.
pub trait Message: prost::Message {
    /// Returns the message type name as a string.
    fn message_type() -> &'static str;
    
    // Additional methods can be added here as needed
}

/// Utility functions for working with OTAP message types
pub mod utils {
    use super::Message;
    
    /// Returns the message type name for the given OTAP message type
    pub fn get_message_type<T: Message>() -> &'static str {
        T::message_type()
    }
    
    /// Helper function to convert an OTAP message to bytes
    pub fn to_bytes<T: Message>(msg: &T) -> Vec<u8> {
        let mut buf = Vec::new();
        msg.encode(&mut buf).expect("Failed to encode message");
        buf
    }
    
    /// Helper function to convert bytes to an OTAP message
    pub fn from_bytes<T: Message + Default>(bytes: &[u8]) -> Result<T, prost::DecodeError> {
        T::decode(bytes)
    }
}

// Re-export the builders for convenience
pub use builders::{
    BatchArrowRecordsBuilder,
    ArrowPayloadBuilder,
    BatchStatusBuilder,
};