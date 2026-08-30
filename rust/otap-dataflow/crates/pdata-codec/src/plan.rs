// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroUsize;
use std::sync::Arc;

use otel_arrow_dfe_config::SignalType;

use crate::{CodecError, CodecRegistry, PdataEncoding, ResolvedCodec};

/// Representation-neutral policy applied to independently encoded output.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct EncodePolicy {
    /// Maximum encoded batch size when a codec can enforce it directly.
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
    pub fn new(
        codec: ResolvedCodec,
        signal: SignalType,
        policy: EncodePolicy,
    ) -> Result<Self, CodecError> {
        codec.require_encoder(signal)?;
        Ok(Self { codec, policy })
    }

    /// Resolves an output identity once while constructing a node.
    pub fn resolve(
        registry: &CodecRegistry,
        encoding: &PdataEncoding,
        signal: SignalType,
        policy: EncodePolicy,
    ) -> Result<Self, CodecError> {
        Self::new(registry.resolve_encoder(encoding, signal)?, signal, policy)
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
}

/// Encoded representations a read-only consumer can inspect directly.
#[derive(Clone, Debug, Default)]
pub struct ViewPlan {
    accepted: Arc<[ResolvedCodec]>,
}

impl ViewPlan {
    /// Requires native OTAP, decoding encoded input on demand.
    #[must_use]
    pub fn native() -> Self {
        Self::default()
    }

    /// Accepts the listed encoded representations without materialization.
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
