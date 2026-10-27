// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline-local lifecycle and access for mutable codec implementations.
//!
//! [`CodecServiceBuilder`] resolves optional per-encoding bindings against a
//! validated registry and prepares immutable factories with fresh runtime state. Cloned [`CodecService`] handles share that
//! pipeline, while decoder, encoder, and batcher instances are created lazily
//! and reused. Each selected decode policy is applied at decoder creation; payload
//! admission and matching-format forwarding therefore require neither codec
//! construction nor mutable runtime access.
//!
//! Synchronous codec operations hold the service runtime lock for their
//! duration and, for prepared output, for
//! the synchronous consumer callback. Nested access from that callback returns
//! an error instead of blocking on the same lock. A panic while state is
//! borrowed poisons the service; later operations report that state as
//! unavailable instead of continuing with possibly corrupted codec state. No
//! codec borrow crosses an async suspension. [`EncodeOutput::into_bytes`]
//! detaches owned bytes before an asynchronous transport send, but it does not
//! make encoding asynchronous.
//! [`CodecService::decode_cooperative`] starts a synchronous decode or an owned
//! incremental job under the lock, then releases the lock before driving that
//! job. The engine supplies scheduler-aware yielding between work segments.
//! There is no worker thread, spawned conversion task, or hidden pending queue.
//! Ordinary decoders still execute synchronously; bounded offloading for work
//! that cannot cooperate requires a separate execution policy.

use std::future::Future;
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};

use bytes::Bytes;
use otel_arrow_dfe_config::codecs::{
    CodecConfiguration, DecodeValidation as ConfigValidation, ResolvedCodecsConfig,
};
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers};

use crate::{
    AcceptedEncodings, BatchSettings, CodecBatches, CodecError, CodecOperation, CodecRegistry,
    DecodeBudget, DecodePolicy, DecodeTask, EncodeOutput, EncodedPdata, EncodingPlan, PdataBatcher,
    PdataDecoder, PdataEncoder, PdataView, RegistryError, ResolvedCodec,
};

struct PreparedCodec {
    slot: usize,
    codec: ResolvedCodec,
    policy: DecodePolicy,
    factory: Arc<dyn crate::PreparedCodecFactory>,
    forwarding: bool,
}

struct EncoderInstance {
    policy: crate::EncodePolicy,
    encoder: Box<dyn PdataEncoder>,
}

#[derive(Default)]
struct CodecInstances {
    decoder: Option<Box<dyn PdataDecoder>>,
    encoders: Vec<EncoderInstance>,
    batcher: Option<Box<dyn PdataBatcher>>,
}

struct CodecRuntime {
    instances: Vec<CodecInstances>,
}

/// Builds a pipeline-local codec service from a validated registry.
pub struct CodecServiceBuilder {
    registry: Arc<CodecRegistry>,
    decode_policy: DecodePolicy,
    configuration: ResolvedCodecsConfig,
}

impl CodecServiceBuilder {
    /// Validates and selects the process-wide link-time registry.
    pub fn from_global_registry() -> Result<Self, RegistryError> {
        Ok(Self {
            registry: CodecRegistry::global()?,
            decode_policy: DecodePolicy::default(),
            configuration: ResolvedCodecsConfig::default(),
        })
    }

    /// Selects an already validated registry.
    #[must_use]
    pub fn from_registry(registry: Arc<CodecRegistry>) -> Self {
        Self {
            registry,
            decode_policy: DecodePolicy::default(),
            configuration: ResolvedCodecsConfig::default(),
        }
    }

    /// Sets a programmatic baseline for decoders; explicit encoding bindings take precedence.
    #[must_use]
    pub const fn with_decode_policy(mut self, decode_policy: DecodePolicy) -> Self {
        self.decode_policy = decode_policy;
        self
    }

    /// Supplies scope-resolved bindings. Build validates names, capabilities and options.
    #[must_use]
    pub fn with_configuration(mut self, configuration: ResolvedCodecsConfig) -> Self {
        self.configuration = configuration;
        self
    }

