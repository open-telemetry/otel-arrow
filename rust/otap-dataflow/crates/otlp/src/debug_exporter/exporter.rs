// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP exporter node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuratin changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg

use crate::LOCAL_EXPORTERS;
use crate::debug_exporter::marshaler::{NormalOTLPMarshaler, PDataMarshaler};
use crate::debug_exporter::verbosity::Verbosity;
use crate::grpc::OTLPData;
use crate::proto::opentelemetry::{
    collector::{
        logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
        profiles::v1development::ExportProfilesServiceRequest,
        trace::v1::ExportTraceServiceRequest,
    },
    metrics::v1::metric::Data,
};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_engine::error::Error;
use otap_df_engine::local::{LocalExporterFactory, exporter as local};
use otap_df_engine::message::{ControlMsg, Message, MessageChannel};
use serde_json::Value;

/// Exporter that outputs all data received to stdout
struct DebugExporter {
    verbosity: Verbosity, // writer: Box<dyn std::io::Write + Sync>,
}

/// Declares the Debug exporter as a local exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(LOCAL_EXPORTERS)]
pub static DEBUG_EXPORTER: LocalExporterFactory<OTLPData> = LocalExporterFactory {
    name: "urn:otel:debug:exporter",
    create: |config: &Value| Box::new(DebugExporter::from_config(config)),
};

impl DebugExporter {
    /// Creates a new Debug exporter
    #[must_use]
    #[allow(dead_code)]
    pub fn new(verbosity: Verbosity) -> Self {
        DebugExporter { verbosity }
    }

    /// Creates a new DebugExporter from a configuration object
    #[must_use]
    pub fn from_config(_config: &Value) -> Self {
        // ToDo: implement config parsing
        DebugExporter {
            verbosity: Verbosity::Detailed,
        }
    }
}

