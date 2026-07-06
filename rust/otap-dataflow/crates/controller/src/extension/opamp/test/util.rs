// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for testing OpAMP Controller Extension

use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use axum::{
    Router,
    extract::{FromRequest, Request, State, WebSocketUpgrade, ws::Message},
    response::IntoResponse,
    routing::any,
    serve,
};
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use otap_df_admin::{
    ControlPlane, ControlPlaneError, EngineConfigReconcileRequest, EngineConfigReconcileState,
    EngineConfigReconcileStatus,
};
use otap_df_config::{
    engine::{EngineConfig, OtelDataflowSpec},
    pipeline_group::PipelineGroupConfig,
    policy::Policies,
};
use prost::Message as _;
use tokio::{net::TcpListener, sync::RwLock};
use tokio_util::sync::CancellationToken;

use crate::extension::opamp::proto::opamp::v1::{
    AgentConfigFile, AgentConfigMap, AgentRemoteConfig, AgentToServer, ServerToAgent,
};

/// Returns an empty Engine configuration
pub fn empty_engine_config() -> OtelDataflowSpec {
    OtelDataflowSpec::from_yaml("version: otel_dataflow/v1").unwrap()
}

/// Mock [`ControlPlane`] implementation for testing behaviour of OpAMP controller extension.
///
/// Many of the [`ControlPlane`] trait methods will return an error that they are unimplemented
/// methods may be given a mock implementation as the need arises and the controller extension's
/// interaction with the control plane become more sophisticated and require more testing.
pub(crate) struct MockControlPlane {
    current_config: Mutex<OtelDataflowSpec>,
    reconcile_result: Mutex<Result<EngineConfigReconcileStatus, ControlPlaneError>>,
}

impl MockControlPlane {
    pub fn new(initial_config: OtelDataflowSpec) -> Self {
        let default_result = Ok(EngineConfigReconcileStatus::new(
            "test-reconcile-0".into(),
            EngineConfigReconcileState::Succeeded,
            None,
            "2026-06-01T00:00:00Z".into(),
        ));

        Self {
            current_config: Mutex::new(initial_config),
            reconcile_result: Mutex::new(default_result),
        }
    }

    pub fn set_reconcile_result(
        &self,
        result: Result<EngineConfigReconcileStatus, ControlPlaneError>,
    ) {
        *self.reconcile_result.lock().unwrap() = result;
    }
}

impl ControlPlane for MockControlPlane {
    fn engine_config_snapshot(&self) -> Result<OtelDataflowSpec, ControlPlaneError> {
        Ok(self.current_config.lock().unwrap().clone())
    }

    fn reconcile_engine_config(
        &self,
        request: EngineConfigReconcileRequest,
    ) -> Result<EngineConfigReconcileStatus, ControlPlaneError> {
        *self.current_config.lock().unwrap() = request.config;
        self.reconcile_result.lock().unwrap().clone()
    }

    fn shutdown_all(&self, _timeout_secs: u64) -> Result<(), ControlPlaneError> {
        Err(not_implemented())
    }

    fn shutdown_pipeline(
        &self,
        _pipeline_group_id: &str,
        _pipeline_id: &str,
        _timeout_secs: u64,
    ) -> Result<otap_df_admin::ShutdownStatus, ControlPlaneError> {
        Err(not_implemented())
    }

    fn reconfigure_pipeline(
        &self,
        _pipeline_group_id: &str,
        _pipeline_id: &str,
        _request: otap_df_admin::ReconfigureRequest,
    ) -> Result<otap_df_admin::RolloutStatus, ControlPlaneError> {
        Err(not_implemented())
    }

    fn pipeline_details(
        &self,
        _pipeline_group_id: &str,
        _pipeline_id: &str,
    ) -> Result<Option<otap_df_admin::PipelineDetails>, ControlPlaneError> {
        Err(not_implemented())
    }

    fn rollout_status(
        &self,
        _pipeline_group_id: &str,
        _pipeline_id: &str,
        _rollout_id: &str,
    ) -> Result<Option<otap_df_admin::RolloutStatus>, ControlPlaneError> {
        Err(not_implemented())
    }

    fn shutdown_status(
        &self,
        _pipeline_group_id: &str,
        _pipeline_id: &str,
        _shutdown_id: &str,
    ) -> Result<Option<otap_df_admin::ShutdownStatus>, ControlPlaneError> {
        Err(not_implemented())
    }
}

fn not_implemented() -> ControlPlaneError {
    ControlPlaneError::InvalidRequest {
        message: "not yet implemented".into(),
    }
}

/// The expected instance_uid (big-endian bytes of UUID 8be4df61-93ca-11d2-aa0d-00e098032b8c,
/// i.e. the standard RFC 4122 byte order).
///
/// This is the instance_uid used in many tests in this module.
pub const EXPECTED_INSTANCE_UID_BYTES: [u8; 16] = [
    139, 228, 223, 97, 147, 202, 17, 210, 170, 13, 0, 224, 152, 3, 43, 140,
];

pub const EXPECTED_INSTANCE_UID_STR: &str = "8be4df61-93ca-11d2-aa0d-00e098032b8c";

/// Constructs a simple [`OtelDataflowSpec`] with a single pipeline group
pub fn test_config() -> OtelDataflowSpec {
    OtelDataflowSpec {
        version: "otel_dataflow/v1".into(),
        policies: Policies::default(),
        topics: HashMap::new(),
        engine: EngineConfig::default(),
        groups: HashMap::from_iter([(Cow::Borrowed("test_pipeline"), PipelineGroupConfig::new())]),
    }
}

