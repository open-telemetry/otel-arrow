//! Engine registration and lifecycle orchestration for the WEF receiver.
//!
//! This module connects the HTTPS manager to the local dataflow receiver task.
//! HTTP parsing and responses belong to `http`; admission, pending bookmarks,
//! and feedback slots belong to `runtime`. The control loop here forwards decoded
//! batches and routes engine Ack/Nack messages back to their waiting deliveries.
//!
//! # Construction and capacity
//!
//! The factory requires exactly one pipeline core. Multiple replicas would need
//! explicit coordination of listener and source-progress ownership, which this
//! implementation does not provide. Byte-rate admission and memory-pressure state
//! are shared with the engine. The configured in-flight limit is capped by the
//! output pdata channel capacity, treating zero capacity as one for this bound;
//! that does not resize the output channel or guarantee an available send slot.
//!
//! # Pipeline handoff
//!
//! Each queued event batch carries a generation-tagged feedback key. Before
//! forwarding it, the receiver subscribes the pdata context to both Ack and Nack
//! using that key as call data. A failed nonblocking send resolves the delivery
//! with Nack instead of waiting for channel capacity. Successful enqueue alone
//! does not commit progress or permit a protocol Ack.
//!
//! Deliveries whose HTTP waiter has already closed are skipped. Cancellation can
//! still race with forwarding and cannot retract pdata already sent downstream.
//! Late or duplicate feedback is handled by the bridge's feedback registry.
//!
//! # Control and termination
//!
//! | Control message | Receiver behavior |
//! | --- | --- |
//! | Ack / Nack | Resolve the matching HTTP delivery through its feedback key. |
//! | DrainIngress | Stop accepting ingress and retain the first drain deadline. |
//! | Shutdown | Cancel telemetry, force HTTP cancellation, and await server cleanup. |
//! | CollectTelemetry | Report pending measurements; reporting errors are ignored here. |
//! | MemoryPressureChanged | Update the shared admission state. |
//! | Other messages | No receiver-specific action. |
//!
//! During drain, the loop continues forwarding queued work and processing
//! feedback. At the drain deadline it signals forced connection cancellation,
//! then waits for server completion. Repeated DrainIngress messages do not replace
//! that deadline. Shutdown forces cancellation immediately rather than waiting
//! until its supplied deadline; the deadline is carried into the terminal state.
//!
//! Successful server completion during drain notifies the engine that ingress is
//! drained. Normal completion and explicit Shutdown hand remaining metric
//! snapshots to the terminal state. Server errors are propagated as receiver
//! transport errors; cancellation is not a guarantee that downstream work has
//! finished or that Windows received an acknowledgement.

use super::metrics::WefMetrics;
use super::{bookmark::BatchOutcome, runtime::DeliveryBridge};
use super::{config::Config, http::serve_manager};
use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::{error::Error as ConfigError, validation::deserialize_typed_config};
use otel_arrow_dfe_engine::{
    Interests, MessageSourceLocalEffectHandlerExtension, ProducerEffectHandlerExtension,
    ReceiverFactory,
    admission::{AdmissionDimension, SharedAdmissionGate},
    control::NodeControlMsg,
    error::{Error, ReceiverErrorKind, format_error_sources},
    local::receiver as local,
    memory_limiter::SharedReceiverAdmissionState,
    receiver::ReceiverWrapper,
    terminal_state::TerminalState,
};
use otel_arrow_dfe_otap::{OTAP_RECEIVER_FACTORIES, pdata::OtapPdata};
use serde_json::Value;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::oneshot;

/// URN of the Windows Event Forwarding receiver.
pub const WINDOWS_EVENT_FORWARDING_URN: &str = "urn:otel:receiver:windows_event_forwarding";