    /// Validates all configuration and prepares factories without creating mutable instances.
    pub fn build(self) -> Result<CodecService, CodecError> {
        let mut registry = (*self.registry).clone();
        // Even unused definitions must not hide typos. Preparing immutable resources
        // does not activate a decoder; selected named factories are reused below.
        let mut named = std::collections::BTreeMap::new();
        for (name, config) in &self.configuration.configurations {
            let id = config.implementation.as_deref().ok_or_else(|| {
                CodecError::configuration(format!(
                    "codecs.configurations.{name}: implementation is required"
                ))
            })?;
            let codec = registry.implementation(id)?;
            let prepared = prepare(codec, config, self.decode_policy).map_err(|error| {
                CodecError::configuration(format!("codecs.configurations.{name}: {error}"))
            })?;
            _ = named.insert(name.as_str(), prepared);
        }
        let bindings = self
            .configuration
            .effective_bindings()
            .map_err(CodecError::configuration)?;
        for (encoding, config) in &bindings {
            let encoding = crate::PdataEncoding::from(encoding.clone());
            let codec = match config.implementation.as_deref() {
                Some(id) => registry.implementation(id)?,
                None => registry.resolve(&encoding)?,
            };
            if codec.encoding() != &encoding {
                return Err(CodecError::configuration(format!(
                    "codecs.bindings.{encoding}: implementation `{}` supports `{}`",
                    codec.implementation(),
                    codec.encoding()
                )));
            }
            registry.select(codec);
        }
        let mut prepared = Vec::new();
        for candidate in registry.codecs() {
            let codec = registry.resolve(candidate.encoding())?;
            if prepared
                .iter()
                .any(|entry: &PreparedCodec| entry.codec.encoding() == codec.encoding())
            {
                continue;
            }
            let config = bindings
                .get(codec.encoding().as_str())
                .cloned()
                .unwrap_or_default();
            let mut entry = match self.configuration.bindings.get(codec.encoding().as_str()) {
                Some(otel_arrow_dfe_config::codecs::CodecBinding::Named(reference)) => {
                    let source = named
                        .get(reference.configuration.as_str())
                        .ok_or_else(|| CodecError::configuration("missing named configuration"))?;
                    PreparedCodec {
                        slot: prepared.len(),
                        codec,
                        policy: source.policy,
                        factory: source.factory.clone(),
                        forwarding: source.forwarding,
                    }
                }
                _ => prepare(codec, &config, self.decode_policy).map_err(|error| {
                    CodecError::configuration(format!(
                        "codecs.bindings.{}: {error}",
                        codec.encoding()
                    ))
                })?,
            };
            entry.slot = prepared.len();
            prepared.push(entry);
        }
        let instances = (0..prepared.len())
            .map(|_| CodecInstances::default())
            .collect();
        Ok(CodecService {
            registry: Arc::new(registry),
            prepared: prepared.into(),
            runtime: Arc::new(Mutex::new(CodecRuntime { instances })),
        })
    }
}

/// Scoped access to mutable codec instances owned by one pipeline runtime.
///
/// Synchronous operations release the runtime lock before returning. Incremental
/// decode jobs own their state and release service access before suspension.
/// Callers must detach prepared output before awaiting asynchronous work.
#[derive(Clone)]
pub struct CodecService {
    registry: Arc<CodecRegistry>,
    prepared: Arc<[PreparedCodec]>,
    runtime: Arc<Mutex<CodecRuntime>>,
}

impl CodecService {
    /// Creates a service after validating the process registry.
    pub fn new() -> Result<Self, CodecError> {
        CodecServiceBuilder::from_global_registry()?.build()
    }

    /// Validated immutable registry used by this pipeline.
    #[must_use]
    pub fn registry(&self) -> &CodecRegistry {
        &self.registry
    }

    fn prepared(&self, codec: ResolvedCodec) -> Result<&PreparedCodec, CodecError> {
        self.prepared
            .iter()
            .find(|entry| entry.codec == codec || entry.codec.encoding() == codec.encoding())
            .ok_or_else(|| {
                RegistryError::NotFound {
                    encoding: codec.encoding().clone(),
                }
                .into()
            })
    }

    /// Effective decode policy for one encoding, independent of the admission implementation.
    pub fn decode_policy_for(&self, codec: ResolvedCodec) -> Result<DecodePolicy, CodecError> {
        Ok(self.prepared(codec)?.policy)
    }