/// Implement the local exporter trait for a OTAP Exporter
#[async_trait(?Send)]
impl local::Exporter<OTLPData> for DebugExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OTLPData>,
        effect_handler: local::EffectHandler<OTLPData>,
    ) -> Result<(), Error<OTLPData>> {
        let mut metric_count: u64 = 0;
        let mut profile_count: u64 = 0;
        let mut span_count: u64 = 0;
        let mut log_count: u64 = 0;
        let mut marshaler = NormalOTLPMarshaler::default();
        // if verbosity == Verbosity::Normal {
        //     marshaler = NormalOTLPMarshaler;
        // } else if verbosity == Verbosity::Detailed {
        //     marshaler = DetailedOTLPMarshaler;
        // }

        // Loop until a Shutdown event is received.
        loop {
            match msg_chan.recv().await? {
                // handle control messages
                Message::Control(ControlMsg::TimerTick { .. }) => {
                    println!("Timer tick received");
                    // print count of messages received
                    println!("Count of metrics received {}", metric_count);
                    println!("Count of spans received {}", span_count);
                    println!("Count of profiles received {}", profile_count);
                    println!("Count of logs received {}", log_count);

                    metric_count = 0;
                    span_count = 0;
                    log_count = 0;
                    profile_count = 0;
                }
                Message::Control(ControlMsg::Config { .. }) => {
                    println!("Config message received");
                }
                // shutdown the exporter
                Message::Control(ControlMsg::Shutdown { .. }) => {
                    // ToDo: add proper deadline function
                    println!("Shutdown message received");
                    break;
                }
                //send data
                Message::PData(message) => {
                    match message {
                        // match on OTLPData type and use the respective client to send message
                        // ToDo: Add Ack/Nack handling, send a signal that data has been exported
                        // check what message
                        OTLPData::Metrics(req) => {
                            push_metric(&self.verbosity, req, &marshaler);
                            metric_count += 1;
                        }
                        OTLPData::Logs(req) => {
                            push_log(&self.verbosity, req, &marshaler);
                            log_count += 1;
                        }
                        OTLPData::Traces(req) => {
                            push_trace(&self.verbosity, req, &marshaler);
                            span_count += 1;
                        }
                        OTLPData::Profiles(req) => {
                            push_profile(&self.verbosity, req, &marshaler);
                            profile_count += 1;
                        }
                    }
                }
                _ => {
                    return Err(Error::ExporterError {
                        exporter: effect_handler.exporter_name(),
                        error: "Unknown control message".to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

fn push_metric(
    verbosity: &Verbosity,
    metric_request: ExportMetricsServiceRequest,
    marshaler: &dyn PDataMarshaler,
) {
    // collect number of resource metrics
    // collect number of metrics
    // collect number of datapoints
    let resouce_metrics = metric_request.resource_metrics.len();
    let mut data_points = 0;
    let mut metrics = 0;
    for resource_metrics in &metric_request.resource_metrics {
        for scope_metrics in &resource_metrics.scope_metrics {
            metrics += scope_metrics.metrics.len();
            for metric in &scope_metrics.metrics {
                if let Some(data) = &metric.data {
                    match data {
                        Data::Gauge(gauge) => {
                            data_points += gauge.data_points.len();
                        }
                        Data::Sum(sum) => {
                            data_points += sum.data_points.len();
                        }
                        Data::Histogram(histogram) => {
                            data_points += histogram.data_points.len();
                        }
                        Data::ExponentialHistogram(exponential_histogram) => {
                            data_points += exponential_histogram.data_points.len();
                        }
                        Data::Summary(summary) => {
                            data_points += summary.data_points.len();
                        }
                    }
                }
            }
        }
    }

    println!("Received {} resource metrics", resouce_metrics);
    println!("Received {} metrics", metrics);
    println!("Received {} data points", data_points);

    if *verbosity == Verbosity::Basic {
        return;
    }

    let report = marshaler.marshal_metrics(metric_request);
    println!("{}", report);
    return;
}

fn push_trace(
    verbosity: &Verbosity,
    trace_request: ExportTraceServiceRequest,
    marshaler: &dyn PDataMarshaler,
) {
    // collect number of resource spans
    // collect number of spans
    let resource_spans = trace_request.resource_spans.len();
    let mut spans = 0;
    for resource_span in &trace_request.resource_spans {
        for scope_span in &resource_span.scope_spans {
            spans += scope_span.spans.len();
        }
    }
    println!("Received {} resource spans", resource_spans);
    println!("Received {} spans", spans);
    if *verbosity == Verbosity::Basic {
        return;
    }

    let report = marshaler.marshal_traces(trace_request);
    println!("{}", report);

    return;
}

fn push_log(
    verbosity: &Verbosity,
    log_request: ExportLogsServiceRequest,
    marshaler: &dyn PDataMarshaler,
) {
    let resource_logs = log_request.resource_logs.len();
    let mut log_records = 0;
    for resource_log in &log_request.resource_logs {
        for scope_log in &resource_log.scope_logs {
            log_records += scope_log.log_records.len();
        }
    }
    println!("Received {} resource logs", resource_logs);
    println!("Received {} log records", log_records);

    if *verbosity == Verbosity::Basic {
        return;
    }

    let report = marshaler.marshal_logs(log_request);
    println!("{}", report);

    return;
}

fn push_profile(
    verbosity: &Verbosity,
    profile_request: ExportProfilesServiceRequest,
    marshaler: &dyn PDataMarshaler,
) {
    // collect number of resource profiles
    // collect number of sample records
    let resource_profiles = profile_request.resource_profiles.len();
    let mut samples = 0;
    for resource_profile in &profile_request.resource_profiles {
        for scope_profile in &resource_profile.scope_profiles {
            for profile in &scope_profile.profiles {
                samples += profile.sample.len();
            }
        }
    }

    println!("Received {} resource profiles", resource_profiles);
    println!("Received {} samples", samples);

    if *verbosity == Verbosity::Basic {
        return;
    }

    let report = marshaler.marshal_profiles(profile_request);
    println!("{}", report);

    return;
}

// #[cfg(test)]
// mod tests {

//     use crate::grpc::OTLPData;
//     use crate::mock::{
//         ArrowLogsServiceMock, ArrowMetricsServiceMock, ArrowTracesServiceMock,
//         create_batch_arrow_record,
//     };
//     use crate::otap_exporter::OTAPExporter;
//     use crate::proto::opentelemetry::experimental::arrow::v1::{
//         ArrowPayloadType, arrow_logs_service_server::ArrowLogsServiceServer,
//         arrow_metrics_service_server::ArrowMetricsServiceServer,
//         arrow_traces_service_server::ArrowTracesServiceServer,
//     };
//     use otap_df_engine::exporter::ExporterWrapper;
//     use otap_df_engine::testing::exporter::TestContext;
//     use otap_df_engine::testing::exporter::TestRuntime;
//     use std::net::SocketAddr;
//     use tokio::net::TcpListener;
//     use tokio::runtime::Runtime;
//     use tokio::time::{Duration, timeout};
//     use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
//     use tonic::transport::Server;

//     const METRIC_BATCH_ID: i64 = 0;
//     const LOG_BATCH_ID: i64 = 1;
//     const TRACE_BATCH_ID: i64 = 2;

//     /// Test closure that simulates a typical test scenario by sending timer ticks, config,
//     /// data message, and shutdown control messages.
//     fn scenario()
//     -> impl FnOnce(TestContext<OTLPData>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
//         |ctx| {
//             Box::pin(async move {
//                 // Send a data message
//                 let metric_message = OTLPData::ArrowMetrics(create_batch_arrow_record(
//                     METRIC_BATCH_ID,
//                     ArrowPayloadType::MultivariateMetrics,
//                 ));
//                 ctx.send_pdata(metric_message)
//                     .await
//                     .expect("Failed to send metric message");

//                 let log_message = OTLPData::ArrowLogs(create_batch_arrow_record(
//                     LOG_BATCH_ID,
//                     ArrowPayloadType::Logs,
//                 ));
//                 ctx.send_pdata(log_message)
//                     .await
//                     .expect("Failed to send log message");

//                 let trace_message = OTLPData::ArrowTraces(create_batch_arrow_record(
//                     TRACE_BATCH_ID,
//                     ArrowPayloadType::Spans,
//                 ));
//                 ctx.send_pdata(trace_message)
//                     .await
//                     .expect("Failed to send trace message");

//                 // Send shutdown
//                 ctx.send_shutdown(Duration::from_millis(200), "test complete")
//                     .await
//                     .expect("Failed to send Shutdown");
//             })
//         }
//     }

//     /// Validation closure that checks the expected counter values
//     fn validation_procedure(
//         mut receiver: tokio::sync::mpsc::Receiver<OTLPData>,
//     ) -> impl FnOnce(TestContext<OTLPData>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
//         |_| {
//             Box::pin(async move {
//                 // check that the message was properly sent from the exporter
//                 let metrics_received = timeout(Duration::from_secs(3), receiver.recv())
//                     .await
//                     .expect("Timed out waiting for message")
//                     .expect("No message received");

//                 // Assert that the message received is what the exporter sent
//                 let _expected_metrics_message = create_batch_arrow_record(
//                     METRIC_BATCH_ID,
//                     ArrowPayloadType::MultivariateMetrics,
//                 );
//                 assert!(matches!(metrics_received, _expected_metrics_message));

//                 let logs_received = timeout(Duration::from_secs(3), receiver.recv())
//                     .await
//                     .expect("Timed out waiting for message")
//                     .expect("No message received");
//                 let _expected_logs_message =
//                     create_batch_arrow_record(LOG_BATCH_ID, ArrowPayloadType::Logs);
//                 assert!(matches!(logs_received, _expected_logs_message));

//                 let traces_received = timeout(Duration::from_secs(3), receiver.recv())
//                     .await
//                     .expect("Timed out waiting for message")
//                     .expect("No message received");

//                 let _expected_trace_message =
//                     create_batch_arrow_record(TRACE_BATCH_ID, ArrowPayloadType::Spans);
//                 assert!(matches!(traces_received, _expected_trace_message));
//             })
//         }
//     }

//     #[test]
//     fn test_otap_exporter() {
//         let test_runtime = TestRuntime::new();
//         let (sender, receiver) = tokio::sync::mpsc::channel(32);
//         let (shutdown_sender, shutdown_signal) = tokio::sync::oneshot::channel();
//         let grpc_addr = "127.0.0.1";
//         let grpc_port = portpicker::pick_unused_port().expect("No free ports");
//         let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
//         let listening_addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
//         // tokio runtime to run grpc server in the background
//         let tokio_rt = Runtime::new().unwrap();

//         // run a gRPC concurrently to receive data from the exporter
//         _ = tokio_rt.spawn(async move {
//             let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
//             let tcp_stream = TcpListenerStream::new(tcp_listener);
//             let mock_logs_service =
//                 ArrowLogsServiceServer::new(ArrowLogsServiceMock::new(sender.clone()));
//             let mock_metrics_service =
//                 ArrowMetricsServiceServer::new(ArrowMetricsServiceMock::new(sender.clone()));
//             let mock_trace_service =
//                 ArrowTracesServiceServer::new(ArrowTracesServiceMock::new(sender.clone()));
//             Server::builder()
//                 .add_service(mock_logs_service)
//                 .add_service(mock_metrics_service)
//                 .add_service(mock_trace_service)
//                 .serve_with_incoming_shutdown(tcp_stream, async {
//                     // Wait for the shutdown signal
//                     let _ = shutdown_signal.await;
//                 })
//                 .await
//                 .expect("Test gRPC server has failed");
//         });

//         let exporter = ExporterWrapper::local(
//             OTAPExporter::new(grpc_endpoint, None),
//             test_runtime.config(),
//         );

//         test_runtime
//             .set_exporter(exporter)
//             .run_test(scenario())
//             .run_validation(validation_procedure(receiver));

//         _ = shutdown_sender.send("Shutdown");
//     }
// }
