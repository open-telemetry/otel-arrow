// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::client::{MetricsPublisher, PublishError};
use super::encoder::EncodedPacket;
use super::otlp_to_geneva::Config;
use super::publication_preparation::prepare_publication;
use async_trait::async_trait;
use futures::future::LocalBoxFuture;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::ConsumerEffectHandlerExtension;
use otel_arrow_dfe_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otel_arrow_dfe_engine::error::Error as EngineError;
use otel_arrow_dfe_engine::local::capability::auth::bearer_token_provider::BearerTokenProvider as LocalBearerTokenProvider;
use otel_arrow_dfe_engine::local::exporter::{EffectHandler, Exporter};
use otel_arrow_dfe_engine::message::{ExporterInbox, Message};
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_otap::http_client_auth::{
    HttpClientAuthProvider, HttpClientAuthProviderEvents,
    new_http_client_auth_provider_from_bearer_token_provider,
};
use otel_arrow_dfe_otap::metrics::{ExporterAttempt, ExporterMetrics};
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_telemetry::otel_warn;
use std::future::poll_fn;
use std::time::Instant;

const UNSUPPORTED_SIGNAL_MESSAGE: &str = "Geneva metrics exporter accepts metrics only";
const SHUTDOWN_AT_CAPACITY_MESSAGE: &str =
    "Geneva metrics exporter shutdown deadline exhausted before publication could start";

const GENEVA_METRICS_AUTH_EVENTS: HttpClientAuthProviderEvents = HttpClientAuthProviderEvents {
    validate_header_name: |_| Ok(()),
    on_invalid: |source, error| {
        otel_warn!("geneva_metrics_exporter.auth.invalid", source = %source, error = %error);
    },
    on_stream_closed: |source| {
        otel_warn!(
            "geneva_metrics_exporter.auth.stream_closed",
            source = %source,
            message = "auth provider closed its stream; no further auth refreshes will arrive"
        );
    },
};

struct InFlightPublication {
    data: OtapPdata,
    auth_generation: u64,
    request: LocalBoxFuture<'static, Result<(), PublishError>>,
    deadline: tokio::time::Instant,
    attempt: ExporterAttempt,
    remaining: std::vec::IntoIter<EncodedPacket>,
}

struct CompletedPublication {
    data: OtapPdata,
    auth_generation: u64,
    remaining: std::vec::IntoIter<EncodedPacket>,
    result: Result<(), PublishError>,
}

impl InFlightPublication {
    fn complete(
        self,
        result: Result<(), PublishError>,
        metrics: &mut ExporterMetrics,
    ) -> CompletedPublication {
        let result = match result {
            Ok(()) => Ok(()),
            Err(error) if error.is_retryable() => Err(self.attempt.failed(error)),
            Err(error) => Err(self.attempt.refused(error)),
        };
        CompletedPublication {
            data: self.data,
            auth_generation: self.auth_generation,
            remaining: self.remaining,
            result: metrics.record(self.attempt.complete(result)),
        }
    }
}

type CompletionFuture = LocalBoxFuture<'static, Result<(), EngineError>>;

enum PublicationStart {
    InFlight(Box<InFlightPublication>),
    Completion(CompletionFuture),
}

/// Pipeline exporter that converts OTLP or native OTAP metrics to Geneva packets.
pub struct GenevaMetricsExporter {
    mapping_config: Config,
    publisher: MetricsPublisher,
    auth: Box<dyn HttpClientAuthProvider>,
    metrics: ExporterMetrics,
}

impl GenevaMetricsExporter {
    pub(crate) fn new(
        mapping_config: Config,
        publisher: MetricsPublisher,
        token_provider: Box<dyn LocalBearerTokenProvider>,
        metrics: ExporterMetrics,
    ) -> Self {
        Self {
            mapping_config,
            publisher,
            metrics,
            auth: Box::new(new_http_client_auth_provider_from_bearer_token_provider(
                token_provider,
            )),
        }
    }

    fn ack_completion(
        effect_handler: &EffectHandler<OtapPdata>,
        data: OtapPdata,
    ) -> CompletionFuture {
        let effect_handler = effect_handler.clone();
        Box::pin(async move { effect_handler.notify_ack(AckMsg::new(data)).await })
    }

    fn nack_completion(
        effect_handler: &EffectHandler<OtapPdata>,
        nack: NackMsg<OtapPdata>,
    ) -> CompletionFuture {
        let effect_handler = effect_handler.clone();
        Box::pin(async move { effect_handler.notify_nack(nack).await })
    }

    fn record_refusal(&mut self, attempt: ExporterAttempt) {
        let result = Err::<(), _>(attempt.refused(()));
        _ = self.metrics.record(attempt.complete(result));
    }