    /// Checks whether compatible bytes satisfy the selected implementation's output settings.
    pub(crate) fn allows_encoded_forwarding(
        &self,
        plan: &EncodingPlan,
        signal: otel_arrow_dfe_config::SignalType,
    ) -> Result<bool, CodecError> {
        let prepared = self.prepared(plan.codec())?;
        prepared.codec.require_encoder(signal)?;
        Ok(prepared.forwarding)
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
        self.decode_parts(encoded.codec(), encoded.signal_type(), encoded.bytes())
    }

    /// Decodes encoded parts without constructing an envelope.
    pub fn decode_parts(
        &self,
        codec: ResolvedCodec,
        signal: otel_arrow_dfe_config::SignalType,
        bytes: &Bytes,
    ) -> Result<OtapArrowRecords, CodecError> {
        let prepared = self.prepared(codec)?;
        let codec = prepared.codec;
        codec.require_decoder(signal)?;
        let mut runtime = self.lock()?;
        let records = runtime
            .decoder(prepared)?
            .decode(signal, bytes)
            .map_err(|error| {
                error.with_operation_context(codec.encoding(), CodecOperation::Decode)
            })?;
        if records.signal_type() != signal {
            return Err(CodecError::SignalChanged {
                encoding: codec.encoding().clone(),
                expected: signal,
                actual: records.signal_type(),
            });
        }
        Ok(records)
    }

    /// Decodes without keeping mutable service access across scheduling points.
    ///
    /// Ordinary codecs finish synchronously. Incremental jobs are driven by the
    /// caller future: there is no spawned task, worker thread, or hidden queue.
    /// Existing node concurrency and channel backpressure bound admitted jobs.
    /// The engine supplies its scheduler-aware yield operation: immediately
    /// self-waking can repeatedly poll a LocalSet without driving I/O or timers.
    pub async fn decode_cooperative<Y: Future<Output = ()>>(
        &self,
        encoded: &EncodedPdata,
        mut yield_to_runtime: impl FnMut() -> Y,
    ) -> Result<OtapArrowRecords, CodecError> {
        let prepared = self.prepared(encoded.codec())?;
        let codec = prepared.codec;
        let signal = encoded.signal_type();
        codec.require_decoder(signal)?;
        let task = {
            let mut runtime = self.lock()?;
            runtime
                .decoder(prepared)?
                .start_decode(signal, encoded.bytes())
                .map_err(|error| {
                    error.with_operation_context(codec.encoding(), CodecOperation::Decode)
                })?
        };
        let records = match task {
            DecodeTask::Ready(records) => records,
            DecodeTask::Incremental(mut job) => loop {
                match job.step(DecodeBudget::default()).map_err(|error| {
                    error.with_operation_context(codec.encoding(), CodecOperation::Decode)
                })? {
                    Some(records) => break records,
                    None => yield_to_runtime().await,
                }
            },
        };
        if records.signal_type() != signal {
            return Err(CodecError::SignalChanged {
                encoding: codec.encoding().clone(),
                expected: signal,
                actual: records.signal_type(),
            });
        }
        Ok(records)
    }

