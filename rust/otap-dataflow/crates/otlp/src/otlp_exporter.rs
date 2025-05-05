use crate::grpc::{LogsServiceImpl, MetricsServiceImpl, TraceServiceImpl, OTLPRequest};
use crate::grpc::grpc_stubs::proto::collector::{logs::v1::logs_service_client::LogsServiceClient,
    metrics::v1::metrics_service_client::MetricsServiceClient,
    trace::v1::trace_service_client::TraceServiceClient};
use otap_df_engine::exporter::{EffectHandlerTrait, Exporter, MessageChannel, SendEffectHandler};
use otap_df_engine::error::Error;
use otap_df_engine::message::{ControlMsg, Message};
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::time::{Duration, sleep};
use tonic::codec::CompressionEncoding;

struct OTLPExporter<'a> {
    grpc_addr: &'a str,
    grpc_port: u16
}

impl OTLPExporter {
    /// Creates a new test exporter with the given counter.
    pub fn new(grpc_addr: &str, grpc_port: u16) -> Self {
        OTLPExporter { grpc_addr: grpc_addr, grpc_port: grpc_port }
    }
}
#[async_trait(?Send)]
impl Exporter<OTLPRequest, SendEffectHandler<OTLPRequest>> for OTLPExporter {

    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OTLPRequest>,
        effect_handler: SendEffectHandler<OTLPRequest>,
    ) -> Result<(), Error<OTLPRequest>> {
        // Loop until a Shutdown event is received.
        // start a grpc client and send data from it to the provided addr

        // sent signals should we send ack and nack control messages??/
        let grpc_endpoint = format!("https://{self.grpc_addr}:{self.grpc_port}");
        let mut metrics_client = MetricsServiceClient::connect(grpc_endpoint.clone()).await;
        let mut logs_client = LogsServiceClient::connect(grpc_endpoint.clone()).await;
        let mut traces_client = TraceServiceClient::connect(grpc_endpoint.clone()).await;
   
        loop {
            match msg_chan.recv().await? {
                Message::Control(ControlMsg::TimerTick { .. }) |
                Message::Control(ControlMsg::Config { .. }) => {
                }
                Message::Control(ControlMsg::Shutdown { .. }) => {
                    break;
                }
                Message::PData(message) => {
                    match message {
                        OTLPRequest::Metrics(req) => {
                            let metrics_response = metrics_client.export(req).await;
                            //check response and return error 
                        }
                        OTLPRequest::Logs(req) => {
                            let logs_response = logs_client.export(req).await;
                            //check response and return error 
                        }
                        OTLPRequest::Traces(req) => {
                            let trace_response = traces_client.export(req).await;
                            //check response and return error 
                        }
                    }
                }
                _ => {
                    return Err(Error::ExporterError {
                        exporter: effect_handler.exporter_name().to_string(),
                        error: "Unknown control message".to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}




// #[cfg(test)]
// mod tests {
//     use otap_df_engine::exporter::{
//         EffectHandlerTrait, Error, Exporter, ExporterWrapper, MessageChannel,
//         SendEffectHandler,
//     };
//     use otap_df_engine::message::{ControlMsg, Message};
//     use otap_df_engine::testing::exporter::TestContext;
//     use otap_df_engine::testing::exporter::TestRuntime;
//     use otap_df_engine::testing::{CtrlMsgCounters, TestMsg, exec_in_send_env};
//     use async_trait::async_trait;
//     use otap_df_channel::error::RecvError;
//     use otap_df_channel::mpsc;
//     use serde_json::Value;
//     use std::future::Future;
//     use std::time::Duration;
//     use tokio::time::sleep;
//     use crate::otlp_exporter::OTLPExporter;

//     /// Test closure that simulates a typical test scenario by sending timer ticks, config,
//     /// data message, and shutdown control messages.
//     fn scenario() -> impl FnOnce(TestContext<TestMsg>) -> std::pin::Pin<Box<dyn Future<Output = ()>>>
//     {
//         |ctx| {
//             Box::pin(async move {
//                 // Send 3 TimerTick events.
//                 for _ in 0..3 {
//                     ctx.send_timer_tick()
//                         .await
//                         .expect("Failed to send TimerTick");
//                     ctx.sleep(Duration::from_millis(50)).await;
//                 }

//                 // Send a Config event.
//                 ctx.send_config(Value::Null)
//                     .await
//                     .expect("Failed to send Config");

//                 // Send a data message
//                 let metric_message = OTLPRequest::Metrics()
//                 let metric_server = 
//                 ctx.send_pdata(metric_message)
//                     .await
//                     .expect("Failed to send metric message");
//                 let log_message = OTLPRequest::Logs()
//                 ctx.send_pdata(log_message)
//                     .await
//                     .expect("Failed to send log message");

//                 let trace_message = OTLPRequest::Traces()
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
//     fn validation_procedure()
//     -> impl FnOnce(TestContext<TestMsg>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
//         |ctx| {
//             Box::pin(async move {
//                 ctx.counters().assert(
//                     3, // timer tick
//                     3, // message
//                     1, // config
//                     1, // shutdown
//                 );
//             })
//         }
//     }

//     #[test]
//     fn test_exporter_with_send_effect_handler() {
//         let test_runtime = TestRuntime::new();

//         let grpc_addr = "127.0.0.1";
//         let grpc_port = "4317";
//         let exporter = ExporterWrapper::with_send(
//             OTLPExporter(grpc_addr: grpc_addr, grpc_port: grpc_port),
//             test_runtime.config(),
//         );

//         test_runtime
//             .set_exporter(exporter)
//             .run_test(scenario())
//             .run_validation(validation_procedure());
//     }

//     fn make_chan() -> (
//         mpsc::Sender<ControlMsg>,
//         mpsc::Sender<String>,
//         MessageChannel<String>,
//     ) {
//         let (control_tx, control_rx) = mpsc::Channel::<ControlMsg>::new(10);
//         let (pdata_tx, pdata_rx) = mpsc::Channel::<String>::new(10);
//         (
//             control_tx,
//             pdata_tx,
//             MessageChannel::new(control_rx, pdata_rx),
//         )
//     }

//     #[tokio::test]
//     async fn test_control_priority() {
//         let (control_tx, pdata_tx, mut channel) = make_chan();

//         pdata_tx.send_async("pdata1".to_owned()).await.unwrap();
//         control_tx
//             .send_async(ControlMsg::Ack { id: 1 })
//             .await
//             .unwrap();

//         // Control message should be received first due to bias
//         let msg = channel.recv().await.unwrap();
//         assert!(matches!(msg, Message::Control(ControlMsg::Ack { id: 1 })));

//         // Then pdata message
//         let msg = channel.recv().await.unwrap();
//         assert!(matches!(msg, Message::PData(ref s) if s == "pdata1"));
//     }

//     #[tokio::test]
//     async fn test_shutdown_drain() {
//         let (control_tx, pdata_tx, mut channel) = make_chan();

//         // Pre-load pdata
//         pdata_tx.send_async("pdata1".to_string()).await.unwrap();
//         pdata_tx.send_async("pdata2".to_string()).await.unwrap();

//         // Send shutdown with a deadline
//         control_tx
//             .send_async(ControlMsg::Shutdown {
//                 deadline: Duration::from_millis(100), // 100ms deadline
//                 reason: "Test Shutdown".to_string(),
//             })
//             .await
//             .unwrap();

//         // Send more pdata after shutdown is sent, but before receiver likely gets it
//         pdata_tx.send_async("pdata3".to_string()).await.unwrap();
//         pdata_tx
//             .send_async("pdata4_during_drain".to_string())
//             .await
//             .unwrap();

//         // --- Start Receiving ---

//         // 1. Should receive pdata1 (drain)
//         let msg1 = channel.recv().await.unwrap();
//         assert!(matches!(msg1, Message::PData(ref s) if s == "pdata1"));

//         // 2. Should receive pdata2 (drain)
//         let msg2 = channel.recv().await.unwrap();
//         assert!(matches!(msg2, Message::PData(ref s) if s == "pdata2"));

//         // 3. Should receive pdata3 (drain)
//         let msg3 = channel.recv().await.unwrap();
//         assert!(matches!(msg3, Message::PData(ref s) if s == "pdata3"));

//         // 4. Should receive pdata4 (drain)
//         let msg4 = channel.recv().await.unwrap();
//         assert!(matches!(msg4, Message::PData(ref s) if s == "pdata4_during_drain"));

//         // Wait for deadline to likely expire
//         sleep(Duration::from_millis(120)).await; // Wait longer than deadline

//         // Send pdata *after* deadline
//         // This might get buffered but shouldn't be received before the shutdown msg
//         let _ = pdata_tx
//             .send_async("pdata5_after_deadline".to_string())
//             .await;

//         // 5. Now, should receive the Shutdown message itself
//         let msg5 = channel.recv().await.unwrap();
//         assert!(matches!(
//             msg5,
//             Message::Control(ControlMsg::Shutdown { .. })
//         ));

//         drop(control_tx);
//         drop(pdata_tx); // Close channels

//         // 6. Check for RecvError after channels closed
//         let msg_err = channel.recv().await;
//         assert!(matches!(msg_err, Err(RecvError::Closed)));
//     }

//     #[tokio::test]
//     async fn test_shutdown_drain_pdata_closes() {
//         let (control_tx, pdata_tx, mut channel) = make_chan();

//         // Pre-load pdata
//         pdata_tx.send_async("pdata1".to_string()).await.unwrap();

//         // Send shutdown with a long deadline
//         control_tx
//             .send_async(ControlMsg::Shutdown {
//                 deadline: Duration::from_secs(5), // Long deadline
//                 reason: "Test Shutdown PData Closes".to_string(),
//             })
//             .await
//             .unwrap();

//         sleep(Duration::from_millis(10)).await; // Give receiver a chance

//         // --- Start Receiving ---

//         // 1. Should receive pdata1 (drain)
//         let msg1 = channel.recv().await.unwrap();
//         assert!(matches!(msg1, Message::PData(ref s) if s == "pdata1"));

//         // Close the pdata channel during drain
//         drop(pdata_tx);

//         // 2. Now, should receive the Shutdown message because pdata channel closed
//         let msg2 = channel.recv().await.unwrap();
//         assert!(matches!(
//             msg2,
//             Message::Control(ControlMsg::Shutdown { .. })
//         ));

//         drop(control_tx);

//         // 3. Check for RecvError after channels closed
//         let msg_err = channel.recv().await;
//         assert!(matches!(msg_err, Err(RecvError::Closed)));
//     }

//     #[tokio::test]
//     async fn test_immediate_shutdown() {
//         let (control_tx, pdata_tx, mut channel) = make_chan();

//         pdata_tx.send_async("pdata1".to_string()).await.unwrap();
//         control_tx
//             .send_async(ControlMsg::Shutdown {
//                 deadline: Duration::from_secs(0), // Immediate deadline
//                 reason: "Immediate Shutdown".to_string(),
//             })
//             .await
//             .unwrap();

//         // Should immediately receive the shutdown message, no draining
//         let msg1 = channel.recv().await.unwrap();
//         assert!(matches!(
//             msg1,
//             Message::Control(ControlMsg::Shutdown { .. })
//         ));

//         // Pdata should be ignored and the recv method should return Closed
//         let msg2 = channel.recv().await;
//         assert!(matches!(msg2, Err(RecvError::Closed)));
//     }

//     /// After Shutdown all later control messages are silently dropped (ignored).
//     #[tokio::test]
//     async fn test_ignore_ctrl_after_shutdown() {
//         let (control_tx, pdata_tx, mut chan) = make_chan();

//         control_tx
//             .send_async(ControlMsg::Shutdown {
//                 deadline: Duration::from_secs(0),
//                 reason: "ignore_followups".into(),
//             })
//             .await
//             .unwrap();

//         let msg = chan.recv().await.unwrap();
//         assert!(matches!(msg, Message::Control(ControlMsg::Shutdown { .. })));

//         // Send a control message that should fail as the channel has been closed
//         // following the shutdown.
//         assert!(
//             control_tx
//                 .send_async(ControlMsg::Ack { id: 99 })
//                 .await
//                 .is_err()
//         );

//         // Send a pdata message that should fail as the channel has been closed
//         // following the shutdown.
//         assert!(pdata_tx.send_async("pdata1".to_owned()).await.is_err());

//         // Another recv should report Closed, proving Ack was discarded.
//         assert!(matches!(chan.recv().await, Err(RecvError::Closed)));
//     }

//     /// Immediate shutdown (deadline == 0) returns Shutdown and then behaves Closed.
//     #[tokio::test]
//     async fn test_immediate_shutdown_closed_afterwards() {
//         let (control_tx, _pdata_tx, mut chan) = make_chan();

//         control_tx
//             .send_async(ControlMsg::Shutdown {
//                 deadline: Duration::from_secs(0),
//                 reason: "now".into(),
//             })
//             .await
//             .unwrap();

//         // First recv -> Shutdown
//         let first = chan.recv().await.unwrap();
//         assert!(matches!(
//             first,
//             Message::Control(ControlMsg::Shutdown { .. })
//         ));

//         // Second recv -> channel considered closed
//         assert!(matches!(chan.recv().await, Err(RecvError::Closed)));
//     }
// }
