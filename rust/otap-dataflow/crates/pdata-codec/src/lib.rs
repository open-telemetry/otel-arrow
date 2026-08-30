// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pluggable codecs between independently decodable bytes and native OTAP.

mod decoder;
mod encoder;
mod error;
mod identity;
mod plan;
mod registry;
mod runtime;
mod view;

mod codecs;

pub use decoder::PdataDecoder;
pub use encoder::{EncodeOutput, PdataEncoder};
pub use error::{CodecError, CodecOperation, RegistryError};
pub use identity::{EncodedPdata, PdataEncoding};
pub use plan::{EncodePolicy, EncodingPlan, ViewPlan};
pub use registry::{
    CodecMetadata, CodecRegistration, CodecRegistry, ItemCounter, PDATA_CODEC_FACTORIES,
    ResolvedCodec,
};
pub use runtime::{CodecService, CodecServiceBuilder};
pub use view::{EncodedView, PdataView};

/// Built-in codec identities and implementations.
pub mod builtins {
    pub use crate::codecs::otlp::{OTLP_ENCODING, OtlpDecoder, OtlpEncoder};
}

/// Reusable codec extension conformance checks.
#[cfg(any(test, feature = "testing"))]
pub mod testing;
