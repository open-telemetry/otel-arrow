// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline-local lifecycle and access for mutable codec implementations.
//!
//! [`CodecServiceBuilder`] combines an immutable validated registry with fresh
//! runtime state. Cloned [`CodecService`] handles share that state within one
//! pipeline, while decoder and encoder instances are created lazily and reused.
//! Payload admission and matching-format forwarding therefore require neither
//! codec construction nor mutable runtime access.
//!
//! Codec trait calls are currently synchronous. The service holds its runtime
//! lock for the duration of each codec operation and, for prepared output, for
//! the synchronous consumer callback. No codec borrow crosses an async
//! suspension. [`EncodeOutput::into_bytes`] detaches owned bytes before an
//! asynchronous transport send, but it does not make encoding asynchronous.
//! A codec that performs slow or blocking work would still block its caller;
//! bounded offloading is intentionally left to a future execution layer at the
//! engine service boundary.

use std::sync::{Arc, Mutex, MutexGuard};

use bytes::Bytes;
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers};

use crate::{
    CodecError, CodecRegistry, EncodeOutput, EncodedPdata, EncodingPlan, PdataDecoder,
    PdataEncoder, PdataView, RegistryError, ResolvedCodec, ViewPlan,
};

struct DecoderInstance {
    codec: ResolvedCodec,
    decoder: Box<dyn PdataDecoder>,
}

struct EncoderInstance {
    plan: EncodingPlan,
    encoder: Box<dyn PdataEncoder>,
}

#[derive(Default)]
struct CodecRuntime {
    decoders: Vec<DecoderInstance>,
    encoders: Vec<EncoderInstance>,
}

/// Builds a pipeline-local codec service from a validated registry.
pub struct CodecServiceBuilder {
    registry: Arc<CodecRegistry>,
}

impl CodecServiceBuilder {
    /// Validates and selects the process-wide link-time registry.
    pub fn from_global_registry() -> Result<Self, RegistryError> {
        Ok(Self {
            registry: CodecRegistry::global()?,
        })
    }

    /// Selects an already validated registry.
    #[must_use]
    pub fn from_registry(registry: Arc<CodecRegistry>) -> Self {
        Self { registry }
    }

    /// Creates fresh lazy mutable state for one pipeline runtime.
    #[must_use]
    pub fn build(self) -> CodecService {
        CodecService {
            registry: self.registry,
            runtime: Arc::new(Mutex::new(CodecRuntime::default())),
        }
    }
}

/// Scoped access to mutable codec instances owned by one pipeline runtime.
///
/// Codec operations are synchronous and release the runtime lock before
/// returning. Callers must detach prepared output before awaiting asynchronous
/// work. The module-level documentation describes the execution boundary.
#[derive(Clone)]
pub struct CodecService {
    registry: Arc<CodecRegistry>,
    runtime: Arc<Mutex<CodecRuntime>>,
}

impl CodecService {
    /// Creates a service after validating the process registry.
    pub fn new() -> Result<Self, RegistryError> {
        Ok(CodecServiceBuilder::from_global_registry()?.build())
    }

    /// Validated immutable registry used by this pipeline.
    #[must_use]
    pub fn registry(&self) -> &CodecRegistry {
        &self.registry
    }

    fn lock(&self) -> MutexGuard<'_, CodecRuntime> {
        self.runtime
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Decodes admitted bytes through a lazily reused decoder instance.
    pub fn decode(&self, encoded: &EncodedPdata) -> Result<OtapArrowRecords, CodecError> {
        let mut runtime = self.lock();
        let records = runtime
            .decoder(encoded.codec())?
            .decode(encoded.signal_type(), encoded.bytes())?;
        if records.signal_type() != encoded.signal_type() {
            return Err(CodecError::SignalChanged {
                encoding: encoded.encoding().clone(),
                expected: encoded.signal_type(),
                actual: records.signal_type(),
            });
        }
        Ok(records)
    }

    /// Returns encoded bytes when accepted, otherwise decoded native records.
    pub fn view<'a>(
        &self,
        encoded: &'a EncodedPdata,
        plan: &ViewPlan,
    ) -> Result<PdataView<'a>, CodecError> {
        if plan.accepts(encoded.codec()) {
            return Ok(PdataView::Encoded(crate::EncodedView::new(
                encoded.encoding(),
                encoded.signal_type(),
                encoded.bytes(),
            )));
        }
        self.decode(encoded)
            .map(|records| PdataView::Native(std::borrow::Cow::Owned(records)))
    }

    /// Runs a synchronous consumer while prepared output may borrow scratch.
    pub fn with_encoded_output<R>(
        &self,
        records: &mut OtapArrowRecords,
        plan: &EncodingPlan,
        consume: impl FnOnce(EncodeOutput<'_>) -> R,
    ) -> Result<R, CodecError> {
        let mut runtime = self.lock();
        let output = runtime.encoder(*plan)?.prepare_encode(records)?;
        Ok(consume(output))
    }

    /// Detaches owned bytes before a caller performs an asynchronous send.
    pub fn encode_bytes(
        &self,
        records: &mut OtapArrowRecords,
        plan: &EncodingPlan,
    ) -> Result<Bytes, CodecError> {
        self.with_encoded_output(records, plan, |output| output.into_bytes())
    }

    /// Returns whether two handles address the same pipeline-owned state.
    #[must_use]
    pub fn shares_state_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.runtime, &other.runtime)
    }

    /// Number of lazily created mutable instances.
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn test_instance_count(&self) -> usize {
        let runtime = self.lock();
        runtime.decoders.len() + runtime.encoders.len()
    }
}

impl CodecRuntime {
    fn decoder(&mut self, codec: ResolvedCodec) -> Result<&mut dyn PdataDecoder, CodecError> {
        let index = match self
            .decoders
            .iter()
            .position(|instance| instance.codec == codec)
        {
            Some(index) => index,
            None => {
                let index = self.decoders.len();
                self.decoders.push(DecoderInstance {
                    codec,
                    decoder: codec.create_decoder()?,
                });
                index
            }
        };
        Ok(self.decoders[index].decoder.as_mut())
    }

    fn encoder(&mut self, plan: EncodingPlan) -> Result<&mut dyn PdataEncoder, CodecError> {
        let index = match self
            .encoders
            .iter()
            .position(|instance| instance.plan == plan)
        {
            Some(index) => index,
            None => {
                let index = self.encoders.len();
                self.encoders.push(EncoderInstance {
                    plan,
                    encoder: plan.codec().create_encoder(plan.policy())?,
                });
                index
            }
        };
        Ok(self.encoders[index].encoder.as_mut())
    }
}
