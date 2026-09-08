// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline-local lifecycle and access for mutable codec implementations.
//!
//! [`CodecServiceBuilder`] combines an immutable validated registry and decode
//! policy with fresh runtime state. Cloned [`CodecService`] handles share that
//! state within one pipeline, while decoder and encoder instances are created
//! lazily and reused. The decode policy is applied once at decoder creation;
//! payload admission and matching-format forwarding therefore require neither
//! codec construction nor mutable runtime access.
//!
//! Codec trait calls are currently synchronous. The service holds its runtime
//! lock for the duration of each codec operation and, for prepared output, for
//! the synchronous consumer callback. Nested access from that callback returns
//! an error instead of blocking on the same lock. A panic while state is
//! borrowed poisons the service; later operations report that state as
//! unavailable instead of continuing with possibly corrupted codec state. No
//! codec borrow crosses an async suspension. [`EncodeOutput::into_bytes`]
//! detaches owned bytes before an asynchronous transport send, but it does not
//! make encoding asynchronous.
//! A codec that performs slow or blocking work would still block its caller;
//! bounded offloading is intentionally left to a future execution layer at the
//! engine service boundary.

use std::sync::{Arc, Mutex, MutexGuard, TryLockError};

use bytes::Bytes;
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers};

use crate::{
    CodecError, CodecOperation, CodecRegistry, DecodePolicy, EncodeOutput, EncodedPdata,
    EncodingPlan, InspectionPlan, PdataDecoder, PdataEncoder, PdataView, RegistryError,
    ResolvedCodec,
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
    decode_policy: DecodePolicy,
}

impl CodecServiceBuilder {
    /// Validates and selects the process-wide link-time registry.
    pub fn from_global_registry() -> Result<Self, RegistryError> {
        Ok(Self {
            registry: CodecRegistry::global()?,
            decode_policy: DecodePolicy::default(),
        })
    }

    /// Selects an already validated registry.
    #[must_use]
    pub fn from_registry(registry: Arc<CodecRegistry>) -> Self {
        Self {
            registry,
            decode_policy: DecodePolicy::default(),
        }
    }

    /// Selects the policy supplied to every decoder created by this service.
    #[must_use]
    pub const fn with_decode_policy(mut self, decode_policy: DecodePolicy) -> Self {
        self.decode_policy = decode_policy;
        self
    }

