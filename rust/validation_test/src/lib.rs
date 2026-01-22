// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validation test module to validate the encoding/decoding process for otlp messages

// ToDo: Support transformative processors in a pipeline,
// we should be able to know when the assert equivalent will fail

use otap_df_config::PipelineGroupId;
use otap_df_config::PipelineId;
use otap_df_config::pipeline::PipelineConfig;
use otap_df_engine::PipelineFactory;
use otap_df_engine::context::ControllerContext;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{PipelineControlMsg, pipeline_ctrl_msg_channel};
use otap_df_otap::otlp_grpc::OTLPData;
use otap_df_pdata::otap::{OtapArrowRecords, from_record_messages};
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::logs_service_client::LogsServiceClient;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::logs_service_server::LogsServiceServer;
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::metrics_service_client::MetricsServiceClient;
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::metrics_service_server::MetricsServiceServer;
use otap_df_pdata::proto::opentelemetry::collector::trace::v1::trace_service_client::TraceServiceClient;
use otap_df_pdata::proto::opentelemetry::collector::trace::v1::trace_service_server::TraceServiceServer;
use otap_df_pdata::proto::opentelemetry::collector::{
    logs::v1::{
        ExportLogsServiceRequest, ExportLogsServiceResponse, logs_service_server::LogsService,
    },
    metrics::v1::{
        ExportMetricsServiceRequest, ExportMetricsServiceResponse,
        metrics_service_server::MetricsService,
    },
    trace::v1::{
        ExportTraceServiceRequest, ExportTraceServiceResponse, trace_service_server::TraceService,
    },
};
use otap_df_pdata::testing::round_trip::{otap_to_otlp, otlp_to_otap};
use otap_df_pdata::{Consumer, Producer};
use otap_df_state::{DeployedPipelineKey, store::ObservedStateStore};
use otap_df_telemetry::MetricsSystem;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, timeout};
use tonic::transport::server::{Router, Server};
use tonic::{Request, Response, Status};

const GRPC_INPUT_ENDPOINT: &str = "http://127.0.0.1:4317";

const CONNECTION_MAX_RETRIES: usize = 20;
const CONNECTION_RETRY_DELAY: Duration = Duration::from_millis(100);

async fn connect_with_retry<T, E, F, Fut>(
    connect_fn: F,
    max_retries: usize,
    retry_delay: Duration,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut last_error = None;
    for attempt in 0..max_retries {
        match connect_fn().await {
            Ok(client) => return Ok(client),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries - 1 {
                    tokio::time::sleep(retry_delay).await;
                }
            }
        }
    }
    Err(last_error.unwrap())
}
const GRPC_OUTPUT_ENDPOINT: &str = "127.0.0.1:4318";

const DEFAULT_CORE_ID: usize = 0;
const DEFAULT_THREAD_ID: usize = 0;
const DEFAULT_PIPELINE_CTRL_MSG_CHANNEL_SIZE: usize = 100;

/// struct to simulate the otel arrow protocol, uses a producer and consumer to encode and decode a otlp request
pub struct OtelProtoSimulator {
    producer: Producer,
    consumer: Consumer,
}

impl OtelProtoSimulator {
    /// Takes the Otlp request message and encodes it and decodes it via producer -> consumer
    pub fn simulate_proto(&mut self, proto_message: &OtlpProtoMessage) -> OtlpProtoMessage {
        // take otlp proto message
        // convert to otap arrow records which we can pass to the producer
        let mut otap_message = otlp_to_otap(proto_message);
        // convert to batch arrow records
        // converg batch arrow records
        // convert msg to proto bytes?
        let mut bar = self.producer.produce_bar(&mut otap_message).unwrap();
        let records = self.consumer.consume_bar(&mut bar).unwrap();
        let otap_message = match proto_message {
            OtlpProtoMessage::Logs(_) => OtapArrowRecords::Logs(from_record_messages(records)),
            OtlpProtoMessage::Metrics(_) => {
                OtapArrowRecords::Metrics(from_record_messages(records))
            }
            OtlpProtoMessage::Traces(_) => OtapArrowRecords::Traces(from_record_messages(records)),
        };
        otap_to_otlp(&otap_message)
    }
}

