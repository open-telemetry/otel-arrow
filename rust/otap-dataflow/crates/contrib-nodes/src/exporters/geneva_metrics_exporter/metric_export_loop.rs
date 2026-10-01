// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::client::MetricsPublisher;
use super::otlp_to_geneva::Config;
use super::publication_preparation::prepare_publication;
use async_trait::async_trait;
use otel_arrow_dfe_config::{SignalFormat, SignalType};
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
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_telemetry::otel_warn;
use std::future::poll_fn;
use std::time::Instant;

const UNSUPPORTED_SIGNAL_MESSAGE: &str = "Geneva metrics exporter accepts metrics only";
const UNSUPPORTED_FORMAT_MESSAGE: &str = "Geneva metrics exporter accepts OTLP only";

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

/// Pipeline exporter that converts OTLP metrics to Geneva protocol v6 packets.
pub struct GenevaMetricsExporter {
    mapping_config: Config,
    publisher: MetricsPublisher,
    auth: Box<dyn HttpClientAuthProvider>,
}

impl GenevaMetricsExporter {
    pub(crate) fn new(
        mapping_config: Config,
        publisher: MetricsPublisher,
        token_provider: Box<dyn LocalBearerTokenProvider>,
    ) -> Self {
        Self {
            mapping_config,
            publisher,
            auth: Box::new(new_http_client_auth_provider_from_bearer_token_provider(
                token_provider,
            )),
        }
    }