    /// Returns encoded bytes when accepted, otherwise decoded native records.
    pub fn view<'a>(
        &self,
        encoded: &'a EncodedPdata,
        accepted_encodings: &AcceptedEncodings,
    ) -> Result<PdataView<'a>, CodecError> {
        self.view_parts(
            encoded.codec(),
            encoded.signal_type(),
            encoded.bytes(),
            accepted_encodings,
        )
    }

    /// Views encoded parts without constructing an envelope.
    pub fn view_parts<'a>(
        &self,
        codec: ResolvedCodec,
        signal: otel_arrow_dfe_config::SignalType,
        bytes: &'a Bytes,
        accepted_encodings: &AcceptedEncodings,
    ) -> Result<PdataView<'a>, CodecError> {
        self.prepared(codec)?.codec.require_decoder(signal)?;
        if accepted_encodings.accepts(codec) {
            return Ok(PdataView::Encoded(crate::EncodedView::new(
                codec.encoding(),
                signal,
                bytes,
            )));
        }
        self.decode_parts(codec, signal, bytes)
            .map(|records| PdataView::Native(std::borrow::Cow::Owned(records)))
    }

    /// Runs a synchronous consumer while prepared output may borrow scratch.
    pub fn with_encoded_output<R>(
        &self,
        records: &mut OtapArrowRecords,
        plan: &EncodingPlan,
        consume: impl FnOnce(EncodeOutput<'_>) -> R,
    ) -> Result<R, CodecError> {
        let prepared = self.prepared(plan.codec())?;
        let codec = prepared.codec;
        codec.require_encoder(records.signal_type())?;
        let mut runtime = self.lock()?;
        let encoder = runtime.encoder(*plan, prepared).map_err(|error| {
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

    /// Re-batches independently decodable inputs with reused runtime-local state.
    pub fn batch(
        &self,
        codec: ResolvedCodec,
        signal: otel_arrow_dfe_config::SignalType,
        settings: &BatchSettings,
        inputs: Vec<Bytes>,
    ) -> Result<CodecBatches, CodecError> {
        let prepared = self.prepared(codec)?;
        let codec = prepared.codec;
        codec.require_batcher(signal)?;
        let mut runtime = self.lock()?;
        let batcher = runtime.batcher(prepared).map_err(|error| {
            error.with_operation_context(codec.encoding(), CodecOperation::Batch)
        })?;
        batcher
            .batch(signal, settings, inputs)
            .map_err(|error| error.with_operation_context(codec.encoding(), CodecOperation::Batch))
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
        Ok(runtime
            .instances
            .iter()
            .map(|entry| {
                usize::from(entry.decoder.is_some())
                    + entry.encoders.len()
                    + usize::from(entry.batcher.is_some())
            })
            .sum())
    }
}

fn prepare(
    codec: ResolvedCodec,
    config: &CodecConfiguration,
    fallback: DecodePolicy,
) -> Result<PreparedCodec, CodecError> {
    let policy = if let Some(decode) = &config.decode {
        if !codec.can_decode() {
            return Err(CodecError::configuration(
                "decode settings require a decoder",
            ));
        }
        DecodePolicy::new(match decode.validation {
            ConfigValidation::BestEffort => crate::DecodeValidation::BestEffort,
            ConfigValidation::Strict => crate::DecodeValidation::Strict,
        })
    } else {
        fallback
    };
    let factory = codec.prepare(config)?;
    let forwarding = factory.allows_encoded_forwarding();
    Ok(PreparedCodec {
        slot: 0,
        codec,
        policy,
        factory,
        forwarding,
    })
}

impl CodecRuntime {
    fn decoder(&mut self, prepared: &PreparedCodec) -> Result<&mut dyn PdataDecoder, CodecError> {
        let entry = &mut self.instances[prepared.slot];
        if entry.decoder.is_none() {
            entry.decoder = Some(
                prepared
                    .factory
                    .create_decoder(prepared.codec, prepared.policy)?,
            );
        }
        // SAFETY: The branch above either installs the decoder or returns its error.
        Ok(entry.decoder.as_deref_mut().expect("decoder initialized"))
    }

    fn encoder(
        &mut self,
        plan: EncodingPlan,
        prepared: &PreparedCodec,
    ) -> Result<&mut dyn PdataEncoder, CodecError> {
        let encoders = &mut self.instances[prepared.slot].encoders;
        let index = match encoders
            .iter()
            .position(|instance| instance.policy == plan.policy())
        {
            Some(index) => index,
            None => {
                let encoder = prepared
                    .factory
                    .create_encoder(prepared.codec, plan.policy())?;
                let index = encoders.len();
                encoders.push(EncoderInstance {
                    policy: plan.policy(),
                    encoder,
                });
                index
            }
        };
        Ok(encoders[index].encoder.as_mut())
    }

    fn batcher(&mut self, prepared: &PreparedCodec) -> Result<&mut dyn PdataBatcher, CodecError> {
        let entry = &mut self.instances[prepared.slot];
        if entry.batcher.is_none() {
            entry.batcher = Some(prepared.factory.create_batcher(prepared.codec)?);
        }
        // SAFETY: The branch above either installs the batcher or returns its error.
        Ok(entry.batcher.as_deref_mut().expect("batcher initialized"))
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
        CodecServiceBuilder::from_registry(Arc::new(registry))
            .build()
            .unwrap()
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
