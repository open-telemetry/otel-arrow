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

/// Registers a pdata codec in the process-wide link-time registry.
///
/// The registration expression must be const evaluable. The macro hides the
/// distributed-slice implementation so extension crates do not need to depend
/// on or use `linkme` directly. Its function-like form deliberately keeps the
/// registration as a typed [`CodecRegistration`] builder expression instead of
/// duplicating that API in a macro-specific syntax. The explicit identifier
/// names the generated static in compiler and linker diagnostics.
///
/// Registry validation, including duplicate-name handling, remains outside the
/// macro so registration order never becomes an implicit precedence policy.
#[macro_export]
macro_rules! register_pdata_codec {
    ($name:ident, $registration:expr $(,)?) => {
        #[allow(unsafe_code)]
        #[$crate::__private::distributed_slice($crate::PDATA_CODEC_FACTORIES)]
        static $name: $crate::CodecRegistration = $registration;
    };
}

/// Implementation details used by exported macros.
#[doc(hidden)]
pub mod __private {
    pub use linkme::distributed_slice;
}

/// Built-in codec identities and implementations.
pub mod builtins {
    pub use crate::codecs::otlp::{OTLP_ENCODING, OtlpDecoder, OtlpEncoder};
}

/// Reusable codec extension conformance checks.
#[cfg(any(test, feature = "testing"))]
pub mod testing;
