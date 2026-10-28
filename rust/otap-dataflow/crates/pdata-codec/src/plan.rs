// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroUsize;
use std::sync::Arc;

use crate::{CodecError, CodecRegistry, PdataEncoding, ResolvedCodec};

/// Representation-neutral policy applied to independently encoded output.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct EncodePolicy {
    /// Maximum encoded batch size, checked on matching-format forwarding and
    /// passed to encoders that can enforce it during encoding.
    pub max_encoded_size: Option<NonZeroUsize>,
}

/// Output codec and policy resolved once during node construction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EncodingPlan {
    codec: ResolvedCodec,
    policy: EncodePolicy,
}

impl EncodingPlan {
    /// Builds a plan from an already validated codec.
    pub fn new(codec: ResolvedCodec, policy: EncodePolicy) -> Result<Self, CodecError> {
        if !codec.can_encode() {
            return Err(CodecError::UnsupportedCodecOperation {
                encoding: codec.encoding().clone(),
                operation: crate::CodecOperation::Encode,
            });
        }
        Ok(Self { codec, policy })
    }

    /// Resolves an output identity once while constructing a node.
    pub fn resolve(
        registry: &CodecRegistry,
        encoding: &PdataEncoding,
        policy: EncodePolicy,
    ) -> Result<Self, CodecError> {
        Self::new(registry.resolve(encoding)?, policy)
    }

    /// Resolved output codec.
    #[must_use]
    pub const fn codec(self) -> ResolvedCodec {
        self.codec
    }

    /// Representation-neutral output policy.
    #[must_use]
    pub const fn policy(self) -> EncodePolicy {
        self.policy
    }

    /// Checks forwarded bytes without allocating or instantiating an encoder.
    pub(crate) fn validate_encoded_size(self, actual: usize) -> Result<(), CodecError> {
        if let Some(limit) = self.policy.max_encoded_size
            && actual > limit.get()
        {
            return Err(CodecError::EncodedSizeLimitExceeded {
                encoding: self.codec.encoding().clone(),
                actual,
                limit: limit.get(),
            });
        }
        Ok(())
    }
}

/// Resolved encodings a read-only consumer accepts for direct borrowed access.
///
/// This set does not restrict which input formats may reach the consumer. When
/// obtaining a [`crate::PdataView`], listed encodings are borrowed without decoding;
/// other encodings fall back to decoding into native OTAP. Already-native OTAP is
/// always borrowed. An empty set therefore requires native OTAP access, rather
/// than rejecting all encoded input. Borrowing encoded bytes does not validate them.
#[derive(Clone, Debug, Default)]
pub struct AcceptedEncodings {
    accepted: Arc<[ResolvedCodec]>,
}

impl AcceptedEncodings {
    /// Accepts no encodings directly, decoding encoded input to native OTAP on demand.
    #[must_use]
    pub fn native() -> Self {
        Self::default()
    }

    /// Accepts the listed encodings directly; other encodings fall back to native OTAP.
    #[must_use]
    pub fn accept_encoded(codecs: impl IntoIterator<Item = ResolvedCodec>) -> Self {
        Self {
            accepted: codecs.into_iter().collect::<Vec<_>>().into(),
        }
    }

    pub(crate) fn accepts(&self, codec: ResolvedCodec) -> bool {
        self.accepted.contains(&codec)
    }
}
