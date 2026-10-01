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
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_telemetry::otel_warn;

const UNSUPPORTED_SIGNAL_MESSAGE: &str = "Geneva metrics exporter accepts metrics only";
const UNSUPPORTED_FORMAT_MESSAGE: &str = "Geneva metrics exporter accepts OTLP only";

/// Pipeline exporter that converts OTLP metrics to Geneva protocol v6 packets.
pub struct GenevaMetricsExporter {
    mapping_config: Config,
    publisher: MetricsPublisher,
    token_provider: Box<dyn LocalBearerTokenProvider>,
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
            token_provider,
        }
    }

    async fn handle_pdata(
        &self,
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

        let bearer_token = match self.token_provider.get_token().await {
            Ok(token) => token,
            Err(error) => {
                let nack = NackMsg::new(
                    format!("failed to acquire Geneva metrics bearer token: {error}"),
                    data,
                );
                return effect_handler.notify_nack(nack).await;
            }
        };
        match self
            .publisher
            .publish(&monitoring_account, packet, bearer_token.expose_token())
            .await
        {
            Ok(()) => effect_handler.notify_ack(AckMsg::new(data)).await,
            Err(error) => {
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
        self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        loop {
            match msg_chan.recv().await? {
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
    use futures::{StreamExt, stream};
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::capability::auth::BearerToken;
    use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::{
        BearerTokenProvider as BearerTokenProviderCapability, TokenStream,
    };
    use otel_arrow_dfe_engine::capability::{CapabilityError, CapabilityErrorSource};
    use otel_arrow_dfe_engine::control::{
        PipelineCompletionMsg, PipelineCompletionMsgReceiver, pipeline_completion_msg_channel,
    };
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
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;
    use std::rc::Rc;
    use std::time::Duration;
    use wiremock::matchers::{header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    struct SequenceTokenProvider {
        tokens: RefCell<VecDeque<String>>,
        calls: Rc<Cell<usize>>,
        errors: CapabilityErrorSource<BearerTokenProviderCapability>,
    }

    impl SequenceTokenProvider {
        fn new(tokens: impl IntoIterator<Item = &'static str>) -> (Self, Rc<Cell<usize>>) {
            let calls = Rc::new(Cell::new(0));
            (
                Self {
                    tokens: RefCell::new(tokens.into_iter().map(str::to_string).collect()),
                    calls: calls.clone(),
                    errors: CapabilityErrorSource::new("geneva-metrics-test".into()),
                },
                calls,
            )
        }
    }

    #[async_trait(?Send)]
    impl LocalBearerTokenProvider for SequenceTokenProvider {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            self.calls.set(self.calls.get() + 1);
            self.tokens
                .borrow_mut()
                .pop_front()
                .map(BearerToken::without_expiry)
                .ok_or_else(|| self.errors.error("no test token available"))
        }

        fn token_stream(&self) -> TokenStream {
            stream::pending().boxed()
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

    fn exporter(endpoint: &str, token_provider: SequenceTokenProvider) -> GenevaMetricsExporter {
        GenevaMetricsExporter::new(
            mapping_config(),
            MetricsPublisher::new(endpoint, Duration::from_secs(1))
                .expect("publisher should be created"),
            Box::new(token_provider),
        )
    }

    /// Scenario: One OTLP request selects two different monitoring accounts.
    /// Guarantees: The exporter permanently NACKs before acquiring a token or publishing either account.
    #[tokio::test]
    async fn rejects_multiple_accounts_before_publication() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&server)
            .await;
        let (provider, calls) = SequenceTokenProvider::new(["unused-token"]);
        let exporter = exporter(&server.uri(), provider);
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
        assert_eq!(calls.get(), 0);
    }

    /// Scenario: The bound bearer provider cannot supply a token for a valid single-account request.
    /// Guarantees: The exporter returns a transient NACK without issuing an HTTP request.
    #[tokio::test]
    async fn token_acquisition_failure_is_retryable() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&server)
            .await;
        let (provider, calls) = SequenceTokenProvider::new([]);
        let exporter = exporter(&server.uri(), provider);
        let (effect_handler, mut completions) = completion_harness();

        exporter
            .handle_pdata(metrics_pdata(&["account-a"], 1), &effect_handler)
            .await
            .expect("NACK should be routed");

        match completions.recv().await.expect("completion should arrive") {
            PipelineCompletionMsg::DeliverNack { nack } => {
                assert!(!nack.permanent);
                assert!(
                    nack.reason
                        .contains("failed to acquire Geneva metrics bearer token")
                );
            }
            PipelineCompletionMsg::DeliverAck { .. } => panic!("expected transient NACK"),
        }
        assert_eq!(calls.get(), 1);
    }

    /// Scenario: A bearer token is rejected with HTTP 401 and the replay obtains a replacement token.
    /// Guarantees: The first attempt is transiently NACKed, the replay is ACKed, and each attempt acquires a token.
    #[tokio::test]
    async fn unauthorized_request_retries_with_a_new_token() {
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
        let (provider, calls) = SequenceTokenProvider::new(["stale-token", "fresh-token"]);
        let exporter = exporter(&server.uri(), provider);
        let (effect_handler, mut completions) = completion_harness();

        exporter
            .handle_pdata(metrics_pdata(&["account-a"], 1), &effect_handler)
            .await
            .expect("first completion should be routed");
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

        exporter
            .handle_pdata(replay, &effect_handler)
            .await
            .expect("replay completion should be routed");
        match completions
            .recv()
            .await
            .expect("replay completion should arrive")
        {
            PipelineCompletionMsg::DeliverAck { .. } => {}
            PipelineCompletionMsg::DeliverNack { .. } => panic!("expected replay ACK"),
        }
        assert_eq!(calls.get(), 2);
    }
}