/// Constructs a [`ServerToAgent`] message containing the passed config & config hash
pub fn server_to_agent_with_config(
    config: &OtelDataflowSpec,
    config_hash: Vec<u8>,
) -> ServerToAgent {
    ServerToAgent {
        remote_config: Some(AgentRemoteConfig {
            config_hash,
            config: Some(AgentConfigMap {
                config_map: HashMap::from_iter([(
                    "desired_state".into(),
                    AgentConfigFile {
                        content_type: "application/json".into(),
                        body: serde_json::to_vec(config).unwrap(),
                    },
                )]),
            }),
        }),
        instance_uid: EXPECTED_INSTANCE_UID_BYTES.to_vec(),
        ..Default::default()
    }
}

/// Mock OpAMP Server implementation serving a websocket.
///
/// The implementation keeps a copy of the request it receives, and replies with a scripted set
/// of responses to each [`AgentToServer`] message it receives.
pub(crate) struct MockWebSocketServer {
    /// The server will bind this port on the loopback IP
    port: u16,

    /// state of the server
    state: ServerState,
}

impl MockWebSocketServer {
    pub fn new(port: u16, responses: Vec<Option<ServerToAgent>>) -> Self {
        Self {
            port,
            state: ServerState::new(Arc::new(RwLock::new(responses.into_iter().collect()))),
        }
    }

    /// Get a clone of the server's state
    pub fn state(&self) -> ServerState {
        self.state.clone()
    }

    pub async fn serve(&mut self, cancellation_token: CancellationToken) {
        let state = self.state.clone();
        let app = Router::new()
            .route("/v1/opamp", any(request_handler))
            .with_state(state);

        let bind_addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&bind_addr).await.unwrap();

        serve(listener, app)
            .with_graceful_shutdown(cancellation_token.cancelled_owned())
            .await
            .unwrap();
    }
}

/// handle a request make to the MockWebSocketServer
async fn request_handler(State(state): State<ServerState>, request: Request) -> impl IntoResponse {
    let upgrade = WebSocketUpgrade::from_request(request, &()).await.unwrap();
    upgrade
        .on_upgrade(async move |socket| {
            let (mut ws_sender, mut ws_receiver) = socket.split();

            loop {
                // save the request
                let Some(rx_result) = ws_receiver.next().await else {
                    // break loop of handing websocket messages when receiver is finished
                    return;
                };

                let request = match rx_result {
                    Err(e) => {
                        let inner_err = e.into_inner();
                        if format!("{:?}", inner_err)
                            .contains("Protocol(ResetWithoutClosingHandshake)")
                        {
                            // test suite doesn't have a very graceful shutdown, so if the other
                            // side hung up, just return assuming test is done
                            return;
                        } else {
                            panic!("{}", inner_err)
                        }
                    }
                    Ok(r) => r,
                };

                let Message::Binary(mut bytes) = request else {
                    panic!("invalid message: {request:?}")
                };

                // Per the OpAMP spec websocket framing, each message starts with a
                // varint-encoded header which must currently be zero.
                let header = prost::encoding::decode_varint(&mut bytes).unwrap();
                assert_eq!(header, 0, "expected zero websocket message header");

                let agent_to_server = AgentToServer::decode(bytes).unwrap();
                {
                    let mut requests = state.requests.write().await;
                    requests.push(agent_to_server);
                }

                // reply if the response is configured
                let next_response = {
                    let mut responses = state.responses.write().await;
                    responses.pop_front()
                };
                _ = state.num_message_exchanges.fetch_add(1, Ordering::Relaxed);

                match next_response {
                    Some(Some(response)) => {
                        // serialize and send the next response, prefixed with the
                        // varint-encoded zero header required by the OpAMP websocket framing
                        let mut bytes = vec![0u8];
                        response.encode(&mut bytes).unwrap();
                        ws_sender
                            .send(Message::Binary(Bytes::from(bytes)))
                            .await
                            .unwrap()
                    }
                    Some(None) => {
                        // nothing to send
                        continue;
                    }
                    None => {
                        // No more scripted responses - close this connection.
                        // Note: the server does keep running and could accept a new connection
                        return;
                    }
                }
            }
        })
        .into_response()
}

#[derive(Clone)]
pub(crate) struct ServerState {
    requests: Arc<RwLock<Vec<AgentToServer>>>,
    responses: Arc<RwLock<VecDeque<Option<ServerToAgent>>>>,
    num_message_exchanges: Arc<AtomicUsize>,
}

impl ServerState {
    fn new(responses: Arc<RwLock<VecDeque<Option<ServerToAgent>>>>) -> Self {
        Self {
            responses,
            requests: Arc::new(RwLock::new(Vec::new())),
            num_message_exchanges: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn num_message_exchanges(&self) -> usize {
        self.num_message_exchanges.load(Ordering::Acquire)
    }

    pub async fn take_requests(&self) -> Vec<AgentToServer> {
        let mut guard = self.requests.write().await;
        std::mem::take(&mut *guard)
    }

    pub async fn wait_for_exchanges(&self, n: usize, timeout: Duration) {
        let start = Instant::now();
        loop {
            if self.num_message_exchanges() >= n {
                return;
            }

            assert!(
                start.elapsed() < timeout,
                "timed out waiting for {n} message exchanges (got {})",
                self.num_message_exchanges()
            );

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}
