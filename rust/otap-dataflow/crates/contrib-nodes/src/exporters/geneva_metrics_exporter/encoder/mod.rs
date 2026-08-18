// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Geneva Metrics ingestion protocol version 6 packet model and encoder.

mod exemplar;
mod histogram;
mod model;
mod numeric;
mod packet;
mod writer;

pub use model::*;
pub use packet::encode;

#[cfg(test)]
mod test_support;