    fn start_publication(
        &mut self,
        mut data: OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> PublicationStart {
        let signal_type = data.signal_type();
        let mut attempt = self.metrics.attempt(signal_type);
        attempt.set_item_count_with(|| data.num_items() as u64);
        if signal_type != SignalType::Metrics {
            self.record_refusal(attempt);
            let nack = NackMsg::new_permanent(UNSUPPORTED_SIGNAL_MESSAGE, data);
            return PublicationStart::Completion(Self::nack_completion(effect_handler, nack));
        }

        // ExporterInbox force-drains pdata during shutdown even when normal
        // admission is closed, so re-check auth before doing publication work.
        if !self.auth.is_ready() {
            self.record_refusal(attempt);
            let nack = NackMsg::new(self.auth.not_ready_reason(), data);
            return PublicationStart::Completion(Self::nack_completion(effect_handler, nack));
        }

        let prepared = match prepare_publication(&data, &self.mapping_config) {
            Ok(prepared) => prepared,
            Err(error) => {
                self.record_refusal(attempt);
                let nack = NackMsg::new_permanent(error.to_string(), data);
                return PublicationStart::Completion(Self::nack_completion(effect_handler, nack));
            }
        };
        if prepared.rejected_data_points > 0 || prepared.cardinality_overflows > 0 {
            otel_warn!(
                "geneva_metrics_exporter.mapping.partial",
                rejected_data_points = prepared.rejected_data_points,
                cardinality_overflows = prepared.cardinality_overflows,
            );
        }
        if prepared.publications.is_empty() {
            self.metrics
                .record(attempt.complete::<(), ()>(Ok(())))
                .expect("successful no-op has no error");
            return PublicationStart::Completion(Self::ack_completion(effect_handler, data));
        }

        // Mapping and packet splitting are shared preparation. Each HTTP
        // submission below owns a separate backend attempt.
        self.start_next_publication(data, prepared.publications.into_iter(), effect_handler)
    }

    fn start_next_publication(
        &mut self,
        data: OtapPdata,
        mut remaining: std::vec::IntoIter<EncodedPacket>,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> PublicationStart {
        let Some(packet) = remaining.next() else {
            return PublicationStart::Completion(Self::ack_completion(effect_handler, data));
        };
        let mut attempt = self.metrics.attempt(SignalType::Metrics);
        attempt.set_item_count_with(|| packet.item_count);
        attempt.set_payload_size_with(|| packet.bytes.len());
        if !self.auth.is_ready() {
            self.record_refusal(attempt);
            let nack = NackMsg::new(self.auth.not_ready_reason(), data);
            return PublicationStart::Completion(Self::nack_completion(effect_handler, nack));
        }
        let Some((auth_header_name, auth_header_value, auth_generation)) = self.auth.header()
        else {
            self.record_refusal(attempt);
            let nack = NackMsg::new(self.auth.not_ready_reason(), data);
            return PublicationStart::Completion(Self::nack_completion(effect_handler, nack));
        };

        let request = self.publisher.publish(
            &self.mapping_config.monitoring_account,
            packet.bytes,
            auth_header_name,
            auth_header_value,
        );
        PublicationStart::InFlight(Box::new(InFlightPublication {
            data,
            auth_generation,
            request,
            deadline: tokio::time::Instant::now() + self.publisher.timeout(),
            attempt,
            remaining,
        }))
    }

    fn publication_completion(
        &mut self,
        publication: CompletedPublication,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> PublicationStart {
        let CompletedPublication {
            data,
            auth_generation,
            remaining,
            result,
        } = publication;

        match result {
            Ok(()) => self.start_next_publication(data, remaining, effect_handler),
            Err(error) => {
                if error.is_unauthorized() {
                    self.auth.invalidate(auth_generation);
                }
                let monitoring_account = &self.mapping_config.monitoring_account;
                let reason = format!(
                    "failed to publish Geneva metrics for account {monitoring_account}: {error}"
                );
                let nack = if error.is_retryable() {
                    NackMsg::new(reason, data)
                } else {
                    NackMsg::new_permanent(reason, data)
                };
                PublicationStart::Completion(Self::nack_completion(effect_handler, nack))
            }
        }
    }

    async fn with_auth_refresh<T>(&mut self, work: impl Future<Output = T>) -> T {
        tokio::pin!(work);
        loop {
            tokio::select! {
                biased;
                () = poll_fn(|cx| self.auth.poll_refresh(cx, &GENEVA_METRICS_AUTH_EVENTS)),
                    if self.auth.is_active() => {}
                result = &mut work => return result,
            }
        }
    }

    async fn wait_for_shutdown_auth(&mut self, deadline: tokio::time::Instant) {
        let wait = async {
            while !self.auth.is_ready() && self.auth.is_active() {
                poll_fn(|cx| self.auth.poll_refresh(cx, &GENEVA_METRICS_AUTH_EVENTS)).await;
            }
        };
        if tokio::time::timeout_at(deadline, wait).await.is_err() {
            otel_warn!(
                "geneva_metrics_exporter.shutdown.deadline_exceeded",
                message = "authentication refresh unavailable before the shutdown deadline"
            );
        }
    }

    async fn finish_shutdown_work(
        &mut self,
        mut in_flight: Option<Box<InFlightPublication>>,
        mut pending_completion: Option<CompletionFuture>,
        deadline: Instant,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        let deadline = tokio::time::Instant::from_std(deadline);
        loop {
            if let Some(completion) = pending_completion.take() {
                match tokio::time::timeout_at(deadline, self.with_auth_refresh(completion)).await {
                    Ok(result) => result?,
                    Err(_) => {
                        otel_warn!(
                            "geneva_metrics_exporter.shutdown.deadline_exceeded",
                            message = "completion notification abandoned at the shutdown deadline"
                        );
                        return Ok(());
                    }
                }
            }
            let Some(mut publication) = in_flight.take() else {
                return Ok(());
            };
            let result = if tokio::time::Instant::now() < deadline {
                Some(
                    tokio::time::timeout_at(
                        deadline,
                        self.with_auth_refresh(publication.request.as_mut()),
                    )
                    .await,
                )
            } else {
                None
            };
            let result = match result {
                Some(Ok(result)) => result,
                None | Some(Err(_)) => {
                    let result = Err::<(), _>(publication.attempt.failed(()));
                    _ = self.metrics.record(publication.attempt.complete(result));
                    otel_warn!(
                        "geneva_metrics_exporter.shutdown.deadline_exceeded",
                        message = "publication work abandoned at the shutdown deadline"
                    );
                    return Ok(());
                }
            };
            let completed = publication.complete(result, &mut self.metrics);
            if completed.result.is_ok() && !completed.remaining.as_slice().is_empty() {
                self.wait_for_shutdown_auth(deadline).await;
            }
            match self.publication_completion(completed, effect_handler) {
                PublicationStart::InFlight(publication) => in_flight = Some(publication),
                PublicationStart::Completion(completion) => pending_completion = Some(completion),
            }
        }
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for GenevaMetricsExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        let margin_sleep = tokio::time::sleep_until(tokio::time::Instant::now());
        tokio::pin!(margin_sleep);
        let mut armed_margin_deadline: Option<Instant> = None;
        let mut in_flight: Option<Box<InFlightPublication>> = None;
        let mut pending_completion: Option<CompletionFuture> = None;
        let mut shutdown_drain_deadline: Option<Instant> = None;

        loop {
            let has_in_flight = in_flight.is_some();
            let has_pending_completion = pending_completion.is_some();
            let accepting_pdata = self.auth.is_ready()
                && !has_in_flight
                && !has_pending_completion
                && shutdown_drain_deadline.is_none();
            let auth_margin_deadline = self.auth.refresh_deadline();
            if auth_margin_deadline != armed_margin_deadline {
                if let Some(deadline) = auth_margin_deadline {
                    margin_sleep
                        .as_mut()
                        .reset(tokio::time::Instant::from_std(deadline));
                }
                armed_margin_deadline = auth_margin_deadline;
            }

            let msg = tokio::select! {
                biased;

                () = &mut margin_sleep, if auth_margin_deadline.is_some() => {
                    continue;
                }

                () = poll_fn(|cx| self.auth.poll_refresh(cx, &GENEVA_METRICS_AUTH_EVENTS)), if self.auth.is_active() => {
                    continue;
                }

                result = async {
                    pending_completion
                        .as_mut()
                        .expect("pending completion branch must be guarded")
                        .as_mut()
                        .await
                }, if has_pending_completion => {
                    drop(pending_completion
                        .take()
                        .expect("completed notification must be present"));
                    result?;
                    continue;
                }

                result = async {
                    in_flight
                        .as_mut()
                        .expect("in-flight publication branch must be guarded")
                        .request
                        .as_mut()
                        .await
                }, if has_in_flight => {
                    let publication = in_flight
                        .take()
                        .expect("completed publication must be present");
                    let completed = publication.complete(result, &mut self.metrics);
                    match self.publication_completion(completed, &effect_handler) {
                        PublicationStart::InFlight(publication) => in_flight = Some(publication),
                        PublicationStart::Completion(completion) => pending_completion = Some(completion),
                    }
                    continue;
                }

                msg = msg_chan.recv_when(accepting_pdata || shutdown_drain_deadline.is_some()) => msg?,
            };

            match msg {
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    let work_deadline =
                        shutdown_drain_deadline.map_or(deadline, |local| local.min(deadline));
                    self.finish_shutdown_work(
                        in_flight.take(),
                        pending_completion.take(),
                        work_deadline,
                        &effect_handler,
                    )
                    .await?;
                    return Ok(TerminalState::new(
                        deadline,
                        self.metrics.terminal_snapshots(),
                    ));
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    if let Err(error) = self.metrics.report(&mut metrics_reporter) {
                        otel_warn!("geneva_metrics_exporter.telemetry.failed", error = %error);
                    }
                }
                Message::PData(mut data) => {
                    if !accepting_pdata {
                        // Closed admission only delivers pdata during shutdown.
                        // The inbox hides the pipeline deadline until draining
                        // completes, so share one local timeout across the drain.
                        let deadline = *shutdown_drain_deadline.get_or_insert_with(|| {
                            in_flight.as_ref().map_or_else(
                                || {
                                    (tokio::time::Instant::now() + self.publisher.timeout())
                                        .into_std()
                                },
                                |publication| publication.deadline.into_std(),
                            )
                        });
                        let at_capacity = has_in_flight || has_pending_completion;
                        if at_capacity {
                            self.finish_shutdown_work(
                                in_flight.take(),
                                pending_completion.take(),
                                deadline,
                                &effect_handler,
                            )
                            .await?;
                        }
                        if data.signal_type() == SignalType::Metrics {
                            self.wait_for_shutdown_auth(tokio::time::Instant::from_std(deadline))
                                .await;
                        }
                        if tokio::time::Instant::now() >= tokio::time::Instant::from_std(deadline) {
                            let mut attempt = self.metrics.attempt(data.signal_type());
                            attempt.set_item_count_with(|| data.num_items() as u64);
                            self.record_refusal(attempt);
                            let nack = NackMsg::new(SHUTDOWN_AT_CAPACITY_MESSAGE, data);
                            pending_completion = Some(Self::nack_completion(&effect_handler, nack));
                            continue;
                        }
                    }
                    match self.start_publication(data, &effect_handler) {
                        PublicationStart::InFlight(publication) => {
                            in_flight = Some(publication);
                        }
                        PublicationStart::Completion(completion) => {
                            pending_completion = Some(completion);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
    use otel_arrow_dfe_channel::mpsc;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::capability::CapabilityError;
    use otel_arrow_dfe_engine::capability::auth::BearerToken;
    use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::TokenStream;
    use otel_arrow_dfe_engine::control::{
        PipelineCompletionMsg, PipelineCompletionMsgReceiver, pipeline_completion_msg_channel,
    };
    use otel_arrow_dfe_engine::local::message::LocalReceiver;
    use otel_arrow_dfe_engine::message::Receiver;
    use otel_arrow_dfe_engine::node::NodeId;
    use otel_arrow_dfe_otap::testing::TestCallData;
    use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric, number_data_point,
    };
    use otel_arrow_dfe_pdata_codec::OtapPayload;
    use otel_arrow_dfe_telemetry::metrics::{MetricSetSnapshot, MetricValue};
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
    use std::cell::RefCell;
    use std::time::{Duration, Instant};
    use wiremock::matchers::{header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    struct TestTokenProvider {
        updates: RefCell<Option<UnboundedReceiver<BearerToken>>>,
        observations: UnboundedSender<()>,
    }

    struct TokenController {
        updates: UnboundedSender<BearerToken>,
        observations: UnboundedReceiver<()>,
    }

    impl TokenController {
        fn publish(&self, value: &str) {
            let token = BearerToken::without_expiry(value.to_string());
            self.updates
                .unbounded_send(token)
                .expect("test token stream should remain open");
        }

        async fn wait_until_observed(&mut self) {
            self.observations
                .next()
                .await
                .expect("exporter should observe the test token update");
        }
    }

    impl TestTokenProvider {
        fn new(initial_token: Option<&str>) -> (Self, TokenController) {
            let (updates_tx, updates_rx) = unbounded();
            let (observations_tx, observations_rx) = unbounded();
            let controller = TokenController {
                updates: updates_tx,
                observations: observations_rx,
            };
            if let Some(token) = initial_token {
                controller.publish(token);
            }
            (
                Self {
                    updates: RefCell::new(Some(updates_rx)),
                    observations: observations_tx,
                },
                controller,
            )
        }
    }

    #[async_trait(?Send)]
    impl LocalBearerTokenProvider for TestTokenProvider {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            panic!("the shared HTTP auth adapter should consume the token stream")
        }

        fn token_stream(&self) -> TokenStream {
            let observations = self.observations.clone();
            let updates = self
                .updates
                .borrow_mut()
                .take()
                .expect("test token stream should be subscribed once")
                .inspect(move |_| {
                    observations
                        .unbounded_send(())
                        .expect("test token observer should remain open");
                });
            Box::pin(updates)
        }
    }

    fn mapping_config() -> Config {
        Config {
            monitoring_account: "default-account".to_string(),
            metric_namespace: "example-namespace".to_string(),
            resource_attributes: Vec::new(),
            honor_resource_attributes: false,
            scope_attributes: Vec::new(),
            honor_scope_attributes: false,
            disable_exemplars: false,
        }
    }

    fn metrics_pdata(call_id: usize) -> OtapPdata {
        metrics_pdata_with_format(call_id, false)
    }

    fn metrics_pdata_with_format(call_id: usize, native: bool) -> OtapPdata {
        let point = NumberDataPoint {
            attributes: Vec::new(),
            start_time_unix_nano: 0,
            time_unix_nano: 1_700_000_000_000_000_000,
            exemplars: Vec::new(),
            flags: 0,
            value: Some(number_data_point::Value::AsDouble(12.5)),
        };
        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: None,
                scope_metrics: vec![ScopeMetrics {
                    scope: None,
                    metrics: vec![Metric {
                        name: "temperature".to_string(),
                        description: String::new(),
                        unit: String::new(),
                        metadata: Vec::new(),
                        data: Some(metric::Data::Gauge(Gauge {
                            data_points: vec![point],
                        })),
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let message = OtlpProtoMessage::Metrics(request.into());
        let payload = if native {
            OtapPayload::from_otap(otel_arrow_dfe_pdata::testing::round_trip::otlp_to_otap(
                &message,
            ))
        } else {
            OtapPayload::try_from(message).expect("request should serialize")
        };
        OtapPdata::new_default(payload).test_subscribe_to(
            Interests::ACKS | Interests::NACKS,
            TestCallData::default().into(),
            call_id,
        )
    }

    fn completion_harness() -> (
        EffectHandler<OtapPdata>,
        PipelineCompletionMsgReceiver<OtapPdata>,
    ) {
        completion_harness_with_capacity(4)
    }

    fn completion_harness_with_capacity(
        capacity: usize,
    ) -> (
        EffectHandler<OtapPdata>,
        PipelineCompletionMsgReceiver<OtapPdata>,
    ) {
        let (_, reporter) = MetricsReporter::create_new_and_receiver(10);
        let mut effect_handler = EffectHandler::new(
            NodeId {
                index: 0,
                name: "geneva-metrics-test".to_string().into(),
            },
            reporter,
            otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
        );
        let (completion_tx, completion_rx) = pipeline_completion_msg_channel(capacity);
        effect_handler.set_pipeline_completion_msg_sender(completion_tx);
        (effect_handler, completion_rx)
    }

    async fn occupy_completion_channel(effect_handler: &EffectHandler<OtapPdata>) {
        effect_handler
            .notify_nack(NackMsg::new(
                "occupy the completion channel",
                metrics_pdata(0),
            ))
            .await
            .expect("completion channel should accept its first message");
    }

    fn message_channel(
        capacity: usize,
    ) -> (
        mpsc::Sender<NodeControlMsg<OtapPdata>>,
        mpsc::Sender<OtapPdata>,
        ExporterInbox<OtapPdata>,
    ) {
        let (control_tx, control_rx) = mpsc::Channel::new(capacity);
        let (pdata_tx, pdata_rx) = mpsc::Channel::new(capacity);
        (
            control_tx,
            pdata_tx,
            ExporterInbox::new(
                Receiver::Local(LocalReceiver::mpsc(control_rx)),
                Receiver::Local(LocalReceiver::mpsc(pdata_rx)),
                0,
                Interests::empty(),
            ),
        )
    }

    fn exporter_with_timeout(
        endpoint: &str,
        timeout: Duration,
        token_provider: TestTokenProvider,
    ) -> GenevaMetricsExporter {
        GenevaMetricsExporter::new(
            mapping_config(),
            MetricsPublisher::new(endpoint, timeout).expect("publisher should be created"),
            Box::new(token_provider),
            ExporterMetrics::register(
                &otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests(
                    Interests::NODE_INPUT_METRICS
                        | Interests::NODE_LOCAL_DURATION
                        | Interests::NODE_ITEM_COUNTS
                        | Interests::NODE_SIZE,
                )
                .0,
            ),
        )
    }

    fn exporter(endpoint: &str, token_provider: TestTokenProvider) -> GenevaMetricsExporter {
        exporter_with_timeout(endpoint, Duration::from_secs(1), token_provider)
    }

    async fn wait_until_request_received(server: &MockServer) {
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                let requests = server
                    .received_requests()
                    .await
                    .expect("test server should record requests");
                if !requests.is_empty() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("exporter should start the HTTP publication");
    }

    fn metric_value<'a>(
        snapshots: &'a [MetricSetSnapshot],
        outcome: &str,
        name: &str,
    ) -> &'a MetricValue {
        snapshots
            .iter()
            .find_map(|snapshot| {
                if snapshot.descriptor().name != "exporter.attempted"
                    || snapshot.measurement_attribute_value("signal") != Some("metrics")
                    || snapshot.measurement_attribute_value("outcome") != Some(outcome)
                {
                    return None;
                }
                snapshot
                    .descriptor()
                    .metrics
                    .iter()
                    .position(|field| field.name == name)
                    .map(|index| &snapshot.get_metrics()[index])
            })
            .expect("expected shared exporter metric")
    }

    /// Scenario: Native OTAP metrics are queued before the bearer provider publishes its initial token.
    /// Guarantees: The start loop publishes and ACKs native metrics after auth arrives, then reports the standard exporter measurements.
    #[tokio::test]
    async fn start_loop_waits_for_initial_auth() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        let (provider, mut controller) = TestTokenProvider::new(None);
        let exporter = exporter(&server.uri(), provider);
        let (effect_handler, mut completions) = completion_harness();
        let (control_tx, pdata_tx, msg_chan) = message_channel(4);
        let control_guard = control_tx.clone();

        let driver = async move {
            pdata_tx
                .send_async(metrics_pdata_with_format(1, true))
                .await
                .expect("pdata should be queued before auth is ready");
            controller.publish("fresh-token");
            controller.wait_until_observed().await;

            match completions.recv().await.expect("completion should arrive") {
                PipelineCompletionMsg::DeliverAck { .. } => {}
                PipelineCompletionMsg::DeliverNack { .. } => {
                    panic!("pdata should wait for auth rather than be NACKed")
                }
            }

            let (snapshots, reporter) = MetricsReporter::create_new_and_receiver(8);
            control_tx
                .send_async(NodeControlMsg::CollectTelemetry {
                    metrics_reporter: reporter,
                })
                .await
                .expect("telemetry collection should be queued");
            let mut collected = Vec::new();
            for _ in 0..4 {
                collected.push(
                    snapshots
                        .recv_async()
                        .await
                        .expect("shared snapshot should arrive"),
                );
            }
            assert_eq!(
                metric_value(&collected, "success", "messages"),
                &MetricValue::U64(1)
            );
            assert_eq!(
                metric_value(&collected, "success", "items"),
                &MetricValue::U64(1)
            );
            assert!(!metric_value(&collected, "success", "payload.size").is_zero());
            assert!(!metric_value(&collected, "success", "duration").is_zero());

            control_tx
                .send_async(NodeControlMsg::Shutdown {
                    deadline: Instant::now(),
                    reason: "test complete".to_owned(),
                })
                .await
                .expect("shutdown should be queued");
            drop(pdata_tx);
        };

        let (start_result, ()) = tokio::time::timeout(Duration::from_secs(2), async {
            tokio::join!(Box::new(exporter).start(msg_chan, effect_handler), driver)
        })
        .await
        .expect("exporter should finish after initial auth arrives");
        drop(control_guard);
        let terminal_state = start_result.expect("exporter should shut down cleanly");
        assert!(
            terminal_state.is_empty(),
            "periodic reporting must clear deltas"
        );
    }

    /// Scenario: Shutdown is already queued with two healthy publications buffered.
    /// Guarantees: Occupied capacity waits within the grace period, both inputs are ACKed, and both attempts reach terminal telemetry.
    #[tokio::test]
    async fn shutdown_publishes_healthy_buffered_metrics() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(2)
            .mount(&server)
            .await;
        let (provider, _controller) = TestTokenProvider::new(Some("ready-token"));
        let exporter = exporter(&server.uri(), provider);
        let (effect_handler, mut completions) = completion_harness();
        let (control_tx, pdata_tx, msg_chan) = message_channel(4);
        for call_id in 1..=2 {
            pdata_tx
                .send_async(metrics_pdata(call_id))
                .await
                .expect("buffer metrics");
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        control_tx
            .send_async(NodeControlMsg::Shutdown {
                deadline,
                reason: "drain healthy backlog".to_owned(),
            })
            .await
            .expect("queue shutdown");
        drop(pdata_tx);
        let driver = async {
            for _ in 0..2 {
                assert!(matches!(
                    completions
                        .recv()
                        .await
                        .expect("drained input should complete"),
                    PipelineCompletionMsg::DeliverAck { .. },
                ));
            }
        };
        let (result, ()) = tokio::time::timeout(Duration::from_secs(10), async {
            tokio::join!(Box::new(exporter).start(msg_chan, effect_handler), driver)
        })
        .await
        .expect("healthy backlog should drain");
        drop(control_tx);
        let terminal = result.expect("shutdown should succeed");
        assert_eq!(terminal.deadline(), deadline);
        assert_eq!(
            metric_value(terminal.metrics(), "success", "messages"),
            &MetricValue::U64(2)
        );
        assert_eq!(
            metric_value(terminal.metrics(), "success", "items"),
            &MetricValue::U64(2)
        );
    }

    /// Scenario: Cached authentication becomes unusable and a replacement arrives during or between shutdown publications.
    /// Guarantees: Shutdown consumes refreshes while awaiting HTTP, waits for credentials between packets, and ACKs the entire input after both packets succeed.
    #[tokio::test]
    async fn shutdown_refreshes_auth_while_draining_split_publications() {
        for refresh_during_request in [true, false] {
            let server = MockServer::start().await;
            for token in ["initial-token", "fresh-token"] {
                Mock::given(method("POST"))
                    .and(header("authorization", format!("Bearer {token}")))
                    .respond_with(ResponseTemplate::new(200))
                    .expect(1)
                    .mount(&server)
                    .await;
            }
            let (provider, mut controller) = TestTokenProvider::new(Some("initial-token"));
            let mut exporter =
                exporter_with_timeout(&server.uri(), Duration::from_secs(10), provider);
            poll_fn(|cx| exporter.auth.poll_refresh(cx, &GENEVA_METRICS_AUTH_EVENTS)).await;
            controller.wait_until_observed().await;
            let (effect_handler, mut completions) = completion_harness();
            let data = metrics_pdata(1);
            let packets = (0..2)
                .flat_map(|_| {
                    prepare_publication(&data, &mapping_config())
                        .expect("prepare packet")
                        .publications
                })
                .collect::<Vec<_>>();
            let PublicationStart::InFlight(mut publication) =
                exporter.start_next_publication(data, packets.into_iter(), &effect_handler)
            else {
                panic!("first publication should start with usable authentication");
            };
            exporter.auth.invalidate(publication.auth_generation);
            assert!(!exporter.auth.is_ready());
            let (started_tx, started_rx) = tokio::sync::oneshot::channel();
            let (release_tx, release_rx) = tokio::sync::oneshot::channel();
            let (finished_tx, finished_rx) = tokio::sync::oneshot::channel();
            let original_request = publication.request;
            publication.request = Box::pin(async move {
                started_tx.send(()).expect("observe publication start");
                release_rx.await.expect("release first publication");
                let result = original_request.await;
                finished_tx
                    .send(())
                    .expect("observe publication completion");
                result
            });
            let driver = async {
                started_rx
                    .await
                    .expect("shutdown should poll the first publication");
                if refresh_during_request {
                    controller.publish("fresh-token");
                    controller.wait_until_observed().await;
                    release_tx
                        .send(())
                        .expect("release publication after refresh");
                    finished_rx.await.expect("first publication should finish");
                } else {
                    release_tx
                        .send(())
                        .expect("complete publication before refresh");
                    finished_rx.await.expect("first publication should finish");
                    controller.publish("fresh-token");
                    controller.wait_until_observed().await;
                }
            };
            let deadline = Instant::now() + Duration::from_secs(30);
            let (result, ()) = tokio::time::timeout(Duration::from_secs(10), async {
                tokio::join!(
                    exporter.finish_shutdown_work(
                        Some(publication),
                        None,
                        deadline,
                        &effect_handler
                    ),
                    driver,
                )
            })
            .await
            .expect("shutdown should consume refreshed authentication and drain both packets");
            result.expect("shutdown publication work should succeed");
            assert!(matches!(
                completions.recv().await.expect("input should complete"),
                PipelineCompletionMsg::DeliverAck { .. },
            ));
            let snapshots = exporter.metrics.terminal_snapshots();
            assert_eq!(
                metric_value(&snapshots, "success", "messages"),
                &MetricValue::U64(2)
            );
        }
    }

    /// Scenario: A split publication needs refreshed authentication, but no replacement arrives before shutdown expires.
    /// Guarantees: Waiting respects the original virtual deadline, never submits the next packet, and retryably NACKs the input.
    #[tokio::test(start_paused = true)]
    async fn shutdown_bounds_wait_for_refreshed_auth() {
        let (provider, _controller) = TestTokenProvider::new(Some("initial-token"));
        let mut exporter = exporter("http://127.0.0.1:1", provider);
        poll_fn(|cx| exporter.auth.poll_refresh(cx, &GENEVA_METRICS_AUTH_EVENTS)).await;
        let (effect_handler, mut completions) = completion_harness();
        let data = metrics_pdata(1);
        let packets = (0..2)
            .flat_map(|_| {
                prepare_publication(&data, &mapping_config())
                    .expect("prepare packet")
                    .publications
            })
            .collect::<Vec<_>>();
        let PublicationStart::InFlight(mut publication) =
            exporter.start_next_publication(data, packets.into_iter(), &effect_handler)
        else {
            panic!("first publication should start");
        };
        exporter.auth.invalidate(publication.auth_generation);
        publication.request = Box::pin(async { Ok(()) });
        let deadline = Instant::now() + Duration::from_secs(1);
        tokio::time::timeout(
            Duration::from_secs(10),
            exporter.finish_shutdown_work(Some(publication), None, deadline, &effect_handler),
        )
        .await
        .expect("authentication wait must be bounded")
        .expect("shutdown should finish");
        assert!(tokio::time::Instant::now() >= tokio::time::Instant::from_std(deadline));
        match completions.recv().await.expect("input should complete") {
            PipelineCompletionMsg::DeliverNack { nack } => assert!(!nack.permanent),
            PipelineCompletionMsg::DeliverAck { .. } => {
                panic!("unsubmitted packets cannot be ACKed")
            }
        }
        let snapshots = exporter.metrics.terminal_snapshots();
        assert_eq!(
            metric_value(&snapshots, "success", "messages"),
            &MetricValue::U64(1)
        );
        assert_eq!(
            metric_value(&snapshots, "refused", "messages"),
            &MetricValue::U64(1)
        );
    }

    /// Scenario: One input is split into two publications and the second succeeds or fails.
    /// Guarantees: No early ACK is sent, any packet failure NACKs the input, and every HTTP submission records its own telemetry.
    #[tokio::test]
    async fn split_publications_complete_once_and_record_each_attempt() {
        for final_status in [200, 503, 400] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(200))
                .with_priority(1)
                .up_to_n_times(1)
                .expect(1)
                .mount(&server)
                .await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(final_status))
                .with_priority(2)
                .expect(1)
                .mount(&server)
                .await;
            let (provider, _controller) = TestTokenProvider::new(Some("ready-token"));
            let mut exporter = exporter(&server.uri(), provider);
            poll_fn(|cx| exporter.auth.poll_refresh(cx, &GENEVA_METRICS_AUTH_EVENTS)).await;
            assert!(exporter.auth.is_ready());
            let (effect_handler, mut completions) = completion_harness();
            let data = metrics_pdata(1);
            let packets = (0..2)
                .flat_map(|_| {
                    prepare_publication(&data, &mapping_config())
                        .expect("prepare packet")
                        .publications
                })
                .collect::<Vec<_>>();
            let mut work =
                exporter.start_next_publication(data, packets.into_iter(), &effect_handler);
            for _ in 0..2 {
                let PublicationStart::InFlight(mut publication) = work else {
                    panic!("each packet must be published before input completion");
                };
                assert!(matches!(
                    completions.try_recv(),
                    Err(otel_arrow_dfe_channel::error::RecvError::Empty),
                ));
                let result =
                    tokio::time::timeout(Duration::from_secs(10), publication.request.as_mut())
                        .await
                        .expect("HTTP publication should finish");
                let completed = publication.complete(result, &mut exporter.metrics);
                work = exporter.publication_completion(completed, &effect_handler);
            }
            let PublicationStart::Completion(completion) = work else {
                panic!("all packets must resolve to one input completion");
            };
            completion
                .await
                .expect("input completion should be delivered");
            match completions
                .recv()
                .await
                .expect("input should complete once")
            {
                PipelineCompletionMsg::DeliverAck { .. } => assert_eq!(final_status, 200),
                PipelineCompletionMsg::DeliverNack { nack } => {
                    assert_ne!(final_status, 200);
                    assert_eq!(nack.permanent, final_status == 400);
                }
            }
            let snapshots = exporter.metrics.terminal_snapshots();
            let successes = if final_status == 200 { 2 } else { 1 };
            assert_eq!(
                metric_value(&snapshots, "success", "messages"),
                &MetricValue::U64(successes)
            );
            assert_eq!(
                metric_value(&snapshots, "success", "items"),
                &MetricValue::U64(successes)
            );
            if final_status != 200 {
                let outcome = if final_status == 400 {
                    "refused"
                } else {
                    "failure"
                };
                assert_eq!(
                    metric_value(&snapshots, outcome, "messages"),
                    &MetricValue::U64(1)
                );
            }
            assert!(matches!(
                completions.try_recv(),
                Err(otel_arrow_dfe_channel::error::RecvError::Empty),
            ));
        }
    }

    /// Scenario: The exporter receives shutdown when no HTTP publication is active.
    /// Guarantees: The terminal state preserves the exact pipeline shutdown deadline.
    #[tokio::test]
    async fn start_loop_preserves_shutdown_deadline() {
        let server = MockServer::start().await;
        let (provider, _controller) = TestTokenProvider::new(None);
        let exporter = exporter(&server.uri(), provider);
        let (effect_handler, _completions) = completion_harness();
        let (control_tx, pdata_tx, msg_chan) = message_channel(1);
        let control_guard = control_tx.clone();
        let pdata_guard = pdata_tx.clone();
        let deadline = Instant::now() + Duration::from_millis(100);

        let driver = async move {
            control_tx
                .send_async(NodeControlMsg::Shutdown {
                    deadline,
                    reason: "test complete".to_owned(),
                })
                .await
                .expect("shutdown should be queued");
            drop(pdata_tx);
        };

        let (start_result, ()) = tokio::time::timeout(Duration::from_secs(2), async {
            tokio::join!(Box::new(exporter).start(msg_chan, effect_handler), driver)
        })
        .await
        .expect("exporter should preserve shutdown responsiveness");
        drop(control_guard);
        drop(pdata_guard);
        let terminal_state = start_result.expect("exporter should shut down cleanly");
        assert_eq!(terminal_state.deadline(), deadline);
    }

    /// Scenario: Shutdown force-drains pdata while one HTTP publication is stalled beyond the pipeline deadline.
    /// Guarantees: Only one request is in flight, force-drained pdata is retryably NACKed,
    /// local timeout bounds publication work, and the terminal state preserves the pipeline deadline.
    #[tokio::test]
    async fn shutdown_bounds_stalled_publication_and_force_drained_pdata() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(5)))
            .expect(1)
            .mount(&server)
            .await;
        let (provider, mut controller) = TestTokenProvider::new(Some("ready-token"));
        let exporter = exporter_with_timeout(&server.uri(), Duration::from_millis(200), provider);
        let (effect_handler, mut completions) = completion_harness();
        let (control_tx, pdata_tx, msg_chan) = message_channel(4);
        let control_guard = control_tx.clone();
        let pdata_guard = pdata_tx.clone();

        let driver = async move {
            controller.wait_until_observed().await;
            pdata_tx
                .send_async(metrics_pdata(1))
                .await
                .expect("first pdata should be queued");
            wait_until_request_received(&server).await;
            pdata_tx
                .send_async(metrics_pdata(2))
                .await
                .expect("second pdata should be buffered behind the active publication");

            let deadline = Instant::now() + Duration::from_millis(100);
            control_tx
                .send_async(NodeControlMsg::Shutdown {
                    deadline,
                    reason: "test shutdown".to_owned(),
                })
                .await
                .expect("shutdown should be queued");

            loop {
                match completions
                    .recv()
                    .await
                    .expect("force-drained pdata completion should arrive")
                {
                    PipelineCompletionMsg::DeliverNack { nack } => {
                        assert!(!nack.permanent);
                        if nack.reason.contains(SHUTDOWN_AT_CAPACITY_MESSAGE) {
                            break;
                        }
                    }
                    PipelineCompletionMsg::DeliverAck { .. } => {
                        panic!("stalled or force-drained pdata should not be ACKed")
                    }
                }
            }

            drop(pdata_tx);
            deadline
        };

        let (start_result, deadline) = tokio::time::timeout(Duration::from_secs(2), async {
            tokio::join!(Box::new(exporter).start(msg_chan, effect_handler), driver)
        })
        .await
        .expect("exporter should stop within its local drain timeout");
        drop(control_guard);
        drop(pdata_guard);
        let terminal_state = start_result.expect("exporter should shut down cleanly");
        assert_eq!(terminal_state.deadline(), deadline);
    }

    /// Scenario: Shutdown begins with a stalled HTTP request and a full, undrained completion channel.
    /// Guarantees: The local drain timeout bounds completion backpressure even before the pipeline deadline is visible.
    #[tokio::test]
    async fn shutdown_bounds_undrained_completion_channel() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(5)))
            .expect(1)
            .mount(&server)
            .await;
        let (provider, mut controller) = TestTokenProvider::new(Some("ready-token"));
        let exporter = exporter_with_timeout(&server.uri(), Duration::from_millis(200), provider);
        let (effect_handler, _completions) = completion_harness_with_capacity(1);
        occupy_completion_channel(&effect_handler).await;
        let (control_tx, pdata_tx, msg_chan) = message_channel(4);
        let control_guard = control_tx.clone();
        let pdata_guard = pdata_tx.clone();

