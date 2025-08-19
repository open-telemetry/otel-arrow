// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This crate contains the rust implementation of the OTEL Arrow Protocol.
//! It contains code for decoding OTAP Arrow record batches into OTLP protos,
//! and encoding OTLP protos into OTAP Messages. It also contains
//! the rust implementation of pdata.

#[allow(dead_code)]
pub(crate) mod arrays;
mod decode;

pub mod encode;
pub mod error;
pub mod otap;
pub mod otlp;
#[allow(dead_code)]
pub mod schema;
#[cfg(test)]
mod test_util;
#[cfg(test)]
mod validation;

pub mod pdata;
pub mod proto;

pub use decode::decoder::Consumer;
pub use encode::producer::Producer;

// Debug tests for EncodedLen visitor functionality
//pub mod debug_test;