/// Register the single-replica WEF receiver with the dataflow engine.
///
/// Creation enforces the pipeline core count, binds framework byte admission,
/// validates configuration, and registers metrics before returning a local
/// receiver wrapper. The standalone validation callback checks configuration only;
/// it cannot validate pipeline core count or output channel capacity.
/// Listener acquisition and TLS credential loading are deferred until startup.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Receiver)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static WINDOWS_EVENT_FORWARDING: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: WINDOWS_EVENT_FORWARDING_URN,
    create: |pipeline, node, node_config, receiver_config, _capabilities| {
        if pipeline.num_cores() != 1 {
            return Err(ConfigError::InvalidUserConfig {
                error: "windows_event_forwarding requires exactly one pipeline core".to_owned(),
            });
        }
        let admission =
            SharedReceiverAdmissionState::from_process_state(&pipeline.memory_pressure_state());
        let rate_limiter = pipeline
            .admission()
            .bind_shared(AdmissionDimension::Bytes, admission.clone())
            .map_err(|error| ConfigError::InvalidUserConfig {
                error: error.to_string(),
            })?;
        let mut config = parse_config(&node_config.config)?;
        config.limits.max_in_flight_batches = config
            .limits
            .max_in_flight_batches
            .min(receiver_config.output_pdata_channel.capacity.max(1));
        Ok(ReceiverWrapper::local(
            WindowsEventForwardingReceiver {
                config,
                metrics: Arc::new(Mutex::new(WefMetrics::register(&pipeline))),
                admission,
                rate_limiter,
            },
            node,
            node_config,
            receiver_config,
        ))
    },
    validate_config: |raw| parse_config(raw).map(|_| ()),
    context_declarations: None,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
};

/// Deserialize and validate settings without constructing an advertisement.
///
/// Startup constructs the actual advertisement using the configured node name as
/// its collector namespace. This helper does not bind sockets or load TLS files.
fn parse_config(raw: &Value) -> Result<Config, ConfigError> {
    let config: Config = deserialize_typed_config(raw)?;
    config
        .validate()
        .map_err(|error| ConfigError::InvalidUserConfig { error })?;
    Ok(config)
}