        let driver = async move {
            controller.wait_until_observed().await;
            pdata_tx
                .send_async(metrics_pdata(1))
                .await
                .expect("first pdata should be queued");
            wait_until_request_received(&server).await;
            pdata_tx
                .send_async(metrics_pdata(2))
                .await
                .expect("second pdata should be buffered behind the active publication");

            let shutdown_started_at = Instant::now();
            let deadline = shutdown_started_at + Duration::from_millis(100);
            control_tx
                .send_async(NodeControlMsg::Shutdown {
                    deadline,
                    reason: "test shutdown".to_owned(),
                })
                .await
                .expect("shutdown should be queued");
            drop(pdata_tx);
            (deadline, shutdown_started_at)
        };

        let (start_result, (deadline, shutdown_started_at)) =
            tokio::time::timeout(Duration::from_secs(2), async {
                tokio::join!(Box::new(exporter).start(msg_chan, effect_handler), driver)
            })
            .await
            .expect("completion backpressure must not block shutdown");
        drop(control_guard);
        drop(pdata_guard);
        let terminal_state = start_result.expect("exporter should shut down cleanly");
        assert_eq!(terminal_state.deadline(), deadline);
        assert!(
            shutdown_started_at.elapsed() < Duration::from_secs(1),
            "shutdown should remain bounded by the local drain timeout"
        );
    }

    /// Scenario: Shutdown force-drains pdata while authentication is unavailable and the completion channel is full.
    /// Guarantees: The shared local drain timeout bounds authentication and retryable NACK backpressure.
    #[tokio::test]
    async fn shutdown_bounds_auth_unready_completion_backpressure() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&server)
            .await;
        let (provider, _controller) = TestTokenProvider::new(None);
        let exporter = exporter_with_timeout(&server.uri(), Duration::from_millis(200), provider);
        let (effect_handler, _completions) = completion_harness_with_capacity(1);
        occupy_completion_channel(&effect_handler).await;
        let (control_tx, pdata_tx, msg_chan) = message_channel(4);
        let control_guard = control_tx.clone();
        let pdata_guard = pdata_tx.clone();

        let driver = async move {
            pdata_tx
                .send_async(metrics_pdata(1))
                .await
                .expect("pdata should be buffered while authentication is unavailable");

            let shutdown_started_at = Instant::now();
            let deadline = shutdown_started_at + Duration::from_millis(100);
            control_tx
                .send_async(NodeControlMsg::Shutdown {
                    deadline,
                    reason: "test shutdown".to_owned(),
                })
                .await
                .expect("shutdown should be queued");
            drop(pdata_tx);
            (deadline, shutdown_started_at)
        };

        let (start_result, (deadline, shutdown_started_at)) =
            tokio::time::timeout(Duration::from_secs(2), async {
                tokio::join!(Box::new(exporter).start(msg_chan, effect_handler), driver)
            })
            .await
            .expect("authentication NACK backpressure must not block shutdown");
        drop(control_guard);
        drop(pdata_guard);
        let terminal_state = start_result.expect("exporter should shut down cleanly");
        assert_eq!(terminal_state.deadline(), deadline);
        assert!(
            shutdown_started_at.elapsed() < Duration::from_secs(1),
            "shutdown should remain bounded by the local drain timeout"
        );
    }

    /// Scenario: Shutdown drains two inputs without authentication while completion delivery is blocked.
    /// Guarantees: One local timeout bounds the entire drain and final completion wait without changing the terminal pipeline deadline.
    #[tokio::test(start_paused = true)]
    async fn shutdown_draining_shares_local_timeout_across_buffered_input() {
        let (provider, _controller) = TestTokenProvider::new(None);
        let drain_timeout = Duration::from_secs(3);
        let exporter = exporter_with_timeout("http://127.0.0.1:1", drain_timeout, provider);
        let (effect_handler, _completions) = completion_harness_with_capacity(1);
        occupy_completion_channel(&effect_handler).await;
        let (control_tx, pdata_tx, msg_chan) = message_channel(4);
        for call_id in [1, 2] {
            pdata_tx
                .send_async(metrics_pdata(call_id))
                .await
                .expect("input should be buffered before shutdown");
        }
        let deadline = Instant::now() + Duration::from_secs(60);
        control_tx
            .send_async(NodeControlMsg::Shutdown {
                deadline,
                reason: "test shutdown".to_owned(),
            })
            .await
            .expect("shutdown should be queued");
        drop(pdata_tx);

        let started_at = tokio::time::Instant::now();
        let terminal_state = tokio::time::timeout(
            Duration::from_secs(10),
            Box::new(exporter).start(msg_chan, effect_handler),
        )
        .await
        .expect("one local timeout should bound the entire shutdown drain")
        .expect("exporter should shut down cleanly");

        assert_eq!(started_at.elapsed(), drain_timeout);
        assert_eq!(terminal_state.deadline(), deadline);
    }

    /// Scenario: An HTTP request completes before shutdown while its ACK is blocked by a full completion channel.
    /// Guarantees: The blocked ACK remains pending work and cannot prevent the exporter from observing the deadline.
    #[tokio::test]
    async fn shutdown_bounds_completed_request_completion_backpressure() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        let (provider, mut controller) = TestTokenProvider::new(Some("ready-token"));
        let exporter = exporter_with_timeout(&server.uri(), Duration::from_secs(5), provider);
        let (effect_handler, _completions) = completion_harness_with_capacity(1);
        occupy_completion_channel(&effect_handler).await;
        let (control_tx, pdata_tx, msg_chan) = message_channel(4);
        let control_guard = control_tx.clone();
        let pdata_guard = pdata_tx.clone();

        let driver = async move {
            controller.wait_until_observed().await;
            pdata_tx
                .send_async(metrics_pdata(1))
                .await
                .expect("pdata should be queued");
            wait_until_request_received(&server).await;
            tokio::time::sleep(Duration::from_millis(100)).await;

            let shutdown_started_at = Instant::now();
            let deadline = shutdown_started_at + Duration::from_millis(100);
            control_tx
                .send_async(NodeControlMsg::Shutdown {
                    deadline,
                    reason: "test shutdown".to_owned(),
                })
                .await
                .expect("shutdown should be queued");
            drop(pdata_tx);
            (deadline, shutdown_started_at)
        };

        let (start_result, (deadline, shutdown_started_at)) =
            tokio::time::timeout(Duration::from_secs(2), async {
                tokio::join!(Box::new(exporter).start(msg_chan, effect_handler), driver)
            })
            .await
            .expect("completed-request ACK backpressure must not block shutdown");
        drop(control_guard);
        drop(pdata_guard);
        let terminal_state = start_result.expect("exporter should shut down cleanly");
        assert_eq!(terminal_state.deadline(), deadline);
        assert!(
            shutdown_started_at.elapsed() < Duration::from_secs(1),
            "shutdown should remain bounded by the supplied deadline"
        );
    }

    /// Scenario: The token stream closes after HTTP 401 while refused pdata is buffered and shutdown begins.
    /// Guarantees: The exporter avoids a closed-stream busy loop, keeps shutdown responsive, and never reacquires or republishes the rejected token.
    #[tokio::test]
    async fn closed_token_stream_keeps_shutdown_responsive() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&server)
            .await;
        let (provider, mut controller) = TestTokenProvider::new(Some("stale-token"));
        let exporter = exporter(&server.uri(), provider);
        let (effect_handler, mut completions) = completion_harness();
        let (control_tx, pdata_tx, msg_chan) = message_channel(4);
        // Keep the control channel alive while the pending shutdown drains pdata.
        let control_guard = control_tx.clone();

        let driver = async move {
            controller.wait_until_observed().await;
            pdata_tx
                .send_async(metrics_pdata(1))
                .await
                .expect("initial pdata should be queued");
            let replay = match completions
                .recv()
                .await
                .expect("first completion should arrive")
            {
                PipelineCompletionMsg::DeliverNack { nack } => {
                    assert!(!nack.permanent);
                    *nack.refused
                }
                PipelineCompletionMsg::DeliverAck { .. } => panic!("expected transient NACK"),
            };

            pdata_tx
                .send_async(replay)
                .await
                .expect("replay should be queued");
            drop(controller);
            control_tx
                .send_async(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_secs(1),
                    reason: "test shutdown".to_owned(),
                })
                .await
                .expect("shutdown should be queued");

            match completions
                .recv()
                .await
                .expect("forced-drain completion should arrive")
            {
                PipelineCompletionMsg::DeliverNack { nack } => {
                    assert!(!nack.permanent);
                    assert!(nack.reason.contains("bearer token unavailable"));
                }
                PipelineCompletionMsg::DeliverAck { .. } => panic!("expected transient NACK"),
            }

            drop(pdata_tx);
        };

        let (start_result, ()) = tokio::time::timeout(Duration::from_secs(2), async {
            tokio::join!(Box::new(exporter).start(msg_chan, effect_handler), driver)
        })
        .await
        .expect("exporter should finish before the shutdown deadline");
        drop(control_guard);
        let _terminal_state = start_result.expect("exporter should shut down cleanly");
    }

    /// Scenario: The start loop receives a replay after HTTP 401 or 403 and the provider publishes refreshed auth.
    /// Guarantees: The rejected auth generation gates the inbox until refreshed auth releases and ACKs the same refused payload.
    #[tokio::test]
    async fn start_loop_releases_replay_after_auth_refresh() {
        for status in [401, 403] {
            assert_releases_replay_after_auth_refresh(status).await;
        }
    }

    async fn assert_releases_replay_after_auth_refresh(status: u16) {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header("authorization", "Bearer stale-token"))
            .respond_with(ResponseTemplate::new(status))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(header("authorization", "Bearer fresh-token"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        let (provider, mut controller) = TestTokenProvider::new(Some("stale-token"));
        let exporter = exporter(&server.uri(), provider);
        let (effect_handler, mut completions) = completion_harness();
        let (control_tx, pdata_tx, msg_chan) = message_channel(4);
        // Keep the control channel alive until the exporter consumes shutdown.
        let control_guard = control_tx.clone();

        let driver = async move {
            controller.wait_until_observed().await;
            pdata_tx
                .send_async(metrics_pdata(1))
                .await
                .expect("initial pdata should be queued");
            let replay = match completions
                .recv()
                .await
                .expect("first completion should arrive")
            {
                PipelineCompletionMsg::DeliverNack { nack } => {
                    assert!(!nack.permanent);
                    assert!(nack.reason.contains(&format!("HTTP {status}")));
                    *nack.refused
                }
                PipelineCompletionMsg::DeliverAck { .. } => panic!("expected transient NACK"),
            };

            pdata_tx
                .send_async(replay)
                .await
                .expect("replay should be queued");
            controller.publish("fresh-token");
            controller.wait_until_observed().await;
            match completions
                .recv()
                .await
                .expect("replay completion should arrive")
            {
                PipelineCompletionMsg::DeliverAck { .. } => {}
                PipelineCompletionMsg::DeliverNack { .. } => panic!("expected replay ACK"),
            }

            control_tx
                .send_async(NodeControlMsg::Shutdown {
                    deadline: Instant::now(),
                    reason: "test complete".to_owned(),
                })
                .await
                .expect("shutdown should be queued");
            drop(pdata_tx);
        };

        let (start_result, ()) = tokio::time::timeout(Duration::from_secs(2), async {
            tokio::join!(Box::new(exporter).start(msg_chan, effect_handler), driver)
        })
        .await
        .expect("exporter should finish after replacement-token replay");
        drop(control_guard);
        let _terminal_state = start_result.expect("exporter should shut down cleanly");
    }
}
