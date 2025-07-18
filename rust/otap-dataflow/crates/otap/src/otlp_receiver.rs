// SPDX-License-Identifier: Apache-2.0
// Copyright The OpenTelemetry Authors

use std::net::SocketAddr;

use async_trait::async_trait;
use otap_df_engine::error::Error;
use otap_df_engine::message::ControlMsg;
use otap_df_engine::shared::receiver as shared;
use otap_df_otlp::compression::CompressionMethod;
use otel_arrow_rust::otap::OtapBatch;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

use crate::grpc::otlp::{LogsServiceServer, MetricsServiceServer, TraceServiceServer};

/// Receiver implementation that receives OTLP grpc service requests and decodes the data into OTAP.
pub struct OTLPReceiver {
    listening_addr: SocketAddr,
    compression_method: Option<CompressionMethod>,
}

#[async_trait]
impl shared::Receiver<OtapBatch> for OTLPReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: shared::ControlChannel,
        effect_handler: shared::EffectHandler<OtapBatch>,
    ) -> Result<(), Error<OtapBatch>> {
        let listener = effect_handler.tcp_listener(self.listening_addr)?;
        let mut listener_stream = TcpListenerStream::new(listener);

        loop {
            let mut logs_service_server = LogsServiceServer::new(effect_handler.clone());
            let mut metrics_service_server = MetricsServiceServer::new(effect_handler.clone());
            let mut trace_service_server = TraceServiceServer::new(effect_handler.clone());

            if let Some(ref compression) = self.compression_method {
                let encoding = compression.map_to_compression_encoding();

                logs_service_server = logs_service_server
                    .send_compressed(encoding)
                    .accept_compressed(encoding);
                metrics_service_server = metrics_service_server
                    .send_compressed(encoding)
                    .accept_compressed(encoding);
                trace_service_server = trace_service_server
                    .send_compressed(encoding)
                    .accept_compressed(encoding);
            }

            tokio::select! {
                biased;

                // Process internal event
                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(ControlMsg::Shutdown {..}) => {
                            break;
                        },
                        Err(e) => {
                            return Err(Error::ChannelRecvError(e));
                        }
                        _ => {
                            // unknown control message do nothing
                        }
                    }
                }
                result = Server::builder()
                    .add_service(logs_service_server)
                    .add_service(metrics_service_server)
                    .add_service(trace_service_server)
                    .serve_with_incoming(&mut listener_stream) => {
                        if let Err(error) = result {
                            return Err(Error::ReceiverError {
                                receiver: effect_handler.receiver_name(),
                                error: error.to_string()
                            })
                        }
                    }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::pin::Pin;
    use std::sync::Arc;
    use std::time::Duration;

    use arrow::array::{DictionaryArray, StringArray, TimestampNanosecondArray, UInt8Array};
    use arrow::datatypes::UInt8Type;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::receiver::{NotSendValidateContext, TestContext, TestRuntime};
    use otap_df_otlp::proto::opentelemetry::collector::logs::v1::ExportLogsServiceResponse;
    use otap_df_otlp::proto::opentelemetry::collector::logs::v1::{
        ExportLogsServiceRequest, logs_service_client::LogsServiceClient,
    };
    use otap_df_otlp::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otap_df_otlp::proto::opentelemetry::collector::metrics::v1::metrics_service_client::MetricsServiceClient;
    use otap_df_otlp::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    use otap_df_otlp::proto::opentelemetry::collector::trace::v1::trace_service_client::TraceServiceClient;
    use otap_df_otlp::proto::opentelemetry::common::v1::{InstrumentationScope, KeyValue};
    use otap_df_otlp::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use otap_df_otlp::proto::opentelemetry::resource::v1::Resource;
    use otel_arrow_rust::otap::OtapBatch;
    use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otel_arrow_rust::schema::consts;
    use tokio::time::timeout;
    use tonic::Code;

    fn scenario(
        grpc_endpoint: String,
    ) -> impl FnOnce(TestContext) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                let mut logs_client = LogsServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server from Logs Service Client");

                let logs_response = logs_client
                    .export(ExportLogsServiceRequest {
                        resource_logs: vec![ResourceLogs {
                            resource: Some(Resource {
                                attributes: vec![KeyValue {
                                    key: "a".to_string(),
                                    ..Default::default()
                                }],
                                ..Default::default()
                            }),
                            scope_logs: vec![ScopeLogs {
                                scope: Some(InstrumentationScope {
                                    attributes: vec![KeyValue {
                                        key: "b".to_string(),
                                        ..Default::default()
                                    }],
                                    ..Default::default()
                                }),
                                log_records: vec![
                                    LogRecord {
                                        time_unix_nano: 1,
                                        attributes: vec![KeyValue {
                                            key: "c".to_string(),
                                            ..Default::default()
                                        }],
                                        ..Default::default()
                                    },
                                    LogRecord {
                                        time_unix_nano: 2,
                                        ..Default::default()
                                    },
                                ],
                                ..Default::default()
                            }],
                            ..Default::default()
                        }],
                    })
                    .await
                    .expect("Can send log request")
                    .into_inner();
                assert_eq!(
                    logs_response,
                    ExportLogsServiceResponse {
                        partial_success: None
                    }
                );

                // TODO -- when we support decoding OTAP from proto bytes for metrics & traces, send real data
                // in these tests, assert the request is successful
                let mut metrics_client = MetricsServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server from Metrics Service Client");
                let metrics_response = metrics_client
                    .export(ExportMetricsServiceRequest::default())
                    .await;
                let err = metrics_response.unwrap_err();
                assert_eq!(err.code(), Code::Unimplemented);
                assert_eq!(err.message(), "signal type not yet implemented");

                let mut traces_client = TraceServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server from Metrics Service Client");
                let traces_response = traces_client
                    .export(ExportTraceServiceRequest::default())
                    .await;
                let err = traces_response.unwrap_err();
                assert_eq!(err.code(), Code::Unimplemented);
                assert_eq!(err.message(), "signal type not yet implemented");

                ctx.send_shutdown(Duration::from_millis(0), "Test")
                    .await
                    .expect("Failed to send Shutdown");

                let fail_client = LogsServiceClient::connect(grpc_endpoint.clone()).await;
                assert!(fail_client.is_err(), "Server did not shutdown");
            })
        }
    }

    fn validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OtapBatch>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                let logs_otap_batch = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

                assert!(matches!(logs_otap_batch, OtapBatch::Logs(_)));

                // make some basic assertions about the batch
                let logs = logs_otap_batch
                    .get(ArrowPayloadType::Logs)
                    .expect("No logs found in otap batch");
                assert_eq!(2, logs.num_rows());
                let timestamp_column = logs
                    .column_by_name(consts::TIME_UNIX_NANO)
                    .unwrap()
                    .as_any()
                    .downcast_ref()
                    .unwrap();
                let expected_timestamps = TimestampNanosecondArray::from_iter_values(vec![1, 2]);
                assert_eq!(&expected_timestamps, timestamp_column);

                let resource_attrs = logs_otap_batch
                    .get(ArrowPayloadType::ResourceAttrs)
                    .expect("No resource attributes found in otap batch");
                assert_eq!(1, resource_attrs.num_rows());
                let key_column: &DictionaryArray<UInt8Type> = resource_attrs
                    .column_by_name(consts::ATTRIBUTE_KEY)
                    .unwrap()
                    .as_any()
                    .downcast_ref()
                    .unwrap();
                let expected_keys = DictionaryArray::new(
                    UInt8Array::from_iter_values(vec![0]),
                    Arc::new(StringArray::from_iter_values(vec!["a"])),
                );
                assert_eq!(&expected_keys, key_column);

                let scope_attrs = logs_otap_batch
                    .get(ArrowPayloadType::ScopeAttrs)
                    .expect("No resource attributes found in otap batch");
                assert_eq!(1, resource_attrs.num_rows());
                let key_column: &DictionaryArray<UInt8Type> = scope_attrs
                    .column_by_name(consts::ATTRIBUTE_KEY)
                    .unwrap()
                    .as_any()
                    .downcast_ref()
                    .unwrap();
                let expected_keys = DictionaryArray::new(
                    UInt8Array::from_iter_values(vec![0]),
                    Arc::new(StringArray::from_iter_values(vec!["b"])),
                );
                assert_eq!(&expected_keys, key_column);

                let log_attrs = logs_otap_batch
                    .get(ArrowPayloadType::LogAttrs)
                    .expect("No log attributes found in otap batch");
                assert_eq!(1, log_attrs.num_rows());
                let key_column: &DictionaryArray<UInt8Type> = log_attrs
                    .column_by_name(consts::ATTRIBUTE_KEY)
                    .unwrap()
                    .as_any()
                    .downcast_ref()
                    .unwrap();
                let expected_keys = DictionaryArray::new(
                    UInt8Array::from_iter_values(vec![0]),
                    Arc::new(StringArray::from_iter_values(vec!["c"])),
                );
                assert_eq!(&expected_keys, key_column);
            })
        }
    }

    #[test]
    fn test_otlp_receiver() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                listening_addr: addr,
                compression_method: None,
            },
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario(grpc_endpoint))
            .run_validation(validation_procedure());
    }
}
