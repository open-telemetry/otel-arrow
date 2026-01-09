// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validation test module to validate the encoding/decoding process for otlp messages

use otap_df_config::PipelineGroupId;
use otap_df_config::PipelineId;
use otap_df_config::pipeline::PipelineConfig;
use otap_df_engine::PipelineFactory;
use otap_df_engine::context::ControllerContext;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::pipeline_ctrl_msg_channel;
use otap_df_engine::runtime_pipeline::RuntimePipeline;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
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
use tokio::sync::mpsc::Sender;
use tonic::transport::server::{Router, Server};
use tonic::{Request, Response, Status};

const GRPC_INPUT_ENDPOINT: &str = "http://127.0.0.1:4317";
const GRPC_OUTPUT_ENDPOINT: &str = "127.0.0.1:4318";
const PIPELINE_CONFIG_DIRECTORY: &str = "../validation_pipelines";
const DEFAULT_CORE_ID: usize = 0;
const DEFAULT_THREAD_ID: usize = 0;

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
    controller_context: ControllerContext,
    core_id: usize,
    thread_id: usize,
    metrics_system: MetricsSystem,
    pipeline_key: DeployedPipelineKey,
}

impl<PData: 'static + Clone + Send + Sync + std::fmt::Debug> PipelineSimulator<PData> {
    // if pipeline alters the data via a processor that performs some transofmration we should expect the equivalent assert to fail
    // otherwise the assert should succeed
    pub async fn simulate_pipeline(
        &self,
        proto_message: OtlpProtoMessage,
        config_file_path: PathBuf,
    ) -> OtlpProtoMessage {
        // start a grpc client to send messages to the receiver
        // start a grpc server to receiver messages from the exporter
        let (sender, mut receiver) = tokio::sync::mpsc::channel(64);
        let grpc_server = self.create_grpc_server(sender);

        // start gRPC server in the Tokio runtime
        let addr: std::net::SocketAddr =
            GRPC_OUTPUT_ENDPOINT.parse().expect("valid socket address");
        let _grpc_server_task = tokio::spawn(async move {
            grpc_server
                .serve(addr)
                .await
                .expect("failed to serve gRPC receiver");
        });

        let pipeline_factory = self.pipeline_factory.clone();
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
            .expect("correct file path");
            let obs_state_store = ObservedStateStore::new(pipeline_config.pipeline_settings());
            let obs_evt_reporter = obs_state_store.reporter();
            let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) = pipeline_ctrl_msg_channel(
                pipeline_config
                    .pipeline_settings()
                    .default_pipeline_ctrl_msg_channel_size,
            );
            let pipeline_runtime = pipeline_factory
                .build(pipeline_context.clone(), pipeline_config.clone())
                .expect("pipeline created");
            pipeline_runtime.run_forever(
                pipeline_key,
                pipeline_context,
                obs_evt_reporter,
                metrics_reporter,
                pipeline_ctrl_msg_tx,
                pipeline_ctrl_msg_rx,
            )
        });

        // start sending messages to the pipeline
        let mut logs_client = LogsServiceClient::connect(GRPC_INPUT_ENDPOINT)
            .await
            .expect("failed to connect to otlp receiver");
        let mut traces_client = TraceServiceClient::connect(GRPC_INPUT_ENDPOINT)
            .await
            .expect("failed to connect to otlp receiver");
        let mut metrics_client = MetricsServiceClient::connect(GRPC_INPUT_ENDPOINT)
            .await
            .expect("failed to connect to otlp receiver");

        match proto_message {
            OtlpProtoMessage::Logs(logs) => {
                logs_client
                    .export(ExportLogsServiceRequest::new(logs.resource_logs))
                    .await
                    .expect("failed to send message to otlp receiver");
            }
            OtlpProtoMessage::Metrics(metrics) => {
                metrics_client
                    .export(ExportMetricsServiceRequest::new(metrics.resource_metrics))
                    .await
                    .expect("failed to send message to otlp receiver");
            }
            OtlpProtoMessage::Traces(traces) => {
                traces_client
                    .export(ExportTraceServiceRequest::new(traces.resource_spans))
                    .await
                    .expect("failed to send message to otlp receiver");
            }
        }

        // read message from receiver (output from pipeline)
        let message: OTLPData = receiver
            .recv()
            .await
            .expect("Failed to receive message")
            .into();

        // convert OTLPData to OTLPProtoMessage and return
        match message {
            OTLPData::Logs(logs) => OtlpProtoMessage::Logs(logs.into()),
            OTLPData::Metrics(metrics) => OtlpProtoMessage::Metrics(metrics.into()),
            OTLPData::Traces(traces) => OtlpProtoMessage::Traces(traces.into()),
            OTLPData::Profiles(_) => {
                // error here as we should not be receiving a profiles type
                panic!("did not send a profile type");
            }
        }
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

impl<PData: 'static + Clone + Send + Sync + std::fmt::Debug> Default for PipelineSimulator<PData> {
    fn default() -> Self {
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
            pipeline_factory: &OTAP_PIPELINE_FACTORY,
            pipeline_context,
            pipeline_id,
            pipeline_group_id,
            core_id,
            thread_id,
            controller_context,
            metrics_system,
            pipeline_key,
        }
    }
}

pub struct LogsServiceChannel {
    sender: Sender<OTLPData>,
}

impl LogsServiceChannel {
    /// creates a new mock logs service
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
    /// creates a new mock metrics service
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
    /// creates a new mock trace service
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
    use crate::fake_data_generator::fake_signal::{
        fake_otlp_logs, fake_otlp_metrics, fake_otlp_traces,
    };
    use otap_df_pdata::testing::equiv::assert_equivalent;

    const LOG_SIGNAL_COUNT: usize = 100;
    const METRIC_SIGNAL_COUNT: usize = 100;
    const TRACE_SIGNAL_COUNT: usize = 100;
    const ITERATIONS: usize = 10;

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

    async fn validate_pipeline() {
        let mut pipeline_simulator = PipelineSimulator::default();

        let registry = get_registry();

        // read the validate_pipelines directory
        // read only md files
        let pipeline_config_files = fs::read_dir(Path::new(PIPELINE_CONFIG_DIRECTORY)).expect("");
        for config_file in pipeline_config_files {
            let file = config_file.expect("ok file to read");
            let file_path = file.path();
            if file_path.is_file() {
                for _ in 0..ITERATIONS {
                    // generate data and simulate the protocol and compare result
                    let logs = OtlpProtoMessage::Logs(fake_otlp_logs(LOG_SIGNAL_COUNT, &registry));
                    let logs_output = pipeline_simulator
                        .simulate_pipeline(&logs, file_path.clone())
                        .await;
                    assert_equivalent(&[logs], &[logs_output]);

                    let metrics = OtlpProtoMessage::Metrics(fake_otlp_metrics(
                        METRIC_SIGNAL_COUNT,
                        &registry,
                    ));
                    let metrics_output = pipeline_simulator
                        .simulate_pipeline(&metrics, file_path.clone())
                        .await;
                    assert_equivalent(&[metrics], &[metrics_output]);

                    let traces =
                        OtlpProtoMessage::Traces(fake_otlp_traces(TRACE_SIGNAL_COUNT, &registry));
                    let traces_output = pipeline_simulator
                        .simulate_pipeline(&traces, file_path.clone())
                        .await;
                    assert_equivalent(&[traces], &[traces_output]);
                }
            }
        }
    }
}