/// struct to simulate a pipeline running, reads a config and starts a pipeline to send and receive data
pub struct PipelineSimulator<PData: 'static + Clone + Send + Sync + std::fmt::Debug> {
    pipeline_factory: &'static PipelineFactory<PData>,
    pipeline_context: PipelineContext,
    pipeline_id: PipelineId,
    pipeline_group_id: PipelineGroupId,
    metrics_system: MetricsSystem,
    pipeline_key: DeployedPipelineKey,
}

impl<PData: 'static + Clone + Send + Sync + std::fmt::Debug> PipelineSimulator<PData> {
    // if pipeline alters the data via a processor that performs some transofmration we should expect the equivalent assert to fail
    // otherwise the assert should succeed

    pub fn new(pipeline_factory: &'static PipelineFactory<PData>) -> Self {
        let core_id = DEFAULT_CORE_ID;
        let thread_id = DEFAULT_THREAD_ID;
        let metrics_system = MetricsSystem::default();
        let controller_context = ControllerContext::new(metrics_system.registry());
        let pipeline_id = PipelineId::default();
        let pipeline_group_id = PipelineGroupId::default();
        let pipeline_context = controller_context.pipeline_context_with(
            pipeline_group_id.clone(),
            pipeline_id.clone(),
            core_id,
            thread_id,
        );

        let pipeline_key = DeployedPipelineKey {
            pipeline_group_id: pipeline_group_id.clone(),
            pipeline_id: pipeline_id.clone(),
            core_id,
        };

        Self {
            pipeline_factory,
            pipeline_context,
            pipeline_id,
            pipeline_group_id,
            metrics_system,
            pipeline_key,
        }
    }
    pub async fn simulate_pipeline(
        &self,
        proto_messages: &[OtlpProtoMessage],
        config_file_path: PathBuf,
    ) -> Result<Vec<OtlpProtoMessage>, String> {
        // start a grpc client to send messages to the receiver
        // start a grpc server to receiver messages from the exporter
        let (sender, mut receiver) = tokio::sync::mpsc::channel(64);
        let grpc_server = self.create_grpc_server(sender);

        // Create shutdown signal for gRPC server
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // start gRPC server in the Tokio runtime
        let addr: std::net::SocketAddr = GRPC_OUTPUT_ENDPOINT.parse().expect("endpoint is valid");
        let _grpc_server_task = tokio::spawn(async move {
            grpc_server
                .serve_with_shutdown(addr, async {
                    shutdown_rx.await.ok();
                })
                .await
                .expect("failed to start up grpc server");
        });

        // Create the pipeline control channel outside the thread so we can send shutdown
        let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) =
            pipeline_ctrl_msg_channel::<PData>(DEFAULT_PIPELINE_CTRL_MSG_CHANNEL_SIZE);
        // Clone the sender before moving into the thread so we can use it for shutdown
        let shutdown_sender = pipeline_ctrl_msg_tx.clone();

        let pipeline_factory = self.pipeline_factory;
        let pipeline_context = self.pipeline_context.clone();
        let pipeline_key = self.pipeline_key.clone();
        let metrics_reporter = self.metrics_system.reporter();
        let pipeline_group_id = self.pipeline_group_id.clone();
        let pipeline_id = self.pipeline_id.clone();
        let _pipeline_thread = std::thread::spawn(move || {
            // build pipeline and run
            let pipeline_config = PipelineConfig::from_file(
                pipeline_group_id.clone(),
                pipeline_id.clone(),
                config_file_path,
            )
            .expect("invalid config");
            let obs_state_store = ObservedStateStore::new(pipeline_config.pipeline_settings());
            let obs_evt_reporter = obs_state_store.reporter();
            let pipeline_runtime = pipeline_factory
                .build(pipeline_context.clone(), pipeline_config.clone())
                .expect("failed to create runtime");
            pipeline_runtime
                .run_forever(
                    pipeline_key,
                    pipeline_context,
                    obs_evt_reporter,
                    metrics_reporter,
                    pipeline_ctrl_msg_tx,
                    pipeline_ctrl_msg_rx,
                )
                .expect("failed to start pipeline");
        });

        // start sending messages to the pipeline
        // Use retry logic to wait for the pipeline's gRPC receiver to be ready
        let mut logs_client = connect_with_retry(
            || LogsServiceClient::connect(GRPC_INPUT_ENDPOINT),
            CONNECTION_MAX_RETRIES,
            CONNECTION_RETRY_DELAY,
        )
        .await
        .map_err(|e| e.to_string())?;

        let mut traces_client = connect_with_retry(
            || TraceServiceClient::connect(GRPC_INPUT_ENDPOINT),
            CONNECTION_MAX_RETRIES,
            CONNECTION_RETRY_DELAY,
        )
        .await
        .map_err(|e| e.to_string())?;

        let mut metrics_client = connect_with_retry(
            || MetricsServiceClient::connect(GRPC_INPUT_ENDPOINT),
            CONNECTION_MAX_RETRIES,
            CONNECTION_RETRY_DELAY,
        )
        .await
        .map_err(|e| e.to_string())?;

        for proto_message in proto_messages {
            match proto_message {
                OtlpProtoMessage::Logs(logs) => {
                    logs_client
                        .export(ExportLogsServiceRequest::new(logs.resource_logs.clone()))
                        .await
                        .map_err(|e| e.to_string())?;
                }
                OtlpProtoMessage::Metrics(metrics) => {
                    metrics_client
                        .export(ExportMetricsServiceRequest::new(
                            metrics.resource_metrics.clone(),
                        ))
                        .await
                        .map_err(|e| e.to_string())?;
                }
                OtlpProtoMessage::Traces(traces) => {
                    traces_client
                        .export(ExportTraceServiceRequest::new(
                            traces.resource_spans.clone(),
                        ))
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }
        }

        // Shutdown the pipeline before reading responses
        shutdown_sender
            .send(PipelineControlMsg::Shutdown {
                deadline: std::time::Instant::now() + std::time::Duration::from_secs(5),
                reason: "Test completed".to_string(),
            })
            .await
            .map_err(|e| e.to_string())?;

        // Shutdown the gRPC server gracefully
        let _ = shutdown_tx.send(());

        // Give pipeline time to process and forward messages
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Read all messages from the receiver until empty
        let mut output_messages = Vec::new();
        while let Ok(Some(message_data)) =
            timeout(Duration::from_millis(100), receiver.recv()).await
        {
            let proto_message = match message_data {
                OTLPData::Logs(logs) => OtlpProtoMessage::Logs(logs.into()),
                OTLPData::Metrics(metrics) => OtlpProtoMessage::Metrics(metrics.into()),
                OTLPData::Traces(traces) => OtlpProtoMessage::Traces(traces.into()),
                OTLPData::Profiles(_) => {
                    return Err("Received unexpected Profiles message".to_string());
                }
            };
            output_messages.push(proto_message);
        }

        Ok(output_messages)
    }

    fn create_grpc_server(&self, sender: Sender<OTLPData>) -> Router {
        let logs_service = LogsServiceServer::new(LogsServiceChannel::new(sender.clone()));
        let metrics_service = MetricsServiceServer::new(MetricsServiceChannel::new(sender.clone()));
        let trace_service = TraceServiceServer::new(TraceServiceChannel::new(sender.clone()));
        Server::builder()
            .add_service(logs_service)
            .add_service(metrics_service)
            .add_service(trace_service)
    }
}

