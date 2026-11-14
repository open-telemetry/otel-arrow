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
    // Preserved across packets so splitting cannot restart the input timeout.
    deadline: tokio::time::Instant,
    attempt: ExporterAttempt,
    remaining: std::vec::IntoIter<EncodedPacket>,
}

struct CompletedPublication {
    data: OtapPdata,
    auth_generation: u64,
    deadline: tokio::time::Instant,
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
            Err(error) if error.is_refusal() => Err(self.attempt.refused(error)),
            Err(error) => Err(self.attempt.failed(error)),
        };
        CompletedPublication {
            data: self.data,
            auth_generation: self.auth_generation,
            deadline: self.deadline,
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

    fn refuse(
        &mut self,
        attempt: ExporterAttempt,
        nack: NackMsg<OtapPdata>,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> CompletionFuture {
        let result = Err::<(), _>(attempt.refused(()));
        _ = self.metrics.record(attempt.complete(result));
        Self::nack_completion(effect_handler, nack)
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
            let nack = NackMsg::new_permanent(UNSUPPORTED_SIGNAL_MESSAGE, data);
            return PublicationStart::Completion(self.refuse(attempt, nack, effect_handler));
        }

        // ExporterInbox force-drains pdata during shutdown even when normal
        // admission is closed, so re-check auth before doing publication work.
        if !self.auth.is_ready() {
            let nack = NackMsg::new(self.auth.not_ready_reason(), data);
            return PublicationStart::Completion(self.refuse(attempt, nack, effect_handler));
        }

        let prepared = match prepare_publication(&data, &self.mapping_config) {
            Ok(prepared) => prepared,
            Err(error) => {
                let nack = NackMsg::new_permanent(error.to_string(), data);
                return PublicationStart::Completion(self.refuse(attempt, nack, effect_handler));
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
        self.start_next_publication(
            data,
            prepared.publications.into_iter(),
            tokio::time::Instant::now() + self.publisher.timeout(),
            effect_handler,
        )
    }

    fn start_next_publication(
        &mut self,
        data: OtapPdata,
        mut remaining: std::vec::IntoIter<EncodedPacket>,
        deadline: tokio::time::Instant,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> PublicationStart {
        let Some(packet) = remaining.next() else {
            return PublicationStart::Completion(Self::ack_completion(effect_handler, data));
        };
        let mut attempt = self.metrics.attempt(SignalType::Metrics);
        attempt.set_item_count_with(|| packet.item_count);
        attempt.set_payload_size_with(|| packet.bytes.len());
        if tokio::time::Instant::now() >= deadline {
            let nack = NackMsg::new(PublishError::DeadlineExceeded.to_string(), data);
            return PublicationStart::Completion(self.refuse(attempt, nack, effect_handler));
        }
        let header = self.auth.header().filter(|_| self.auth.is_ready());
        let Some((auth_header_name, auth_header_value, auth_generation)) = header else {
            let nack = NackMsg::new(self.auth.not_ready_reason(), data);
            return PublicationStart::Completion(self.refuse(attempt, nack, effect_handler));
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
            request: Box::pin(async move {
                tokio::select! {
                    biased;
                    () = tokio::time::sleep_until(deadline) => Err(PublishError::DeadlineExceeded),
                    result = request => result,
                }
            }),
            deadline,
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
            deadline,
            remaining,
            result,
        } = publication;

        match result {
            Ok(()) => self.start_next_publication(data, remaining, deadline, effect_handler),
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
            let publication_deadline = deadline.min(publication.deadline);
            let result = if tokio::time::Instant::now() < publication_deadline {
                Some(
                    tokio::time::timeout_at(
                        publication_deadline,
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
                    let nack = NackMsg::new(
                        "Geneva metrics exporter shutdown deadline exhausted during publication",
                        publication.data,
                    );
                    pending_completion = Some(Self::nack_completion(effect_handler, nack));
                    continue;
                }
            };
            let completed = publication.complete(result, &mut self.metrics);
            if completed.result.is_ok() && !completed.remaining.as_slice().is_empty() {
                self.wait_for_shutdown_auth(publication_deadline).await;
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
            if let Some(deadline) = shutdown_drain_deadline {
                self.finish_shutdown_work(
                    in_flight.take(),
                    pending_completion.take(),
                    deadline,
                    &effect_handler,
                )
                .await?;
            }
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
                            let nack = NackMsg::new(SHUTDOWN_AT_CAPACITY_MESSAGE, data);
                            pending_completion = Some(self.refuse(attempt, nack, &effect_handler));
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
    use super::super::publication_preparation::test_support::{
        mapping_config, otap_pdata, otlp_pdata, oversized_request, single_point_request,
    };
    use super::*;
    use futures::StreamExt;
    use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
    use otel_arrow_dfe_channel::mpsc;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::capability::CapabilityError;
    use otel_arrow_dfe_engine::capability::auth::BearerToken;
    use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::TokenStream;
    use otel_arrow_dfe_engine::clock::SimClock;
    use otel_arrow_dfe_engine::control::{
        PipelineCompletionMsg, PipelineCompletionMsgReceiver, pipeline_completion_msg_channel,
    };
    use otel_arrow_dfe_engine::local::message::LocalReceiver;
    use otel_arrow_dfe_engine::message::Receiver;
    use otel_arrow_dfe_engine::node::NodeId;
    use otel_arrow_dfe_otap::testing::TestCallData;
    use otel_arrow_dfe_telemetry::metrics::{MetricSetSnapshot, MetricValue};
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
    use std::cell::{Cell, RefCell};
    use std::time::{Duration, Instant};
    use wiremock::matchers::{header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    type ControlSender = mpsc::Sender<NodeControlMsg<OtapPdata>>;
    type Completions = PipelineCompletionMsgReceiver<OtapPdata>;

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

    fn subscribe(data: OtapPdata, call_id: usize) -> OtapPdata {
        data.test_subscribe_to(
            Interests::ACKS | Interests::NACKS,
            TestCallData::default().into(),
            call_id,
        )
    }

    fn metrics_pdata(call_id: usize) -> OtapPdata {
        subscribe(otlp_pdata(single_point_request()), call_id)
    }

    fn completion_harness() -> (EffectHandler<OtapPdata>, Completions) {
        completion_harness_with_capacity(4)
    }

    fn completion_harness_with_capacity(
        capacity: usize,
    ) -> (EffectHandler<OtapPdata>, Completions) {
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

    /// A completion harness whose only slot is already full, so the next ACK
    /// or NACK blocks.
    async fn blocked_completion_harness() -> (EffectHandler<OtapPdata>, Completions) {
        let (effect_handler, completions) = completion_harness_with_capacity(1);
        effect_handler
            .notify_nack(NackMsg::new(
                "occupy the completion channel",
                metrics_pdata(0),
            ))
            .await
            .expect("completion channel should accept its first message");
        (effect_handler, completions)
    }

    async fn expect_ack(completions: &mut Completions) {
        match completions.recv().await.expect("completion should arrive") {
            PipelineCompletionMsg::DeliverAck { .. } => {}
            PipelineCompletionMsg::DeliverNack { nack } => {
                panic!("expected ACK, got NACK: {}", nack.reason)
            }
        }
    }

    async fn expect_nack(completions: &mut Completions) -> NackMsg<OtapPdata> {
        match completions.recv().await.expect("completion should arrive") {
            PipelineCompletionMsg::DeliverNack { nack } => nack,
            PipelineCompletionMsg::DeliverAck { .. } => panic!("expected NACK, got ACK"),
        }
    }

    fn message_channel(
        capacity: usize,
    ) -> (
        ControlSender,
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

    async fn send_shutdown(control_tx: &ControlSender, deadline: Instant) {
        control_tx
            .send_async(NodeControlMsg::Shutdown {
                deadline,
                reason: "test shutdown".to_owned(),
            })
            .await
            .expect("shutdown should be queued");
    }

    async fn collect_telemetry(control_tx: &ControlSender) -> Vec<MetricSetSnapshot> {
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
        collected
    }

    /// Runs the exporter alongside `driver`, failing unless both finish within
    /// `limit` and the exporter shuts down cleanly.
    async fn run_exporter<T>(
        exporter: GenevaMetricsExporter,
        msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
        limit: Duration,
        driver: impl Future<Output = T>,
    ) -> (TerminalState, T) {
        let (result, output) = tokio::time::timeout(limit, async {
            tokio::join!(Box::new(exporter).start(msg_chan, effect_handler), driver)
        })
        .await
        .expect("exporter and test driver should finish within the limit");
        (result.expect("exporter should shut down cleanly"), output)
    }

    fn exporter_with_publisher(
        publisher: MetricsPublisher,
        token_provider: TestTokenProvider,
    ) -> GenevaMetricsExporter {
        GenevaMetricsExporter::new(
            mapping_config(),
            publisher,
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
        // Generous so loaded CI cannot time out local mock-server requests.
        let publisher = MetricsPublisher::new(endpoint, Duration::from_secs(10))
            .expect("publisher should be created");
        exporter_with_publisher(publisher, token_provider)
    }

    /// Builds an exporter that has already consumed its initial bearer token.
    async fn ready_exporter(endpoint: &str) -> (GenevaMetricsExporter, TokenController) {
        let (provider, mut controller) = TestTokenProvider::new(Some("initial-token"));
        let mut exporter = exporter(endpoint, provider);
        poll_fn(|cx| exporter.auth.poll_refresh(cx, &GENEVA_METRICS_AUTH_EVENTS)).await;
        controller.wait_until_observed().await;
        (exporter, controller)
    }

    /// Builds an exporter that never touches the network. Each publication is
    /// reported on the returned stream before `request` supplies its outcome,
    /// so start-loop tests can run on paused Tokio time.
    fn scripted_exporter(
        timeout: Duration,
        token_provider: TestTokenProvider,
        request: impl Fn() -> LocalBoxFuture<'static, Result<(), PublishError>> + 'static,
    ) -> (GenevaMetricsExporter, UnboundedReceiver<()>) {
        let (started_tx, started_rx) = unbounded();
        let publisher = MetricsPublisher::scripted(timeout, move || {
            started_tx
                .unbounded_send(())
                .expect("test should observe every publication request");
            request()
        });
        (
            exporter_with_publisher(publisher, token_provider),
            started_rx,
        )
    }

    fn stalled_request() -> LocalBoxFuture<'static, Result<(), PublishError>> {
        Box::pin(std::future::pending())
    }

    /// Starts the first packet of one input split into two packets.
    fn start_two_packet_publication(
        exporter: &mut GenevaMetricsExporter,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> PublicationStart {
        let data = metrics_pdata(1);
        let packets: Vec<_> = (0..2)
            .flat_map(|_| {
                prepare_publication(&data, &mapping_config())
                    .expect("prepare packet")
                    .publications
            })
            .collect();
        exporter.start_next_publication(
            data,
            packets.into_iter(),
            tokio::time::Instant::now() + exporter.publisher.timeout(),
            effect_handler,
        )
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

    fn assert_metric(snapshots: &[MetricSetSnapshot], outcome: &str, name: &str, expected: u64) {
        assert_eq!(
            metric_value(snapshots, outcome, name),
            &MetricValue::U64(expected),
            "{outcome} {name}"
        );
    }

    /// Mounts a POST response, optionally matching one bearer token, that must be hit `times` times.
    async fn mock_post(server: &MockServer, token: Option<&str>, status: u16, times: u64) {
        let mock = Mock::given(method("POST"));
        let mock = match token {
            Some(token) => mock.and(header("authorization", format!("Bearer {token}"))),
            None => mock,
        };
        mock.respond_with(ResponseTemplate::new(status))
            .expect(times)
            .mount(server)
            .await;
    }

    /// Scenario: Native OTAP metrics are queued before the bearer provider publishes its initial token.
    /// Guarantees: The start loop publishes and ACKs native metrics after auth arrives, then reports the standard exporter measurements.
    #[tokio::test]
    async fn start_loop_waits_for_initial_auth() {
        let server = MockServer::start().await;
        mock_post(&server, None, 200, 1).await;
        let (provider, mut controller) = TestTokenProvider::new(None);
        let exporter = exporter(&server.uri(), provider);
        let (effect_handler, mut completions) = completion_harness();
        let (control_tx, pdata_tx, msg_chan) = message_channel(4);

        let driver = async {
            pdata_tx
                .send_async(subscribe(otap_pdata(single_point_request()), 1))
                .await
                .expect("pdata should be queued before auth is ready");
            controller.publish("fresh-token");
            controller.wait_until_observed().await;
            expect_ack(&mut completions).await;

            let collected = collect_telemetry(&control_tx).await;
            assert_metric(&collected, "success", "messages", 1);
            assert_metric(&collected, "success", "items", 1);
            assert!(!metric_value(&collected, "success", "payload.size").is_zero());
            assert!(!metric_value(&collected, "success", "duration").is_zero());

            send_shutdown(&control_tx, Instant::now()).await;
            drop(pdata_tx);
        };

        let (terminal_state, ()) = run_exporter(
            exporter,
            msg_chan,
            effect_handler,
            Duration::from_secs(10),
            driver,
        )
        .await;
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
        mock_post(&server, None, 200, 2).await;
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
        let deadline = Instant::now() + Duration::from_secs(60);
        send_shutdown(&control_tx, deadline).await;
        drop(pdata_tx);
        let driver = async {
            for _ in 0..2 {
                expect_ack(&mut completions).await;
            }
        };

        let (terminal, ()) = run_exporter(
            exporter,
            msg_chan,
            effect_handler,
            Duration::from_secs(10),
            driver,
        )
        .await;
        assert_eq!(terminal.deadline(), deadline);
        assert_metric(terminal.metrics(), "success", "messages", 2);
        assert_metric(terminal.metrics(), "success", "items", 2);
    }

    /// Scenario: A split input starts during shutdown draining while upstream remains open.
    /// Guarantees: The input deadline cancels the active packet and prevents later packets without waiting for another inbox message.
    #[tokio::test(start_paused = true)]
    async fn shutdown_bounds_split_publication_while_upstream_remains_open() {
        assert_shutdown_bounds_split_publication(true, true, Duration::from_secs(60)).await;
    }

    /// Scenario: A split input is the first buffered input drained with ready auth and an engine clock ahead of paused Tokio time, with upstream open or closed and either a short or long shutdown grace period.
    /// Guarantees: The first input shares one publication budget, honors an earlier delivered shutdown deadline (including channel closure), is retryably NACKed on expiry, and starts no further packets.
    #[tokio::test(start_paused = true)]
    async fn shutdown_bounds_first_buffered_split_publication() {
        for upstream_open in [true, false] {
            for grace_period in [Duration::from_secs(6), Duration::from_secs(60)] {
                assert_shutdown_bounds_split_publication(false, upstream_open, grace_period).await;
            }
        }
    }

    async fn assert_shutdown_bounds_split_publication(
        preliminary_input: bool,
        upstream_open: bool,
        grace_period: Duration,
    ) {
        let clock = SimClock::new();
        let _clock_guard = clock.install();
        let tokio_now = tokio::time::Instant::now();
        // Model setup outlasting the grace period while Tokio time is paused.
        clock.advance_to(tokio_now.into_std() + grace_period + Duration::from_secs(1));
        // Preserve Tokio's timer alignment even if the system clock is further ahead.
        let alignment = clock.now().duration_since(tokio_now.into_std());
        tokio::time::advance(Duration::from_secs(alignment.as_secs() + 1)).await;
        clock.advance_to(tokio::time::Instant::now().into_std());

        let split_data = subscribe(otlp_pdata(oversized_request()), 2);
        let publisher_timeout = Duration::from_secs(10);
        let (provider, _controller) = TestTokenProvider::new(Some("ready-token"));
        let calls = Cell::new(0_u32);
        let (exporter, mut requests) = scripted_exporter(publisher_timeout, provider, move || {
            let delay = if calls.replace(calls.get() + 1) == 0 {
                Duration::from_secs(4)
            } else {
                Duration::from_secs(8)
            };
            Box::pin(async move {
                tokio::time::sleep(delay).await;
                Ok(())
            })
        });
        let (effect_handler, mut completions) = completion_harness();
        let (control_tx, pdata_tx, msg_chan) = message_channel(4);
        if preliminary_input {
            pdata_tx
                .send_async(metrics_pdata(1))
                .await
                .expect("first input should be buffered");
        }
        pdata_tx
            .send_async(split_data)
            .await
            .expect("split input should be buffered");
        let deadline = (tokio::time::Instant::now() + grace_period).into_std();
        send_shutdown(&control_tx, deadline).await;
        let pdata_tx = upstream_open.then_some(pdata_tx);

        let driver = async {
            requests
                .next()
                .await
                .expect("the first input should start a publication");
            let first_started_at = tokio::time::Instant::now();
            if upstream_open {
                requests
                    .next()
                    .await
                    .expect("a second publication should start");
                if preliminary_input {
                    expect_ack(&mut completions).await;
                }
            }
            let nack = expect_nack(&mut completions).await;
            assert!(!nack.permanent, "deadline expiry should allow retry");
            let budget_deadline = first_started_at + publisher_timeout.min(grace_period);
            if upstream_open {
                assert_eq!(
                    tokio::time::Instant::now(),
                    budget_deadline,
                    "the split packet should be cancelled at the shared input or delivered shutdown deadline"
                );
            } else {
                assert!(
                    tokio::time::Instant::now() <= budget_deadline,
                    "channel closure may deliver an earlier shutdown, but must not extend publication"
                );
            }
            drop(pdata_tx);
        };

        let (terminal, ()) = {
            let work = run_exporter(
                exporter,
                msg_chan,
                effect_handler,
                Duration::from_secs(600),
                driver,
            );
            tokio::pin!(work);
            // Mirror the production inbox timer so paused Tokio time wakes the
            // independent engine clock at the pipeline deadline, not at the
            // next HTTP timer.
            let pipeline_timer = tokio::time::sleep_until(tokio::time::Instant::from_std(deadline));
            tokio::pin!(pipeline_timer);
            poll_fn(|cx| {
                let _ = pipeline_timer.as_mut().poll(cx);
                clock.advance_to(tokio::time::Instant::now().into_std());
                work.as_mut().poll(cx)
            })
            .await
        };
        if upstream_open {
            assert_eq!(terminal.deadline(), deadline);
            assert_metric(terminal.metrics(), "success", "messages", 1);
        } else {
            assert!(
                terminal.deadline() <= deadline,
                "the exporter must honor the earlier shutdown delivered for channel closure"
            );
        }
        assert_metric(terminal.metrics(), "failure", "messages", 1);
        assert!(
            requests.next().await.is_none(),
            "no later packet of the split input may start"
        );
    }

    /// Scenario: A normal input splits into two packets whose individual delays fit the timeout but whose combined duration may exceed it.
    /// Guarantees: Both packets share one deadline, expiry produces exactly one retryable NACK, and the next input receives a fresh budget.
    #[tokio::test(start_paused = true)]
    async fn split_publications_share_input_timeout_during_normal_operation() {
        for second_delay in [Duration::from_secs(5), Duration::from_secs(8)] {
            let timeout = Duration::from_secs(10);
            let (provider, _controller) = TestTokenProvider::new(Some("ready-token"));
            let calls = Cell::new(0);
            let (exporter, mut requests) = scripted_exporter(timeout, provider, move || {
                let delay = match calls.replace(calls.get() + 1) {
                    0 => Duration::from_secs(4),
                    1 => second_delay,
                    _ => Duration::ZERO,
                };
                Box::pin(async move {
                    tokio::time::sleep(delay).await;
                    Ok(())
                })
            });
            let split_data = subscribe(otlp_pdata(oversized_request()), 1);
            let (effect_handler, mut completions) = completion_harness();
            let (control_tx, pdata_tx, msg_chan) = message_channel(4);
            pdata_tx
                .send_async(split_data)
                .await
                .expect("split input should be buffered");

            let expires = Duration::from_secs(4) + second_delay > timeout;
            let driver = async {
                requests.next().await.expect("first packet should start");
                let started_at = tokio::time::Instant::now();
                requests.next().await.expect("second packet should start");
                if expires {
                    let nack = expect_nack(&mut completions).await;
                    assert!(!nack.permanent);
                    assert!(nack.reason.contains("input publication deadline exhausted"));
                } else {
                    expect_ack(&mut completions).await;
                }
                assert_eq!(
                    started_at.elapsed(),
                    timeout.min(Duration::from_secs(4) + second_delay),
                );
                pdata_tx
                    .send_async(metrics_pdata(2))
                    .await
                    .expect("next input should be accepted");
                requests.next().await.expect("next input should publish");
                expect_ack(&mut completions).await;
                send_shutdown(&control_tx, Instant::now()).await;
                drop(pdata_tx);
            };
            _ = run_exporter(
                exporter,
                msg_chan,
                effect_handler,
                Duration::from_secs(60),
                driver,
            )
            .await;
            assert!(
                requests.next().await.is_none(),
                "each input must resolve before accepting the next input",
            );
            assert!(
                completions.try_recv().is_err(),
                "each input must receive exactly one completion",
            );
        }
    }

    /// Scenario: Cached authentication becomes unusable and a replacement arrives during or between shutdown publications.
    /// Guarantees: Shutdown consumes refreshes while awaiting HTTP, waits for credentials between packets, and ACKs the entire input after both packets succeed.
    #[tokio::test]
    async fn shutdown_refreshes_auth_while_draining_split_publications() {
        for refresh_during_request in [true, false] {
            let server = MockServer::start().await;
            for token in ["initial-token", "fresh-token"] {
                mock_post(&server, Some(token), 200, 1).await;
            }
            let (mut exporter, mut controller) = ready_exporter(&server.uri()).await;
            let (effect_handler, mut completions) = completion_harness();
            let PublicationStart::InFlight(mut publication) =
                start_two_packet_publication(&mut exporter, &effect_handler)
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
            expect_ack(&mut completions).await;
            let snapshots = exporter.metrics.terminal_snapshots();
            assert_metric(&snapshots, "success", "messages", 2);
        }
    }

    /// Scenario: An in-flight publication is cancelled at an expired or subsequently reached shutdown deadline.
    /// Guarantees: Cancellation sends a retryable NACK when completion delivery is ready without extending the deadline.
    #[tokio::test(start_paused = true)]
    async fn shutdown_nacks_cancelled_publication() {
        for remaining in [Duration::ZERO, Duration::from_secs(1)] {
            let (mut exporter, _controller) = ready_exporter("http://127.0.0.1:1").await;
            let (effect_handler, mut completions) = completion_harness();
            let PublicationStart::InFlight(mut publication) =
                exporter.start_publication(metrics_pdata(1), &effect_handler)
            else {
                panic!("publication should start");
            };
            publication.request = stalled_request();
            let deadline = tokio::time::Instant::now() + remaining;
            tokio::time::timeout(
                Duration::from_secs(10),
                exporter.finish_shutdown_work(
                    Some(publication),
                    None,
                    deadline.into_std(),
                    &effect_handler,
                ),
            )
            .await
            .expect("shutdown must remain bounded")
            .expect("shutdown should finish");
            assert_eq!(tokio::time::Instant::now(), deadline);
            let nack = tokio::time::timeout(Duration::from_secs(1), expect_nack(&mut completions))
                .await
                .expect("cancelled input should complete");
            assert!(!nack.permanent);
            assert!(nack.reason.contains("shutdown deadline exhausted"));
        }
    }

    /// Scenario: A split publication needs refreshed authentication, but no replacement arrives before shutdown expires.
    /// Guarantees: Waiting respects the original virtual deadline, never submits the next packet, and retryably NACKs the input.
    #[tokio::test(start_paused = true)]
    async fn shutdown_bounds_wait_for_refreshed_auth() {
        let (mut exporter, _controller) = ready_exporter("http://127.0.0.1:1").await;
        let (effect_handler, mut completions) = completion_harness();
        let PublicationStart::InFlight(mut publication) =
            start_two_packet_publication(&mut exporter, &effect_handler)
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
        assert!(
            !expect_nack(&mut completions).await.permanent,
            "unsubmitted packets must be retryable"
        );
        let snapshots = exporter.metrics.terminal_snapshots();
        assert_metric(&snapshots, "success", "messages", 1);
        assert_metric(&snapshots, "refused", "messages", 1);
    }

    /// Scenario: One input is split into two publications and the second succeeds or fails.
    /// Guarantees: Completion waits for both packets; retryable refusals stay retryable and each attempt has the correct telemetry outcome.
    #[tokio::test]
    async fn split_publications_complete_once_and_record_each_attempt() {
        for final_status in [200, 503, 400, 401, 403, 408, 429] {
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
            let (mut exporter, _controller) = ready_exporter(&server.uri()).await;
            let (effect_handler, mut completions) = completion_harness();
            let mut work = start_two_packet_publication(&mut exporter, &effect_handler);
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
            if final_status == 200 {
                expect_ack(&mut completions).await;
            } else {
                let nack = expect_nack(&mut completions).await;
                assert_eq!(nack.permanent, final_status == 400);
            }
            let snapshots = exporter.metrics.terminal_snapshots();
            let successes = if final_status == 200 { 2 } else { 1 };
            assert_metric(&snapshots, "success", "messages", successes);
            assert_metric(&snapshots, "success", "items", successes);
            if final_status != 200 {
                let outcome = if matches!(final_status, 400 | 401 | 403 | 429) {
                    "refused"
                } else {
                    "failure"
                };
                assert_metric(&snapshots, outcome, "messages", 1);
            }
            assert!(matches!(
                completions.try_recv(),
                Err(otel_arrow_dfe_channel::error::RecvError::Empty),
            ));
        }
    }

    /// Scenario: Shutdown force-drains pdata while one publication is stalled beyond the hidden pipeline deadline, with completion delivery free or blocked.
    /// Guarantees: Only one request is in flight, force-drained pdata is retryably NACKed, the local timeout bounds
    /// publication work and completion backpressure, and the terminal state preserves the pipeline deadline.
    #[tokio::test(start_paused = true)]
    async fn shutdown_bounds_stalled_publication_and_force_drained_pdata() {
        for blocked_completions in [false, true] {
            let publisher_timeout = Duration::from_secs(60);
            let (provider, mut controller) = TestTokenProvider::new(Some("ready-token"));
            let (exporter, mut requests) =
                scripted_exporter(publisher_timeout, provider, stalled_request);
            let (effect_handler, mut completions) = if blocked_completions {
                blocked_completion_harness().await
            } else {
                completion_harness()
            };
            let (control_tx, pdata_tx, msg_chan) = message_channel(4);

            let driver = async {
                controller.wait_until_observed().await;
                pdata_tx
                    .send_async(metrics_pdata(1))
                    .await
                    .expect("first pdata should be queued");
                requests
                    .next()
                    .await
                    .expect("first pdata should start a publication");
                let request_started_at = tokio::time::Instant::now();
                pdata_tx
                    .send_async(metrics_pdata(2))
                    .await
                    .expect("second pdata should be buffered behind the active publication");
                // The pipeline deadline is hidden while the second input is force-drained.
                let deadline = (request_started_at + Duration::from_secs(30)).into_std();
                send_shutdown(&control_tx, deadline).await;
                if !blocked_completions {
                    loop {
                        let nack = expect_nack(&mut completions).await;
                        assert!(!nack.permanent);
                        if nack.reason.contains(SHUTDOWN_AT_CAPACITY_MESSAGE) {
                            break;
                        }
                    }
                }
                (deadline, request_started_at)
            };

            let (terminal_state, (deadline, request_started_at)) = run_exporter(
                exporter,
                msg_chan,
                effect_handler,
                Duration::from_secs(600),
                driver,
            )
            .await;
            assert_eq!(terminal_state.deadline(), deadline);
            assert_eq!(
                tokio::time::Instant::now(),
                request_started_at + publisher_timeout
            );
            assert!(
                requests.next().await.is_none(),
                "only one publication may be in flight"
            );
        }
    }

    /// Scenario: Shutdown drains two inputs without authentication while completion delivery is blocked.
    /// Guarantees: One local timeout bounds authentication waits and NACK backpressure across the entire drain,
    /// no publication starts, and the terminal pipeline deadline is unchanged.
    #[tokio::test(start_paused = true)]
    async fn shutdown_draining_shares_local_timeout_across_buffered_input() {
        let drain_timeout = Duration::from_secs(3);
        let (provider, _controller) = TestTokenProvider::new(None);
        let (exporter, mut requests) = scripted_exporter(drain_timeout, provider, stalled_request);
        let (effect_handler, _completions) = blocked_completion_harness().await;
        let (control_tx, pdata_tx, msg_chan) = message_channel(4);
        for call_id in [1, 2] {
            pdata_tx
                .send_async(metrics_pdata(call_id))
                .await
                .expect("input should be buffered before shutdown");
        }
        let deadline = Instant::now() + Duration::from_secs(60);
        send_shutdown(&control_tx, deadline).await;
        drop(pdata_tx);

        let started_at = tokio::time::Instant::now();
        let (terminal_state, ()) = run_exporter(
            exporter,
            msg_chan,
            effect_handler,
            Duration::from_secs(10),
            async {},
        )
        .await;
        assert_eq!(started_at.elapsed(), drain_timeout);
        assert_eq!(terminal_state.deadline(), deadline);
        assert!(
            requests.next().await.is_none(),
            "no publication may start without credentials"
        );
    }

    /// Scenario: A request completes before shutdown while its ACK is blocked by a full completion channel.
    /// Guarantees: The blocked ACK remains pending work and cannot prevent the exporter from observing the deadline.
    #[tokio::test(start_paused = true)]
    async fn shutdown_bounds_completed_request_completion_backpressure() {
        let (provider, mut controller) = TestTokenProvider::new(Some("ready-token"));
        let (exporter, mut requests) = scripted_exporter(Duration::from_secs(60), provider, || {
            Box::pin(std::future::ready(Ok(())))
        });
        let (effect_handler, _completions) = blocked_completion_harness().await;
        let (control_tx, pdata_tx, msg_chan) = message_channel(4);

        let driver = async {
            controller.wait_until_observed().await;
            pdata_tx
                .send_async(metrics_pdata(1))
                .await
                .expect("pdata should be queued");
            // The ready request resolves and queues its blocked ACK before the
            // exporter yields back to this driver.
            requests
                .next()
                .await
                .expect("pdata should start a publication");
            // Far ahead of the real clock the inbox checks.
            let deadline = (tokio::time::Instant::now() + Duration::from_secs(30)).into_std();
            send_shutdown(&control_tx, deadline).await;
            deadline
        };

        let (terminal_state, deadline) = run_exporter(
            exporter,
            msg_chan,
            effect_handler,
            Duration::from_secs(600),
            driver,
        )
        .await;
        assert_eq!(terminal_state.deadline(), deadline);
        assert_eq!(
            tokio::time::Instant::now(),
            tokio::time::Instant::from_std(deadline)
        );
        assert!(
            requests.next().await.is_none(),
            "only one publication may start"
        );
    }

    /// Scenario: The token stream closes after HTTP 401 while refused pdata is buffered and shutdown begins.
    /// Guarantees: The exporter avoids a closed-stream busy loop, keeps shutdown responsive, and never reacquires or republishes the rejected token.
    #[tokio::test]
    async fn closed_token_stream_keeps_shutdown_responsive() {
        let server = MockServer::start().await;
        mock_post(&server, None, 401, 1).await;
        let (provider, mut controller) = TestTokenProvider::new(Some("stale-token"));
        let exporter = exporter(&server.uri(), provider);
        let (effect_handler, mut completions) = completion_harness();
        let (control_tx, pdata_tx, msg_chan) = message_channel(4);

        let driver = async {
            controller.wait_until_observed().await;
            pdata_tx
                .send_async(metrics_pdata(1))
                .await
                .expect("initial pdata should be queued");
            let nack = expect_nack(&mut completions).await;
            assert!(!nack.permanent);
            pdata_tx
                .send_async(*nack.refused)
                .await
                .expect("replay should be queued");
            drop(controller);
            send_shutdown(&control_tx, Instant::now() + Duration::from_secs(60)).await;

            let nack = expect_nack(&mut completions).await;
            assert!(!nack.permanent);
            assert!(nack.reason.contains("bearer token unavailable"));
            drop(pdata_tx);
        };

        let _ = run_exporter(
            exporter,
            msg_chan,
            effect_handler,
            Duration::from_secs(10),
            driver,
        )
        .await;
    }

    /// Scenario: The start loop receives a replay after HTTP 401 or 403 and the provider publishes refreshed auth.
    /// Guarantees: The rejected auth generation gates the inbox until refreshed auth releases and ACKs the same refused payload.
    #[tokio::test]
    async fn start_loop_releases_replay_after_auth_refresh() {
        for status in [401, 403] {
            let server = MockServer::start().await;
            mock_post(&server, Some("stale-token"), status, 1).await;
            mock_post(&server, Some("fresh-token"), 200, 1).await;
            let (provider, mut controller) = TestTokenProvider::new(Some("stale-token"));
            let exporter = exporter(&server.uri(), provider);
            let (effect_handler, mut completions) = completion_harness();
            let (control_tx, pdata_tx, msg_chan) = message_channel(4);

            let driver = async {
                controller.wait_until_observed().await;
                pdata_tx
                    .send_async(metrics_pdata(1))
                    .await
                    .expect("initial pdata should be queued");
                let nack = expect_nack(&mut completions).await;
                assert!(!nack.permanent);
                assert!(nack.reason.contains(&format!("HTTP {status}")));
                pdata_tx
                    .send_async(*nack.refused)
                    .await
                    .expect("replay should be queued");
                controller.publish("fresh-token");
                controller.wait_until_observed().await;
                expect_ack(&mut completions).await;

                send_shutdown(&control_tx, Instant::now()).await;
                drop(pdata_tx);
            };

            let _ = run_exporter(
                exporter,
                msg_chan,
                effect_handler,
                Duration::from_secs(10),
                driver,
            )
            .await;
        }
    }
}