/// Local receiver state before startup creates the HTTP-to-pipeline bridge.
///
/// Metrics and memory-pressure state are shared with HTTP request handling.
/// Bookmark and feedback state are created at startup and are not persisted.
struct WindowsEventForwardingReceiver {
    config: Config,
    metrics: Arc<Mutex<WefMetrics>>,
    admission: SharedReceiverAdmissionState,
    rate_limiter: Option<SharedAdmissionGate>,
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for WindowsEventForwardingReceiver {
    /// Acquire the listener, start telemetry, and drive HTTP, control, and delivery work.
    ///
    /// A one-shot signal requests graceful ingress shutdown; a separate bridge
    /// token interrupts active HTTP work when draining expires or Shutdown arrives.
    /// The server future is polled in this task, while it manages connection tasks.
    ///
    /// The biased selection checks control messages before server completion,
    /// drain expiry, and queued deliveries. This allows feedback and lifecycle
    /// messages to be handled without blocking on downstream send capacity.
    /// Periodic telemetry is requested every second and cancelled on the explicit
    /// Shutdown and server-completion paths. Listener, timer, control-channel,
    /// transport, and drain-notification errors propagate to the engine.
    async fn start(
        self: Box<Self>,
        mut control: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let transport_error = |error: std::io::Error| Error::ReceiverError {
            receiver: effect_handler.receiver_id(),
            kind: ReceiverErrorKind::Transport,
            error: "WEF SubscriptionManager transport failed".to_owned(),
            source_detail: format_error_sources(&error),
        };
        let listener = effect_handler.tcp_listener(self.config.endpoint)?;
        let telemetry_timer = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;
        let (stop, stopped) = oneshot::channel();
        let mut stop = Some(stop);
        let mut draining_deadline = None;
        let mut force_deadline: Option<std::time::Instant> = None;
        let receiver_id = effect_handler.receiver_id();
        let (bridge, mut deliveries) = DeliveryBridge::new(&self.config.limits);
        let bridge = bridge
            .with_metrics(Arc::clone(&self.metrics))
            .with_admission(self.admission.clone(), self.rate_limiter);
        let force_stop = bridge.force_stop.clone();
        let feedback = bridge.feedback.clone();
        let server = serve_manager(
            listener,
            self.config,
            receiver_id.name.as_ref(),
            bridge,
            async {
                let _ = stopped.await;
            },
        );
        tokio::pin!(server);
        loop {
            tokio::select! {
                biased;
                message = control.recv() => {
                    match message.map_err(Error::ChannelRecvError)? {
                        NodeControlMsg::Ack(ack) => {
                            feedback.resolve(ack.unwind.route.calldata, BatchOutcome::Ack);
                        }
                        NodeControlMsg::Nack(nack) => {
                            feedback.resolve(nack.unwind.route.calldata, BatchOutcome::Nack);
                        }
                        NodeControlMsg::DrainIngress { deadline, .. } => {
                            if draining_deadline.is_none() {
                                draining_deadline = Some(deadline);
                                force_deadline = Some(deadline);
                                if let Some(stop) = stop.take() { let _ = stop.send(()); }
                            }
                        }
                        NodeControlMsg::Shutdown { deadline, .. } => {
                            let _ = telemetry_timer.cancel().await;
                            force_stop.cancel();
                            if let Some(stop) = stop.take() { let _ = stop.send(()); }
                            server.await.map_err(transport_error)?;
                            return Ok(TerminalState::new(deadline, self.metrics.lock().expect("WEF metrics lock poisoned").terminal_snapshots()));
                        }
                        NodeControlMsg::CollectTelemetry { mut metrics_reporter } => {
                            let _ = self.metrics.lock().expect("WEF metrics lock poisoned").report(&mut metrics_reporter);
                        }
                        NodeControlMsg::MemoryPressureChanged { update } => self.admission.apply(update),
                        _ => {}
                    }
                }
                result = &mut server => {
                    let _ = telemetry_timer.cancel().await;
                    result.map_err(transport_error)?;
                    if draining_deadline.is_some() { effect_handler.notify_receiver_drained().await?; }
                    return Ok(TerminalState::new(draining_deadline.unwrap_or_else(std::time::Instant::now), self.metrics.lock().expect("WEF metrics lock poisoned").terminal_snapshots()));
                }
                _ = async {
                    match force_deadline {
                        Some(deadline) => tokio::time::sleep_until(deadline.into()).await,
                        None => std::future::pending::<()>().await,
                    }
                } => {
                    force_stop.cancel();
                    force_deadline = None;
                }
                Some(delivery) = deliveries.recv() => {
                    if delivery.feedback.is_closed() { continue; }
                    let mut pdata = OtapPdata::new_todo_context(delivery.records.into());
                    effect_handler.subscribe_to(Interests::ACKS | Interests::NACKS, delivery.feedback.calldata(), &mut pdata);
                    if effect_handler.try_send_message_with_source_node(pdata).is_err() {
                        let _ = delivery.feedback.send(BatchOutcome::Nack);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::wsman::messages::subscription::Advertisement;
    use super::*;
    use otel_arrow_dfe_config::node::NodeUserConfig;
    use otel_arrow_dfe_engine::{
        capability::registry::Capabilities,
        context::ControllerContext,
        testing::{receiver::TestRuntime, test_node},
    };
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use otel_arrow_dfe_test_tls_certs::{ExtendedKeyUsage, generate_ca};
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };

    fn sample_config() -> Value {
        let pipeline: Value = serde_yaml::from_str(include_str!(
            "../../../../../configs/windows-event-forwarding-console.yaml"
        ))
        .unwrap();
        pipeline["groups"]["default"]["pipelines"]["main"]["nodes"]["windows_event_forwarding"]["config"].clone()
    }

    /// Scenario: the engine creates WEF from the shipped pipeline with one or multiple cores.
    /// Guarantees: factory registration resolves, semantic config validation runs, and replicas are rejected.
    #[test]
    fn engine_factory_validation() {
        assert!(
            OTAP_RECEIVER_FACTORIES
                .iter()
                .any(|factory| factory.name == WINDOWS_EVENT_FORWARDING_URN)
        );
        let raw = sample_config();
        (WINDOWS_EVENT_FORWARDING.validate_config)(&raw).unwrap();
        let mut invalid = raw.clone();
        invalid["auth"]["allowed_sources"] = serde_json::json!([]);
        assert!((WINDOWS_EVENT_FORWARDING.validate_config)(&invalid).is_err());
        for query in ["<QueryList>", "<Query/>", "<QueryList xmlns='urn:other'/>"] {
            let mut invalid = raw.clone();
            invalid["subscriptions"][0]["query"] = serde_json::json!(query);
            assert!(matches!(
                (WINDOWS_EVENT_FORWARDING.validate_config)(&invalid),
                Err(ConfigError::InvalidUserConfig { .. })
            ));
        }
        let runtime = TestRuntime::<OtapPdata>::new();
        let controller = ControllerContext::new(TelemetryRegistryHandle::new());
        for cores in [1, 2] {
            let pipeline =
                controller.pipeline_context_with("default".into(), "main".into(), 0, cores, 0);
            let mut node_config = NodeUserConfig::new_receiver_config(WINDOWS_EVENT_FORWARDING_URN);
            node_config.config = raw.clone();
            let result = (WINDOWS_EVENT_FORWARDING.create)(
                pipeline,
                test_node(runtime.config().name.clone()),
                Arc::new(node_config),
                runtime.config(),
                &Capabilities::empty(),
            );
            assert_eq!(result.is_ok(), cores == 1);
        }
    }

    /// Scenario: a registered WEF receiver receives authenticated events, downstream feedback, and shutdown.
    /// Guarantees: complete batches reach the engine, Ack/Nack control messages determine HTTP success, and shutdown exits.
    #[test]
    fn engine_delivery_and_shutdown() {
        run_engine_delivery(false, false);
    }

    /// Scenario: ingress draining begins while an authenticated Events request awaits downstream Ack.
    /// Guarantees: the control loop still routes feedback and returns HTTP success before completing drain.
    #[test]
    fn engine_drain_completes_pending_ack() {
        run_engine_delivery(true, false);
    }

    /// Scenario: a downstream request remains unresolved past the ingress drain deadline.
    /// Guarantees: the HTTP waiter is cancelled without a success response and the receiver exits within the deadline budget.
    #[test]
    fn engine_drain_deadline_cancels_pending_request() {
        run_engine_delivery(true, true);
    }

    fn run_engine_delivery(drain: bool, expire: bool) {
        use super::super::http::tests::{connector, exchange, request};
        use otel_arrow_dfe_engine::control::{AckMsg, NackMsg};
        use otel_arrow_dfe_otap::testing::{next_ack, next_nack};
        use tokio::time::timeout;

        let _ = rustls::crypto::ring::default_provider().install_default();
        let directory = tempfile::tempdir().unwrap();
        let ca = generate_ca("WEF engine test CA");
        ca.write_cert_to_dir(directory.path(), "ca");
        ca.issue_leaf(
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        )
        .write_to_dir(directory.path(), "server");
        let client = ca.issue_leaf(
            "host.example",
            Some("host.example"),
            Some(ExtendedKeyUsage::ClientAuth),
        );
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let mut raw = sample_config();
        raw["endpoint"] = serde_json::json!(address.to_string());
        raw["public_endpoint"] = serde_json::json!("https://localhost");
        raw["auth"]["allowed_sources"] = serde_json::json!(["host.example"]);
        raw["tls"] = serde_json::json!({
            "cert_file": directory.path().join("server.crt"),
            "key_file": directory.path().join("server.key"),
            "client_ca_files": [directory.path().join("ca.crt")]
        });
        let runtime = TestRuntime::<OtapPdata>::new();
        let advertisement = Advertisement::from_config(
            &parse_config(&raw).unwrap(),
            runtime.config().name.as_ref(),
        )
        .unwrap()
        .unwrap();
        let controller = ControllerContext::new(TelemetryRegistryHandle::new());
        let pipeline = controller.pipeline_context_with("default".into(), "main".into(), 0, 1, 0);
        let mut node_config = NodeUserConfig::new_receiver_config(WINDOWS_EVENT_FORWARDING_URN);
        node_config.config = raw;
        let receiver = (WINDOWS_EVENT_FORWARDING.create)(
            pipeline,
            test_node(runtime.config().name.clone()),
            Arc::new(node_config),
            runtime.config(),
            &Capabilities::empty(),
        )
        .unwrap();
        drop(listener);
        runtime
            .set_receiver(receiver)
            .run_test(move |context| async move {
                timeout(Duration::from_secs(5), async {
                    loop {
                        if tokio::net::TcpStream::connect(address).await.is_ok() { break; }
                        tokio::task::yield_now().await;
                    }
                }).await.unwrap();
                let connector = connector(&ca.cert_pem, Some(&client));
                let path = format!("/wsman/subscriptions/{}/previous-version", advertisement.identifier());
                for (index, expected) in [hyper::StatusCode::OK, if drain { hyper::StatusCode::OK } else { hyper::StatusCode::SERVICE_UNAVAILABLE }].into_iter().enumerate() {
                    let xml = format!(r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:a="http://schemas.xmlsoap.org/ws/2004/08/addressing" xmlns:w="http://schemas.dmtf.org/wbem/wsman/1/wsman.xsd" xmlns:e="http://schemas.xmlsoap.org/ws/2004/08/eventing"><s:Header><a:Action>http://schemas.dmtf.org/wbem/wsman/1/wsman/Events</a:Action><a:MessageID>uuid:engine-test</a:MessageID><a:To>https://localhost{path}</a:To><e:Identifier>{}</e:Identifier><w:AckRequested/></s:Header><s:Body><w:Events><w:Event Action="http://schemas.dmtf.org/wbem/wsman/1/wsman/Event"><![CDATA[<Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event"><System><EventID>42</EventID><TimeCreated SystemTime="2026-09-22T19:28:11Z"/></System><RenderingInfo><Message>engine test</Message></RenderingInfo></Event>]]></w:Event></w:Events></s:Body></s:Envelope>"#, advertisement.identifier());
                    let response = timeout(Duration::from_secs(5), exchange(address, &connector, request("POST", &path, "application/soap+xml;charset=utf-8", xml.into_bytes()))).await.unwrap();
                    if expire && index == 1 { assert!(response.is_err()); }
                    else { assert_eq!(response.unwrap().0, expected); }
                }
                if !drain { context
                    .send_shutdown(Instant::now() + Duration::from_secs(5), "test shutdown")
                    .await
                    .unwrap(); }
            })
            .run_validation_concurrent(move |mut context| async move {
                let pdata = timeout(Duration::from_secs(5), context.recv()).await.unwrap().unwrap();
                let (_, ack) = next_ack(AckMsg::new(pdata)).expect("receiver Ack subscription");
                context.send_control_msg(NodeControlMsg::Ack(ack)).await.unwrap();
                let pdata = timeout(Duration::from_secs(5), context.recv()).await.unwrap().unwrap();
                if drain {
                    context.send_control_msg(NodeControlMsg::DrainIngress {
                        deadline: Instant::now() + if expire { Duration::from_millis(50) } else { Duration::from_secs(5) },
                        reason: "test drain".into(),
                    }).await.unwrap();
                    if !expire {
                        let (_, ack) = next_ack(AckMsg::new(pdata)).expect("receiver Ack subscription");
                        context.send_control_msg(NodeControlMsg::Ack(ack)).await.unwrap();
                    }
                } else {
                    let (_, nack) = next_nack(NackMsg::new("test rejection", pdata)).expect("receiver Nack subscription");
                    context.send_control_msg(NodeControlMsg::Nack(nack)).await.unwrap();
                }
                assert!(timeout(Duration::from_secs(5), context.recv()).await.unwrap().is_err());
            });
    }
}