impl Default for OtelProtoSimulator {
    fn default() -> Self {
        Self {
            producer: Producer::new(),
            consumer: Consumer::default(),
        }
    }
}

pub struct LogsServiceChannel {
    sender: Sender<OTLPData>,
}

impl LogsServiceChannel {
    /// creates a new logs service
    #[must_use]
    pub fn new(sender: Sender<OTLPData>) -> Self {
        Self { sender }
    }
}

/// struct that implements the Metrics Service trait
pub struct MetricsServiceChannel {
    sender: Sender<OTLPData>,
}

impl MetricsServiceChannel {
    /// creates a new metrics service
    #[must_use]
    pub fn new(sender: Sender<OTLPData>) -> Self {
        Self { sender }
    }
}

/// struct that implements the Trace Service trait
pub struct TraceServiceChannel {
    sender: Sender<OTLPData>,
}

impl TraceServiceChannel {
    /// creates a new trace service
    #[must_use]
    pub fn new(sender: Sender<OTLPData>) -> Self {
        Self { sender }
    }
}

#[tonic::async_trait]
impl LogsService for LogsServiceChannel {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        self.sender
            .send(OTLPData::Logs(request.into_inner()))
            .await
            .expect("Logs failed to be sent through channel");
        Ok(Response::new(ExportLogsServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl MetricsService for MetricsServiceChannel {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        self.sender
            .send(OTLPData::Metrics(request.into_inner()))
            .await
            .expect("Metrics failed to be sent through channel");
        Ok(Response::new(ExportMetricsServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl TraceService for TraceServiceChannel {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        self.sender
            .send(OTLPData::Traces(request.into_inner()))
            .await
            .expect("Traces failed to be sent through channel");
        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use otap_df_otap::OTAP_PIPELINE_FACTORY;
    use otap_df_otap::fake_data_generator::fake_signal::{
        fake_otlp_logs, fake_otlp_metrics, fake_otlp_traces,
    };
    use otap_df_pdata::testing::equiv::assert_equivalent;
    use std::fs;
    use std::path::Path;
    use weaver_common::result::WResult;
    use weaver_common::vdir::VirtualDirectoryPath;
    use weaver_forge::registry::ResolvedRegistry;
    use weaver_resolver::SchemaResolver;
    use weaver_semconv::registry::SemConvRegistry;
    use weaver_semconv::registry_repo::RegistryRepo;

    const LOG_SIGNAL_COUNT: usize = 100;
    const METRIC_SIGNAL_COUNT: usize = 100;
    const TRACE_SIGNAL_COUNT: usize = 100;
    const ITERATIONS: usize = 10;
    const PIPELINE_CONFIG_DIRECTORY: &str = "./validation_pipelines";
    const MESSAGE_COUNT: usize = 10;

    fn get_registry() -> ResolvedRegistry {
        let registry_repo = RegistryRepo::try_new(
            "main",
            &VirtualDirectoryPath::GitRepo {
                url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
                sub_folder: Some("model".to_owned()),
                refspec: None,
            },
        )
        .expect("all registries are definied under the model folder in semantic convention repo");

        // Load the semantic convention specs
        let semconv_specs = match SchemaResolver::load_semconv_specs(&registry_repo, true, false) {
            WResult::Ok(semconv_specs) => semconv_specs,
            WResult::OkWithNFEs(semconv_specs, _) => semconv_specs,
            WResult::FatalErr(_err) => {
                panic!("Failed to load semantic convention specs");
            }
        };

        // Resolve the main registry
        let mut registry = SemConvRegistry::from_semconv_specs(&registry_repo, semconv_specs)
            .expect("Can resolve the registries defined in semantic convention repo");
        // Resolve the semantic convention specifications.
        // If there are any resolution errors, they should be captured into the ongoing list of
        // diagnostic messages and returned immediately because there is no point in continuing
        // as the resolution is a prerequisite for the next stages.
        let resolved_schema =
            match SchemaResolver::resolve_semantic_convention_registry(&mut registry, true) {
                WResult::Ok(resolved_schema) => resolved_schema,
                WResult::OkWithNFEs(resolved_schema, _) => resolved_schema,
                WResult::FatalErr(_err) => {
                    panic!("Failed to resolve semantic convetion schema");
                }
            };

        ResolvedRegistry::try_from_resolved_registry(
            &resolved_schema.registry,
            resolved_schema.catalog(),
        )
        .expect("can get resolved registry from official semantic convention repo")
    }

    // validate the encoding and decoding
    #[test]
    fn validate_encode_decode() {
        let mut otel_proto_simulator = OtelProtoSimulator::default();

        let registry = get_registry();

        for _ in 0..ITERATIONS {
            // generate data and simulate the protocol and compare result
            let logs = OtlpProtoMessage::Logs(fake_otlp_logs(LOG_SIGNAL_COUNT, &registry));
            let logs_output = otel_proto_simulator.simulate_proto(&logs);
            assert_equivalent(&[logs], &[logs_output]);

            let metrics =
                OtlpProtoMessage::Metrics(fake_otlp_metrics(METRIC_SIGNAL_COUNT, &registry));
            let metrics_output = otel_proto_simulator.simulate_proto(&metrics);
            assert_equivalent(&[metrics], &[metrics_output]);

            let traces = OtlpProtoMessage::Traces(fake_otlp_traces(TRACE_SIGNAL_COUNT, &registry));
            let traces_output = otel_proto_simulator.simulate_proto(&traces);
            assert_equivalent(&[traces], &[traces_output]);
        }
    }

    // validate the encoding and decoding
    #[tokio::test]
    async fn validate_pipeline() {
        let pipeline_simulator = PipelineSimulator::new(&OTAP_PIPELINE_FACTORY);

        let registry = get_registry();

        // read the validate_pipelines directory
        // read only md files
        let pipeline_config_files =
            fs::read_dir(Path::new(PIPELINE_CONFIG_DIRECTORY)).expect("Directory exists");
        for config_file in pipeline_config_files {
            let file = config_file.expect("ok file to read");
            let file_path = file.path();
            if file_path.is_file() {
                println!("Validating Pipeline: {}", file_path.display());

                // generate data and simulate the protocol and compare result
                let logs: Vec<OtlpProtoMessage> = (0..MESSAGE_COUNT)
                    .map(|_| OtlpProtoMessage::Logs(fake_otlp_logs(LOG_SIGNAL_COUNT, &registry)))
                    .collect();
                let logs_output = pipeline_simulator
                    .simulate_pipeline(&logs, file_path.clone())
                    .await
                    .expect("failed to simulate pipeline");
                assert_equivalent(&logs, &logs_output);

                let metrics: Vec<OtlpProtoMessage> = (0..MESSAGE_COUNT)
                    .map(|_| {
                        OtlpProtoMessage::Metrics(fake_otlp_metrics(METRIC_SIGNAL_COUNT, &registry))
                    })
                    .collect();
                let metrics_output = pipeline_simulator
                    .simulate_pipeline(&metrics, file_path.clone())
                    .await
                    .expect("failed to simulate pipeline");
                assert_equivalent(&metrics, &metrics_output);

                let traces: Vec<OtlpProtoMessage> = (0..MESSAGE_COUNT)
                    .map(|_| {
                        OtlpProtoMessage::Traces(fake_otlp_traces(TRACE_SIGNAL_COUNT, &registry))
                    })
                    .collect();
                let traces_output = pipeline_simulator
                    .simulate_pipeline(&traces, file_path.clone())
                    .await
                    .expect("failed to simulate pipeline");
                assert_equivalent(&traces, &traces_output);
            }
        }
    }
}