    async fn handle_pdata(
        &mut self,
        data: OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        let signal_type = data.signal_type();
        let signal_format = data.signal_format();
        if signal_type != SignalType::Metrics {
            let nack = NackMsg::new_permanent(UNSUPPORTED_SIGNAL_MESSAGE, data);
            return effect_handler.notify_nack(nack).await;
        }
        if signal_format != SignalFormat::OtlpBytes {
            let nack = NackMsg::new_permanent(UNSUPPORTED_FORMAT_MESSAGE, data);
            return effect_handler.notify_nack(nack).await;
        }

        // ExporterInbox force-drains pdata during shutdown even when normal
        // admission is closed, so re-check auth before doing publication work.
        if !self.auth.is_ready() {
            let nack = NackMsg::new(self.auth.not_ready_reason(), data);
            return effect_handler.notify_nack(nack).await;
        }

        let prepared = match prepare_publication(&data, &self.mapping_config) {
            Ok(prepared) => prepared,
            Err(error) => {
                let nack = NackMsg::new_permanent(error.to_string(), data);
                return effect_handler.notify_nack(nack).await;
            }
        };
        if prepared.rejected_data_points > 0 || prepared.cardinality_overflows > 0 {
            otel_warn!(
                "geneva_metrics_exporter.mapping.partial",
                rejected_data_points = prepared.rejected_data_points,
                cardinality_overflows = prepared.cardinality_overflows,
            );
        }
        let Some((monitoring_account, packet)) = prepared.publication else {
            return effect_handler.notify_ack(AckMsg::new(data)).await;
        };

        let Some((auth_header_name, auth_header_value, auth_generation)) = self.auth.header()
        else {
            let nack = NackMsg::new(self.auth.not_ready_reason(), data);
            return effect_handler.notify_nack(nack).await;
        };
        match self
            .publisher
            .publish(
                &monitoring_account,
                packet,
                auth_header_name,
                auth_header_value,
            )
            .await
        {
            Ok(()) => effect_handler.notify_ack(AckMsg::new(data)).await,
            Err(error) => {
                if error.is_unauthorized() {
                    self.auth.invalidate(auth_generation);
                }
                let reason = format!(
                    "failed to publish Geneva metrics for account {monitoring_account}: {error}"
                );
                let nack = if error.is_retryable() {
                    NackMsg::new(reason, data)
                } else {
                    NackMsg::new_permanent(reason, data)
                };
                effect_handler.notify_nack(nack).await
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

        loop {
            let accepting_pdata = self.auth.is_ready();
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

                () = async {
                    _ = poll_fn(|cx| self.auth.poll_refresh(cx, &GENEVA_METRICS_AUTH_EVENTS)).await;
                }, if self.auth.is_active() => {
                    continue;
                }

                msg = msg_chan.recv_when(accepting_pdata) => msg?,
            };

            match msg {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                Message::PData(data) => {
                    self.handle_pdata(data, &effect_handler).await?;
                }
                _ => {}
            }
        }

        Ok(TerminalState::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
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
    use otel_arrow_dfe_pdata::OtlpProtoBytes;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue, any_value};
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric, number_data_point,
    };
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
    use prost::Message as _;
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

    fn metrics_pdata(accounts: &[&str], call_id: usize) -> OtapPdata {
        let account_attribute = |account: &str| KeyValue {
            key: "_microsoft_metrics_account".to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(account.to_string())),
            }),
        };
        let point = |account: &str| NumberDataPoint {
            attributes: vec![account_attribute(account)],
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
                            data_points: accounts.iter().map(|account| point(account)).collect(),
                        })),
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let mut bytes = Vec::new();
        request
            .encode(&mut bytes)
            .expect("request should serialize");
        OtapPdata::new_default(OtlpProtoBytes::ExportMetricsRequest(Bytes::from(bytes)).into())
            .test_subscribe_to(
                Interests::ACKS | Interests::NACKS,
                TestCallData::default().into(),
                call_id,
            )
    }

    fn completion_harness() -> (
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
        let (completion_tx, completion_rx) = pipeline_completion_msg_channel(4);
        effect_handler.set_pipeline_completion_msg_sender(completion_tx);
        (effect_handler, completion_rx)
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

    fn exporter(endpoint: &str, token_provider: TestTokenProvider) -> GenevaMetricsExporter {
        GenevaMetricsExporter::new(
            mapping_config(),
            MetricsPublisher::new(endpoint, Duration::from_secs(1))
                .expect("publisher should be created"),
            Box::new(token_provider),
        )
    }

    async fn ready_exporter(endpoint: &str) -> GenevaMetricsExporter {
        let (provider, mut controller) = TestTokenProvider::new(Some("ready-token"));
        let mut exporter = exporter(endpoint, provider);
        assert!(poll_fn(|cx| exporter.auth.poll_refresh(cx, &GENEVA_METRICS_AUTH_EVENTS)).await);
        controller.wait_until_observed().await;
        exporter
    }

    /// Scenario: One OTLP request selects two different monitoring accounts.
    /// Guarantees: The exporter permanently NACKs without issuing an HTTP request.
    #[tokio::test]
    async fn rejects_multiple_accounts_before_publication() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&server)
            .await;
        let mut exporter = ready_exporter(&server.uri()).await;
        let (effect_handler, mut completions) = completion_harness();

        exporter
            .handle_pdata(
                metrics_pdata(&["account-a", "account-b"], 1),
                &effect_handler,
            )
            .await
            .expect("NACK should be routed");

        match completions.recv().await.expect("completion should arrive") {
            PipelineCompletionMsg::DeliverNack { nack } => {
                assert!(nack.permanent);
                assert!(
                    nack.reason
                        .contains("supports one monitoring account per OTLP request")
                );
            }
            PipelineCompletionMsg::DeliverAck { .. } => panic!("expected permanent NACK"),
        }
    }

    /// Scenario: Pdata is queued before the bearer provider publishes its initial token.
    /// Guarantees: The start loop backpressures the inbox and ACKs that same pdata after auth becomes ready.
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
                .send_async(metrics_pdata(&["account-a"], 1))
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
        let _terminal_state = start_result.expect("exporter should shut down cleanly");
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
                .send_async(metrics_pdata(&["account-a"], 1))
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

    /// Scenario: The start loop receives a replay after HTTP 401 and the provider publishes refreshed auth.
    /// Guarantees: The rejected auth generation gates the inbox until refreshed auth releases and ACKs the same refused payload.
    #[tokio::test]
    async fn start_loop_releases_replay_after_auth_refresh() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header("authorization", "Bearer stale-token"))
            .respond_with(ResponseTemplate::new(401))
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
                .send_async(metrics_pdata(&["account-a"], 1))
                .await
                .expect("initial pdata should be queued");
            let replay = match completions
                .recv()
                .await
                .expect("first completion should arrive")
            {
                PipelineCompletionMsg::DeliverNack { nack } => {
                    assert!(!nack.permanent);
                    assert!(nack.reason.contains("HTTP 401"));
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