    /// Creates fresh lazy mutable state for one pipeline runtime.
    #[must_use]
    pub fn build(self) -> CodecService {
        CodecService {
            registry: self.registry,
            decode_policy: self.decode_policy,
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
    decode_policy: DecodePolicy,
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

    /// Decode policy resolved for this pipeline-local service.
    #[must_use]
    pub const fn decode_policy(&self) -> DecodePolicy {
        self.decode_policy
    }

    fn lock(&self) -> Result<MutexGuard<'_, CodecRuntime>, CodecError> {
        match self.runtime.try_lock() {
            Ok(runtime) => Ok(runtime),
            Err(TryLockError::WouldBlock) => Err(CodecError::ServiceBusy),
            Err(TryLockError::Poisoned(_)) => Err(CodecError::ServicePoisoned),
        }
    }

    /// Decodes admitted bytes through a lazily reused decoder instance.
    pub fn decode(&self, encoded: &EncodedPdata) -> Result<OtapArrowRecords, CodecError> {
        let mut runtime = self.lock()?;
        let records = runtime
            .decoder(encoded.codec(), self.decode_policy)?
            .decode(encoded.signal_type(), encoded.bytes())
            .map_err(|error| {
                error.with_operation_context(encoded.encoding(), CodecOperation::Decode)
            })?;
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
        plan: &InspectionPlan,
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
        let codec = plan.codec();
        codec.require_encoder(records.signal_type())?;
        let mut runtime = self.lock()?;
        let encoder = runtime.encoder(*plan).map_err(|error| {
            error.with_operation_context(codec.encoding(), CodecOperation::Encode)
        })?;
        let output = encoder.prepare_encode(records).map_err(|error| {
            error.with_operation_context(codec.encoding(), CodecOperation::Encode)
        })?;
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
    pub fn test_instance_count(&self) -> Result<usize, CodecError> {
        let runtime = self.lock()?;
        Ok(runtime.decoders.len() + runtime.encoders.len())
    }
}

impl CodecRuntime {
    fn decoder(
        &mut self,
        codec: ResolvedCodec,
        policy: DecodePolicy,
    ) -> Result<&mut dyn PdataDecoder, CodecError> {
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
                    decoder: codec.create_decoder(policy)?,
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

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::Arc;

    use prost::Message;

    use super::*;
    use crate::{CodecMetadata, CodecRegistration, EncodePolicy, PdataEncoding};
    use otel_arrow_dfe_config::SignalType;
    use otel_arrow_dfe_pdata::testing::fixtures::{
        logs_with_full_resource_and_scope, metrics_sum_with_full_resource_and_scope,
    };

    const LOGS_ONLY_ENCODING: PdataEncoding = PdataEncoding::new("logs-only-test-v1");
    static LOGS_ONLY_METADATA: CodecMetadata =
        CodecMetadata::new(LOGS_ONLY_ENCODING, &[SignalType::Logs]);
    static LOGS_ONLY_REGISTRATIONS: [CodecRegistration; 1] =
        [CodecRegistration::new(&LOGS_ONLY_METADATA)
            .with_decoder(|_| Box::new(FailingDecoder))
            .with_encoder(|_| Ok(Box::new(PanicEncoder)))
            .with_item_counter(|_, _| Some(0))];

    struct FailingDecoder;

    impl PdataDecoder for FailingDecoder {
        fn decode(
            &mut self,
            _signal: SignalType,
            _bytes: &Bytes,
        ) -> Result<OtapArrowRecords, CodecError> {
            Err(CodecError::ServiceBusy)
        }
    }

    struct PanicEncoder;

    impl PdataEncoder for PanicEncoder {
        fn encode(&mut self, _records: OtapArrowRecords) -> Result<Bytes, CodecError> {
            panic!("signal validation must run before this test encoder")
        }
    }

    fn service_for(registrations: &'static [CodecRegistration]) -> CodecService {
        let registry = CodecRegistry::validate(registrations).expect("valid test registry");
        CodecServiceBuilder::from_registry(Arc::new(registry)).build()
    }

    fn records(signal: SignalType) -> OtapArrowRecords {
        let bytes: Bytes = match signal {
            SignalType::Logs => logs_with_full_resource_and_scope().encode_to_vec().into(),
            SignalType::Metrics => metrics_sum_with_full_resource_and_scope()
                .encode_to_vec()
                .into(),
            SignalType::Traces => unreachable!("runtime service tests do not request traces"),
        };
        let service = CodecService::new().expect("valid built-in registry");
        let codec = service
            .registry()
            .resolve_decoder(&PdataEncoding::OTLP, signal)
            .expect("built-in OTLP decoder");
        let encoded = codec.admit(signal, bytes).expect("supported OTLP signal");
        service.decode(&encoded).expect("valid OTLP fixture")
    }

    fn otlp_plan(service: &CodecService) -> EncodingPlan {
        EncodingPlan::resolve(
            service.registry(),
            &PdataEncoding::OTLP,
            EncodePolicy::default(),
        )
        .expect("built-in OTLP encoder")
    }

    /// Scenario: A logs-only output plan receives native metrics records.
    /// Guarantees: Runtime signal validation rejects the payload before creating or invoking the encoder.
    #[test]
    fn encoding_plan_validates_the_actual_records_signal() {
        let service = service_for(&LOGS_ONLY_REGISTRATIONS);
        let plan = EncodingPlan::resolve(
            service.registry(),
            &LOGS_ONLY_ENCODING,
            EncodePolicy::default(),
        )
        .unwrap();
        let mut metrics = records(SignalType::Metrics);

        assert!(matches!(
            service.encode_bytes(&mut metrics, &plan),
            Err(CodecError::Unsupported {
                operation: CodecOperation::Encode,
                signal: SignalType::Metrics,
                ..
            })
        ));
        assert_eq!(service.test_instance_count().unwrap(), 0);
    }

    /// Scenario: A decoder returns an error without the resolved codec identity.
    /// Guarantees: The codec service adds the actual identity and operation exactly once.
    #[test]
    fn decoder_failures_receive_service_owned_context() {
        let service = service_for(&LOGS_ONLY_REGISTRATIONS);
        let codec = service
            .registry()
            .resolve_decoder(&LOGS_ONLY_ENCODING, SignalType::Logs)
            .unwrap();
        let encoded = codec.admit(SignalType::Logs, Bytes::new()).unwrap();

        assert!(matches!(
            service.decode(&encoded),
            Err(CodecError::Operation {
                encoding,
                operation: CodecOperation::Decode,
                ..
            }) if encoding == LOGS_ONLY_ENCODING
        ));
    }

    /// Scenario: A prepared-output callback attempts another operation on the same service.
    /// Guarantees: Nested codec access returns an error instead of deadlocking the pipeline runtime.
    #[test]
    fn prepared_output_callback_cannot_reenter_the_service() {
        let service = CodecService::new().unwrap();
        let plan = otlp_plan(&service);
        let mut outer = records(SignalType::Logs);
        let mut nested = records(SignalType::Logs);

        let nested_result = service
            .with_encoded_output(&mut outer, &plan, |_| {
                service.encode_bytes(&mut nested, &plan)
            })
            .unwrap();

        assert!(matches!(nested_result, Err(CodecError::ServiceBusy)));
    }

    /// Scenario: A prepared-output consumer panics while holding codec runtime state.
    /// Guarantees: Later operations report poisoned state instead of reusing potentially corrupted buffers.
    #[test]
    fn poisoned_codec_state_is_not_reused() {
        let service = CodecService::new().unwrap();
        let plan = otlp_plan(&service);
        let mut records = records(SignalType::Logs);

        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _ = service.with_encoded_output(&mut records, &plan, |_| -> () {
                panic!("poison codec service for the test")
            });
        }));
        assert!(panic.is_err());

        assert!(matches!(
            service.encode_bytes(&mut records, &plan),
            Err(CodecError::ServicePoisoned)
        ));
    }
}
