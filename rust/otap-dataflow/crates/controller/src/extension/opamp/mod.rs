// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OpAMP controller extension.
//!
//! This allows the engine to be controlled by a connection to a remote OpAMP server.
//!
//! Example:
//! ```yaml
//! version: otel_dataflow/v1
//! engine:
//!   controller:
//!     extensions:
//!       opamp:
//!         type: "urn:otel:extension:opamp"
//!         config:
//!           endpoint: "ws://127.0.0.1:4320/v1/opamp"
//! ```
//!
//! See [config] for more configuration options.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use linkme::distributed_slice;
use otap_df_admin::{
    ControlPlaneError, EngineConfigReconcileRequest, EngineConfigReconcileState,
    EngineConfigReconcileStatus,
};
use otap_df_config::engine::OtelDataflowSpec;
use otap_df_state::phase::PipelinePhase;
use prost::Message as _;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use otap_df_config::PipelineKey;
use otap_df_config::error::Error as ConfigError;
use otap_df_state::pipeline_status::PipelineStatus;
use otap_df_telemetry::{otel_debug, otel_error, otel_info, otel_warn};

use crate::extension::opamp::config::Config;
use crate::extension::opamp::consts::health_status;
use crate::extension::opamp::error::Error;
use crate::extension::opamp::proto::opamp::v1::server_error_response::Details;
use crate::extension::opamp::proto::opamp::v1::{
    AgentCapabilities, AgentConfigFile, AgentConfigMap, AgentDescription, AgentDisconnect,
    AgentIdentification, AgentToServer, AgentToServerFlags, CommandType, ComponentHealth,
    ConnectionSettingsStatus, ConnectionSettingsStatuses, CustomCapabilities, CustomMessage,
    EffectiveConfig, KeyValue, OpAmpConnectionSettings, RemoteConfigStatus, RemoteConfigStatuses,
    ServerErrorResponseType, ServerToAgent, ServerToAgentFlags,
};
use crate::extension::opamp::util::ExponentialBackoff;
use crate::{
    CONTROLLER_EXTENSION_FACTORIES, ControllerExtensionContext, ControllerExtensionError,
    ControllerExtensionFactory, ControllerExtensionTaskFactory,
};

pub mod config;
pub mod consts;
pub mod error;
pub mod proto;
mod util;

const CONTROL_EXTENSION_URN: &str = "urn:otel:extension:opamp";

/// Custom capability type - represents the custom message which can be sent by this OpAMP agent
/// implementation containing the full pipeline status.
const CUSTOM_CAPABILITY_STATUS: &str = "io.opentelemetry.otap-dfe.pipeline-status/v1";

/// Header prepended to every OpAMP WebSocket message. Per the OpAMP spec, each WebSocket
/// message is a varint-encoded unsigned 64 bit integer header (currently always zero)
/// followed by the protobuf-encoded message bytes.
/// See https://opentelemetry.io/docs/specs/opamp/#websocket-message-format
const WS_MESSAGE_HEADER: u64 = 0;

/// Factory registration of OpAMP Controller Extension
#[allow(unsafe_code)]
#[distributed_slice(CONTROLLER_EXTENSION_FACTORIES)]
pub static OPAMP_CONTROLLER_EXTENSION: ControllerExtensionFactory = ControllerExtensionFactory {
    name: CONTROL_EXTENSION_URN,
    description: "OpAMP controller extension",
    documentation_url: "",
    validate_config,
    start,
};

fn validate_config(config: &serde_json::Value) -> Result<(), ConfigError> {
    let config: Config =
        serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })?;

    config
        .validate()
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })
}

fn start(
    context: ControllerExtensionContext,
) -> Result<ControllerExtensionTaskFactory, ControllerExtensionError> {
    // safety: we can 'expect' the deserialization to have succeeded because it's also deserialized
    // in the `validate_config` call
    let config: Config =
        serde_json::from_value(context.extension.config.clone()).expect("config validated");

    Ok(Box::new(move |cancellation_token| {
        Box::pin(
            async move { run_websocket_connect_loop(context, config, cancellation_token).await },
        )
    }))
}

/// State of session between OpAMP Agent (this client) and control planes OpAMP server
struct SessionState {
    // The OpAMP Agent's instance_uid. This will be initialized from config (if set), otherwise
    // a UUIDv7 will be generated.
    instance_uid: [u8; 16],

    /// OpAMP sequence number. Increments by one for each new message sent to server and used
    /// to detect missing sequences of messages.
    sequence_num: u64,

    /// The last config_hash received from the server a part of a [`AgentRemoteConfig`] message.
    /// This is used by the agent when replying to the server to fill in the value of
    /// [`RemoteConfigStatus::last_remote_config_hash`].
    last_config_hash: Option<Vec<u8>>,

    /// The result the attempt to reconcile the previous configuration sent from the server.
    /// This will be used by the agent when replying to the server to fill in
    /// [`RemoteConfigStats::status`] and any associated errors.
    last_reconcile_result: Option<Result<EngineConfigReconcileStatus, ControlPlaneError>>,

    /// Phase of message exchange. When connected, the client runs a simple loop and this
    /// phase determines the behaviour of each iteration. See [`Phase`] enum.
    phase: Phase,

    /// this is retained for cases where the server tells us to re-send the last message
    last_sent_message: Option<AgentToServer>,

    /// when the server requests the previous message be resent after a reconnect (by sending the
    /// error response of  type `Unavailable` See
    /// https://opentelemetry.io/docs/specs/opamp/#throttling  for more information.), the client
    /// enters the retry/backoff phase.
    ///
    /// This is value for how long to backoff after reconnecting before resending the message.
    retry_backoff_ns: Option<u64>,

    /// The start time of the controller extension (e..g start time of the data-plane itself).
    /// This will be reported to the server as part of the [`AgentToServer::health`] message.
    start_time_unix_nano: u64,
}

impl SessionState {
    fn try_new(config: &Config) -> Result<Self, Error> {
        let instance_uid = match &config.instance_uid {
            Some(instance_uid) => {
                // safety: this was validated in validate_config
                Uuid::parse_str(instance_uid).expect("valid UUID")
            }
            None => Uuid::now_v7(),
        }
        .into_bytes();

        Ok(Self {
            instance_uid,
            sequence_num: 0,
            phase: Phase::Initiating,
            last_sent_message: None,
            last_config_hash: None,
            last_reconcile_result: None,
            retry_backoff_ns: None,
            start_time_unix_nano: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("now after epoch")
                .as_nanos() as u64,
        })
    }
}

/// Phase of execution of client session. When the client is connected, this controls what
/// action the client will perform next.
#[derive(PartialEq)]
enum Phase {
    /// Session initiating - the client will craft an initial [`ServerToAgent`] message
    /// containing the full state.
    Initiating,

    /// The client is exchanging a stream of messages with the server. In this phase, the client is
    /// waiting to receive a [`ServerToAgent`] message with updated config while sending periodic
    /// heartbeat's to the server.
    ExchangeMessages,

    /// The client waits some time, then resends the previous message. This occurs when  server
    /// has sent an error reply with type `Unavailable`, meaning the client must backoff then retry
    /// previous message (see See https://opentelemetry.io/docs/specs/opamp/#throttling).
    RetryBackoff,
}

/// This "outer" loop connects the websocket, and then passes websocket the inner
/// `run_request_loop` which drives the exchange of messages until either cancelled or the
/// websocket must disconnect for some reason.
async fn run_websocket_connect_loop(
    context: ControllerExtensionContext,
    config: Config,
    cancellation_token: CancellationToken,
) -> Result<(), ControllerExtensionError> {
    let mut session_state = SessionState::try_new(&config)?;

    let mut retry_connect_backoff = config.connect_retry.clone();
    let retry_connect_initial_backoff = retry_connect_backoff.next_delay();
    retry_connect_backoff.set_current(retry_connect_initial_backoff); // reset

    loop {
        // connect websocket
        let Some(ws_stream) =
            connect_websocket(&config, &cancellation_token, &mut retry_connect_backoff).await
        else {
            break;
        };

        // enter inner loop where client and server exchange messages over the connected websocket.
        let stats = run_websocket_request_loop(
            &context,
            &config,
            &cancellation_token,
            &mut session_state,
            ws_stream,
        )
        .await;

        // If only a single message was exchanged, perform an backoff before we reconnect.
        // This prevents case where server does something that causes a socket disconnect (like
        // returning an unexpected error), and there being a tight loop of retrying the initial
        // AgentToServer request.
        if stats.messages_sent <= 1 || stats.messages_received <= 1 {
            let delay = retry_connect_backoff.next_delay();
            if cancellation_token
                .run_until_cancelled(tokio::time::sleep(delay))
                .await
                .is_none()
            {
                break;
            }
        } else {
            retry_connect_backoff.set_current(retry_connect_initial_backoff); // reset
        }

        // if we're not in the "disconnect then retry" phase, always restart communication from
        // the initial message
        if session_state.phase != Phase::RetryBackoff {
            session_state.phase = Phase::Initiating;
        }
    }

    Ok(())
}

/// Connect to the websocket. This will continue to retry connecting until cancelled and returns
/// `None` if cancelled before connecting.
async fn connect_websocket(
    config: &Config,
    cancellation_token: &CancellationToken,
    backoff: &mut ExponentialBackoff,
) -> Option<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    loop {
        let connect_result = cancellation_token
            .run_until_cancelled(connect_async(&config.endpoint))
            .await?;

        match connect_result {
            Ok((stream, _response)) => {
                return Some(stream);
            }

            // backoff and retry connection
            Err(e) => {
                let retry_in = backoff.next_delay();
                otel_warn!(
                    "opamp.controller_extension.ws_connect.error",
                    message = "Error initiating websocket",
                    error =? e,
                    retry_in =? retry_in
                );

                cancellation_token
                    .run_until_cancelled(tokio::time::sleep(retry_in))
                    .await?;
            }
        }
    }
}

/// Statistics about how many messages were received by the connected websocket.
///
/// This is used by the loop that controls reestablishing the connection so it knows how quickly
/// to retry connecting.
#[derive(Default)]
struct WebsocketRequestLoopStats {
    messages_sent: usize,
    messages_received: usize,
}

async fn run_websocket_request_loop(
    context: &ControllerExtensionContext,
    config: &Config,
    cancellation_token: &CancellationToken,
    session_state: &mut SessionState,
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> WebsocketRequestLoopStats {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut stats = WebsocketRequestLoopStats::default();

    loop {
        match session_state.phase {
            Phase::Initiating => {
                let message = initial_message(session_state, config, context);
                let send_result = send_message(&message, cancellation_token, &mut ws_sender).await;
                if !is_message_send_success(send_result) {
                    return stats;
                }

                stats.messages_sent += 1;
                session_state.last_sent_message = Some(message);
                session_state.phase = Phase::ExchangeMessages;
            }

            Phase::RetryBackoff => {
                // wait the configured backoff
                let wait_ns = session_state.retry_backoff_ns.unwrap_or_default();
                if cancellation_token
                    .run_until_cancelled(tokio::time::sleep(Duration::from_nanos(wait_ns)))
                    .await
                    .is_none()
                {
                    // cancellation token was cancelled during backoff
                    return stats;
                }

                let Some(last_message) = session_state.last_sent_message.as_ref() else {
                    otel_warn!(
                        "opamp.controller_extension.retry.invalid_state",
                        message = "RetryBackoff phase entered with no last_sent_message; resetting to Initiating"
                    );
                    session_state.phase = Phase::Initiating;
                    continue;
                };

                let send_result =
                    send_message(last_message, cancellation_token, &mut ws_sender).await;
                if !is_message_send_success(send_result) {
                    return stats;
                }

                stats.messages_sent += 1;
                session_state.phase = Phase::ExchangeMessages;
            }
            Phase::ExchangeMessages => {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        send_websocket_disconnect_and_close(
                            session_state,
                            context,
                            &mut ws_sender,
                            config.shutdown_timeout
                        ).await;
                        return stats;
                    }

                    // receive message from server
                    maybe_received_message = ws_receiver.next() => {
                        let Some(received_message) = maybe_received_message else {
                            otel_info!(
                                "opamp.controller_extension.receive_message.server_disconnected",
                                message = "Server disconnected websocket while waiting for message"
                            );
                            return stats
                        };

                        let message = match received_message {
                            Ok(m) => m,
                            Err(e) => {
                                otel_error!(
                                    "opamp.controller_extension.receive_message.error",
                                    error =? e
                                );
                                return stats
                            }
                        };

                        stats.messages_received += 1;
                        let action = handle_received_websocket_message(
                            message,
                            session_state,
                            config,
                        );
                        let do_continue = process_handled_message_action(
                            action,
                            cancellation_token,
                            config,
                            context,
                            session_state,
                            &mut ws_sender,
                        ).await;

                        if !do_continue {
                            return stats
                        }
                    }

                    // periodically send heartbeat
                    _ = tokio::time::sleep(config.heartbeat_interval) => {
                        session_state.sequence_num += 1;
                         stats.messages_sent += 1;
                        let message = heartbeat_message(
                            session_state,
                            context,
                        );
                        let send_result = send_message(
                            &message,
                            cancellation_token,
                            &mut ws_sender
                        ).await;
                        if !is_message_send_success(send_result) {
                            return stats;
                        }
                        session_state.last_sent_message = Some(message);
                    }
                }
            }
        }
    }
}

/// Try to send message to server
///
/// Returns `None` if cancellation token is cancelled before request completes,
/// otherwise returns the result of the attempt to send the message.
async fn send_message<T, E>(
    message: &AgentToServer,
    cancellation_token: &CancellationToken,
    ws_sender: &mut T,
) -> Option<Result<(), E>>
where
    T: SinkExt<Message, Error = E> + Unpin,
{
    // safety: encode will return an error if the buffer does not have sufficient capacity which
    // is not the case here because vec can always grow, so we are safe to expect
    let mut msg_bytes = Vec::with_capacity(message.encoded_len() + 1);
    prost::encoding::encode_varint(WS_MESSAGE_HEADER, &mut msg_bytes);
    message.encode(&mut msg_bytes).expect("buffer has capacity");

    cancellation_token
        .run_until_cancelled(ws_sender.send(Message::Binary(Bytes::from(msg_bytes))))
        .await
}

fn is_message_send_success<E: std::fmt::Debug>(
    maybe_message_send_result: Option<Result<(), E>>,
) -> bool {
    match maybe_message_send_result {
        Some(message_send_result) => {
            if let Err(e) = message_send_result {
                otel_error!(
                    "opamp.controller_extension.send_message.error",
                    message = "Error sending AgentToServer message",
                    error =? e
                );

                false
            } else {
                true
            }
        }

        // cancelled before send completed
        None => false,
    }
}

/// Attempts send message to the server with the agent_disconnect flag set, then sends a websocket
/// close frame. This is the shutdown behaviour in the OpAMP spec. The deadline argument is how
/// long the operation has to complete before it will be abandoned, so that shutdown is not blocked
/// indefinitely by slow network and/or server
async fn send_websocket_disconnect_and_close<T, E>(
    session_state: &SessionState,
    context: &ControllerExtensionContext,
    ws_sender: &mut T,
    deadline: Duration,
) where
    T: SinkExt<Message, Error = E> + Unpin,
    E: std::fmt::Debug,
{
    let cancellation_token = CancellationToken::new();
    tokio::select! {
        _ = send_disconnect_message_and_close_frame(
            session_state,
            context,
            cancellation_token.clone(),
            ws_sender,
        ) => {
            // success
        }

        _ = tokio::time::sleep(deadline) => {
            cancellation_token.cancel();
        }
    };
}

async fn send_disconnect_message_and_close_frame<T, E>(
    session_state: &SessionState,
    context: &ControllerExtensionContext,
    shutdown_cancellation_token: CancellationToken,
    ws_sender: &mut T,
) where
    T: SinkExt<Message, Error = E> + Unpin,
    E: std::fmt::Debug,
{
    // end the final AgentToServer message with agent_disconnect set
    let mut last_message = heartbeat_message(session_state, context);
    last_message.agent_disconnect = Some(AgentDisconnect::default());
    match send_message(&last_message, &shutdown_cancellation_token, ws_sender).await {
        Some(Err(e)) => {
            otel_error!(
                "opamp.controller_extension.shutdown_message.error",
                message = "Error sending final AgentToServer message frame",
                error =? e
            );

            return;
        }
        None => {
            // timed out
            otel_warn!(
                "opamp.controller_extension.shutdown_message.timeout",
                message = "Unable to send final AgentToServer message before shutdown timeout"
            );
            return;
        }
        _ => {
            // success
        }
    }

    // send the final WebSocket Close frame
    let shutdown_send_result = shutdown_cancellation_token
        .run_until_cancelled(ws_sender.send(Message::Close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: "Agent shutting down".into(),
        }))))
        .await;

    match shutdown_send_result {
        Some(Err(e)) => {
            otel_error!(
                "opamp.controller_extension.shutdown_close_ws.error",
                message = "Error sending close frame",
                error =? e
            );
        }
        None => {
            otel_warn!(
                "opamp.controller_extension.shutdown_close_ws.timeout",
                message = "Unable to send WebSocket Close message before timeout"
            )
        }
        _ => {
            // success
        }
    }
}

/// Decide what must be done with a websocket message received from OpAMP server
fn handle_received_websocket_message(
    message: Message,
    session_state: &SessionState,
    config: &Config,
) -> HandledMessageAction {
    match message {
        Message::Binary(mut message_bytes) => {
            // Per the OpAMP spec, each WebSocket message starts with a varint-encoded header
            // (currently always zero) followed by the protobuf message bytes.
            // See https://opentelemetry.io/docs/specs/opamp/#websocket-message-format
            match prost::encoding::decode_varint(&mut message_bytes) {
                Ok(WS_MESSAGE_HEADER) => {}
                Ok(header) => {
                    otel_error!(
                        "opamp.controller_extension.decode.error",
                        message = "Unexpected non-zero header in websocket message",
                        header = header,
                    );
                    return HandledMessageAction::Ignore;
                }
                Err(e) => {
                    otel_error!(
                        "opamp.controller_extension.decode.error",
                        message = "Failed to decode websocket message header",
                        error =? e,
                    );
                    return HandledMessageAction::Ignore;
                }
            }

            match ServerToAgent::decode(message_bytes) {
                Ok(message) => handle_server_to_agent_message(message, session_state, config),
                Err(e) => {
                    otel_error!(
                        "opamp.controller_extension.decode.error",
                        message = "Failed to decode ServerToAgent protobuf message",
                        error =? e,
                    );
                    HandledMessageAction::Ignore
                }
            }
        }
        _ => HandledMessageAction::Ignore,
    }
}

/// Action to take in response to the received [`ServerToAgent`] message
///
/// TODO: more actions may be added in the future, such as `Restart` to handle server pushed
/// restart command
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum HandledMessageAction {
    /// Take no action, either because the message was an error,  or something invalid such as an
    /// instance_uid mismatch (indicating this instance of the agent was not target recipient),
    /// an unknown command type, etc.
    Ignore,

    /// The client should disconnect and retry the request due to an overloaded server. The server
    /// may optionally specify an amount of time to wait before retrying, otherwise an exponential
    /// backoff strategy must be used. For more details, see
    /// https://opentelemetry.io/docs/specs/opamp/#throttling
    DisconnectAndRetry(Option<u64>),

    /// Update the configuration of the engine and/or the state of the client, and send a reply to
    /// the server with the updated status and whatever else it has requested via flags.
    UpdateAndReply {
        updates: ServerUpdates,
        reply: ReplyOptions,
    },
}

/// Updates that the agent should apply, either to the engine it is managing or its own state
#[derive(Debug, Default)]
struct ServerUpdates {
    /// Update engine config
    engine_config: Option<EngineConfigUpdate>,

    /// Update agent identification
    agent_identification: Option<AgentIdentification>,

    /// Update connection settings between agent and server
    connection_settings: Option<OpAmpConnectionSettings>,
}

/// Server initiated update to the engine configuration
#[derive(Debug)]
struct EngineConfigUpdate {
    /// Engine configuration to be applied
    engine_config: OtelDataflowSpec,

    /// Server's Hash of the configuration
    config_hash: Vec<u8>,
}

/// Options determining what to include in the reply to the server. These may be controlled by the
/// [`ServerToAgent`] message's flags
#[derive(Debug, Default)]
struct ReplyOptions {
    /// Whether the server has requested that the full state be supplied. This may be requested by
    /// the server in the event of a missed message / unexpected sequence_num
    report_full_state: bool,

    /// Whether to send the full list of available components.
    report_available_components: bool,

    /// Whether to reply to the server that there was an error applying the configuration it sent.
    /// This could be the case where, for example, the configuration was not deserializable.
    reply_error: Option<ReplyError>,
}

#[derive(Debug, Default)]
struct ReplyError {
    message: String,

    config_hash: Vec<u8>,
}

/// Decide what must be done with the ServerToAgent message
fn handle_server_to_agent_message(
    message: ServerToAgent,
    session_state: &SessionState,
    config: &Config,
) -> HandledMessageAction {
    // ensure uid matches. An empty instance_uid is accepted: the field exists to
    // disambiguate multiplexed connections, and servers (e.g. the opamp-go reference
    // server) may leave it unset on server-initiated pushes over a dedicated connection.
    if !message.instance_uid.is_empty() && message.instance_uid != session_state.instance_uid {
        otel_warn!(
            "opamp.controller_extension.message.ignored",
            message = "Ignoring ServerToAgent message due to instance_uid mismatch",
            expected =? session_state.instance_uid,
            received =? message.instance_uid
        );
        return HandledMessageAction::Ignore;
    }

    // if error message is set, we handle it and can ignore the other fields. According to the
    // OpAMP spec, if this field is set, then all other fields MUST be unset so we assume this is
    // the case
    if let Some(error_response) = message.error_response {
        let error_type = ServerErrorResponseType::try_from(error_response.r#type).ok();
        let (should_retry, retry_after_ns) = match error_type {
            Some(ServerErrorResponseType::BadRequest | ServerErrorResponseType::Unknown) | None => {
                (false, None)
            }

            // Only retry on Unavailable errors.
            // For more details, see
            // https://opentelemetry.io/docs/specs/opamp/#throttling
            //https://opentelemetry.io/docs/specs/opamp/#servererrorresponsetype
            Some(ServerErrorResponseType::Unavailable) => match error_response.details {
                Some(Details::RetryInfo(retry_info)) => {
                    (true, Some(retry_info.retry_after_nanoseconds))
                }
                None => (true, None),
            },
        };

        otel_error!(
            "opamp.controller_extension.message.error",
            message = "ServerToAgent message contained error response",
            type =? error_type,
            error = error_response.error_message,
            will_retry = should_retry,
        );

        return if should_retry {
            HandledMessageAction::DisconnectAndRetry(retry_after_ns)
        } else {
            HandledMessageAction::Ignore
        };
    }

    // If the server has supplied a command, as per the OpAMP spec, the agent should execute the
    // command and ignore all other fields
    if let Some(command) = message.command {
        return match CommandType::try_from(command.r#type).ok() {
            Some(CommandType::Restart) => {
                // TODO - handle this differently once Restart command supported
                otel_warn!(
                    "opamp.controller_extension.message.command_ignored",
                    message = "ServerToAgent message contained command which is not yet supported.",
                    command =? "Restart"
                );
                HandledMessageAction::Ignore
            }
            None => {
                otel_info!(
                    "opamp.controller_extension.message.command",
                    message = "ServerToAgent message contained command",
                    command_type =? command.r#type
                );
                HandledMessageAction::Ignore
            }
        };
    }

    // below, we prepare any state updates and determine what must be sent to the server in
    // response to its message.

    let mut updates = ServerUpdates::default();
    let mut reply = ReplyOptions::default();

    if let Some(connection_settings) = message.connection_settings {
        if let Some(opamp_connection_settings) = connection_settings.opamp {
            updates.connection_settings = Some(opamp_connection_settings);
        }

        // since the DFE has its own internal telemetry pipeline which is supplied by configuration
        // (and it may not always export via OTLP, or some other networked exporter), we the own-
        // telemetry connection settings are simply ignored. Any connection settings for this type
        // of telemetry export should be supplied to the agent via engine config.
        if connection_settings.own_logs.is_some() {
            otel_debug!(
                "opamp.controller_extension.message.ignored_connection_setting",
                message = "ServerToAgent message value connection_setting.own_logs will be ignored"
            )
        }
        if connection_settings.own_metrics.is_some() {
            otel_debug!(
                "opamp.controller_extension.message.ignored_connection_setting",
                message =
                    "ServerToAgent message value connection_setting.own_metrics will be ignored"
            )
        }
        if connection_settings.own_traces.is_some() {
            otel_debug!(
                "opamp.controller_extension.message.ignored_connection_setting",
                message =
                    "ServerToAgent message value connection_setting.own_traces will be ignored"
            )
        }
        if !connection_settings.other_connections.is_empty() {
            otel_debug!(
                "opamp.controller_extension.message.ignored_connection_setting",
                message = "ServerToAgent message value connection_setting.other_connections will be ignored"
            )
        }
    }

    if message.flags != ServerToAgentFlags::Unspecified as u64 {
        if message.flags & ServerToAgentFlags::ReportFullState as u64 > 0 {
            reply.report_full_state = true;
        }

        if message.flags & ServerToAgentFlags::ReportAvailableComponents as u64 > 0 {
            reply.report_available_components = true
        }
    }

    if let Some(agent_identification) = message.agent_identification {
        updates.agent_identification = Some(agent_identification)
    }

    if let Some(remote_config) = message.remote_config {
        if let Some(remote_config_config) = remote_config.config {
            if Some(&remote_config.config_hash) != session_state.last_config_hash.as_ref() {
                if let Some(config_file) = remote_config_config
                    .config_map
                    .get(&config.remote_config_key)
                {
                    // assume empty content type to be JSON
                    if config_file.content_type == "application/json"
                        || config_file.content_type.is_empty()
                    {
                        match serde_json::from_slice::<OtelDataflowSpec>(&config_file.body) {
                            Ok(engine_config) => {
                                updates.engine_config = Some(EngineConfigUpdate {
                                    engine_config,
                                    config_hash: remote_config.config_hash,
                                })
                            }
                            Err(e) => {
                                let message =
                                    "Could not deserialize JSON encoded engine config".to_string();
                                otel_error!(
                                    "opamp.controller_extension.message.invalid_config_json",
                                    message = message,
                                    error =? e,
                                );
                                reply.reply_error = Some(ReplyError {
                                    message,
                                    config_hash: remote_config.config_hash,
                                })
                            }
                        }
                    } else {
                        let message = "Invalid content type. expected application/json".to_string();
                        otel_error!(
                            "opamp.controller_extension.message.invalid_serialized_config",
                            message = message,
                            received = config_file.content_type,
                        );
                        reply.reply_error = Some(ReplyError {
                            message,
                            config_hash: remote_config.config_hash,
                        })
                    }
                } else {
                    otel_warn!(
                        "opamp.controller_extension.message.missing_config_key",
                        message = "No key config key found in remote_config.config.config_map",
                        expected_key = config.remote_config_key
                    );
                }
            } else {
                otel_debug!(
                    "opamp.controller_extension.message.skip_update_config",
                    message = "Skipping config update because config hash matches last applied config hash"
                )
            }

            // emit logs about the ignored config
            if remote_config_config.config_map.len() > 1 {
                for key in remote_config_config.config_map.keys() {
                    if key == &config.remote_config_key {
                        continue;
                    }

                    otel_debug!(
                        "opamp.controller.message.ignored_config",
                        message = "Ignoring unexpected key found in remote.config.config_map",
                        key = key
                    )
                }
            }
        }
    }

    HandledMessageAction::UpdateAndReply { updates, reply }
}

/// Processes the handled message. Returns `true` if the request/response flow should continue,
/// or false if the client should disconnect/reconnect
async fn process_handled_message_action<T, E>(
    handled_action: HandledMessageAction,
    cancellation_token: &CancellationToken,
    config: &Config,
    context: &ControllerExtensionContext,
    session_state: &mut SessionState,
    ws_sender: &mut T,
) -> bool
where
    T: SinkExt<Message, Error = E> + Unpin,
    E: std::fmt::Debug,
{
    match handled_action {
        HandledMessageAction::Ignore => {
            session_state.phase = Phase::ExchangeMessages;
            session_state.retry_backoff_ns = None;
            true
        }
        HandledMessageAction::DisconnectAndRetry(maybe_backoff_ns) => {
            // returning will cause the websocket to disconnect, then after reconnect
            // we will reenter this loop to resend the message
            session_state.phase = Phase::RetryBackoff;
            if maybe_backoff_ns.is_some() {
                session_state.retry_backoff_ns = maybe_backoff_ns;
            } else {
                let mut backoff = config.request_retry.clone();
                if let Some(last_backoff) = session_state.retry_backoff_ns {
                    backoff.set_current(Duration::from_nanos(last_backoff));
                }
                session_state.retry_backoff_ns = Some(backoff.next_delay().as_nanos() as u64);
            }

            false
        }

        HandledMessageAction::UpdateAndReply { updates, reply } => {
            session_state.retry_backoff_ns = None;

            // update agent identification
            if let Some(agent_identification) = updates.agent_identification {
                if agent_identification.new_instance_uid.len() == 16 {
                    session_state.instance_uid = agent_identification
                        .new_instance_uid
                        .try_into()
                        // safety: we've checked the length
                        .expect("len == 16");
                } else {
                    otel_error!(
                        "opamp.controller_extension.new_instance_id.invalid",
                        message = "Will not update instance_id. Supplied instance_id has invalid length (expected len = 16)",
                        actual_len = agent_identification.new_instance_uid.len(),
                        new_instance_id =? &agent_identification.new_instance_uid
                    )
                }
            }

            // update connection settings
            if let Some(_connection_settings) = updates.connection_settings {
                // TODO support this eventually
                otel_warn!(
                    "opamp.controller_extension.connection_settings.update",
                    message = "Ignoring update to opamp connection settings. Not yet supported"
                )
            }

            // update engine config & report results to server
            if let Some(engine_config) = updates.engine_config {
                // Send message to let the server we're applying the config
                session_state.sequence_num += 1;
                let message = applying_message(
                    session_state,
                    &engine_config.config_hash,
                    &reply,
                    config,
                    context,
                );
                let send_result = send_message(&message, cancellation_token, ws_sender).await;
                if !is_message_send_success(send_result) {
                    return false;
                }
                session_state.last_sent_message = Some(message);

                // time to update config
                let reconcile_result =
                    context
                        .control_plane
                        .reconcile_engine_config(EngineConfigReconcileRequest {
                            config: engine_config.engine_config,
                            step_timeout_secs: config.reconcile.step_timeout_secs,
                            drain_timeout_secs: config.reconcile.drain_timeout_secs,
                            delete_timeout_secs: config.reconcile.delete_timeout_secs,
                            delete_missing: config.reconcile.delete_missing,
                        });

                session_state.sequence_num += 1;
                let reconcile_result_message = applied_result_message(
                    &reconcile_result,
                    session_state,
                    &engine_config.config_hash,
                    context,
                );

                let send_result =
                    send_message(&reconcile_result_message, cancellation_token, ws_sender).await;
                if !is_message_send_success(send_result) {
                    return false;
                }
                session_state.last_sent_message = Some(reconcile_result_message);
                session_state.last_reconcile_result = Some(reconcile_result);
                session_state.last_config_hash = Some(engine_config.config_hash.clone());

                session_state.phase = Phase::ExchangeMessages
            } else if reply.report_full_state
                || reply.report_available_components
                || reply.reply_error.is_some()
            {
                // Server didn't send valid new config but we need to reply to it either with state
                // it requested, or possibly an error letting it know we couldn't process the
                // config it tried to send
                session_state.sequence_num += 1;
                let mut message = full_state_reply_message(session_state, config, context);

                if let Some(error) = reply.reply_error {
                    message.remote_config_status = Some(RemoteConfigStatus {
                        last_remote_config_hash: error.config_hash,
                        status: RemoteConfigStatuses::Failed as i32,
                        error_message: error.message,
                    })
                }

                let send_result = send_message(&message, cancellation_token, ws_sender).await;
                if !is_message_send_success(send_result) {
                    return false;
                }
                session_state.last_sent_message = Some(message);
            }

            true
        }
    }
}

/// Generate the initial [`AgentToServer`] message sent when connection to the server is initiated
fn initial_message(
    session_state: &SessionState,
    config: &Config,
    context: &ControllerExtensionContext,
) -> AgentToServer {
    let status_snapshot = context.observed_state.snapshot();
    AgentToServer {
        instance_uid: session_state.instance_uid.to_vec(),
        sequence_num: session_state.sequence_num,
        agent_description: agent_description(config),
        capabilities: capabilities(),
        health: Some(component_health(session_state, &status_snapshot)),
        effective_config: Some(effective_config(context)),
        flags: AgentToServerFlags::Unspecified as u64,
        custom_capabilities: Some(custom_capabilities()),
        remote_config_status: Some(RemoteConfigStatus {
            status: RemoteConfigStatuses::Unset as i32,
            ..Default::default()
        }),
        connection_settings_status: Some(ConnectionSettingsStatus {
            status: ConnectionSettingsStatuses::Unset as i32,
            ..Default::default()
        }),
        agent_disconnect: None,
        package_statuses: None,
        connection_settings_request: None,
        custom_message: pipeline_status_custom_message(&status_snapshot),
        available_components: None,
    }
}

/// Generate the reply to the server after it has sent a new config and we begin applying it.
fn applying_message(
    session_state: &SessionState,
    config_hash: &[u8],
    reply_to_server: &ReplyOptions,
    config: &Config,
    context: &ControllerExtensionContext,
) -> AgentToServer {
    let status_snapshot = context.observed_state.snapshot();
    let mut message = AgentToServer {
        instance_uid: session_state.instance_uid.to_vec(),
        sequence_num: session_state.sequence_num,
        capabilities: capabilities(),
        health: Some(component_health(session_state, &status_snapshot)),
        remote_config_status: Some(RemoteConfigStatus {
            last_remote_config_hash: config_hash.to_vec(),
            status: RemoteConfigStatuses::Applying as i32,
            error_message: "".into(),
        }),
        agent_disconnect: None,
        flags: AgentToServerFlags::Unspecified as u64,
        connection_settings_request: None,
        custom_message: pipeline_status_custom_message(&status_snapshot),

        // by OpAMP spec, the following fields SHOULD remain unset if not changed since the last
        // message. Since these currently don't change at runtime, we set them all to None.
        // however, we'll fill them in below if the server asked for them in its last message
        effective_config: None,
        agent_description: None,
        available_components: None,
        custom_capabilities: None,

        ..Default::default()
    };

    if reply_to_server.report_full_state {
        message.effective_config = Some(effective_config(context));
        message.agent_description = agent_description(config);
        message.custom_capabilities = Some(custom_capabilities());

        // TODO - when we support updating connection statuses, need to fill this in for real
        message.connection_settings_status = Some(ConnectionSettingsStatus {
            status: ConnectionSettingsStatuses::Unset as i32,
            ..Default::default()
        })
    }

    if reply_to_server.report_available_components {
        // TODO when we support the agent reporting its components, fill these in here.
    }

    message
}

/// Generate a reply to the server after we have attempted to reconcile the config it has sent us
/// reporting state from the result of the reconciliation
fn applied_result_message(
    result: &Result<EngineConfigReconcileStatus, ControlPlaneError>,
    session_state: &SessionState,
    config_hash: &[u8],
    context: &ControllerExtensionContext,
) -> AgentToServer {
    let mut remote_config_status = RemoteConfigStatus {
        last_remote_config_hash: config_hash.to_vec(),
        ..Default::default()
    };
    set_remote_config_status_from_reconcile_result(&mut remote_config_status, Some(result));
    let status_snapshot = context.observed_state.snapshot();
    AgentToServer {
        instance_uid: session_state.instance_uid.to_vec(),
        sequence_num: session_state.sequence_num,
        flags: AgentToServerFlags::Unspecified as u64,
        capabilities: capabilities(),
        health: Some(component_health(session_state, &status_snapshot)),
        effective_config: Some(effective_config(context)),
        remote_config_status: Some(remote_config_status),
        custom_message: pipeline_status_custom_message(&status_snapshot),
        ..Default::default()
    }
}

/// Derive [`AgentDescription`] from config for [`AgentToServer::agent_description`] field
fn agent_description(config: &Config) -> Option<AgentDescription> {
    config
        .agent_description
        .as_ref()
        .map(|agent_description_config| AgentDescription {
            identifying_attributes: agent_description_config
                .identifying_attributes
                .as_ref()
                .map(|attrs| {
                    attrs
                        .iter()
                        .map(|(k, v)| KeyValue {
                            key: k.clone(),
                            value: Some(v.into()),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            non_identifying_attributes: agent_description_config
                .non_identifying_attributes
                .as_ref()
                .map(|attrs| {
                    attrs
                        .iter()
                        .map(|(k, v)| KeyValue {
                            key: k.clone(),
                            value: Some(v.into()),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        })
}

/// Generate a full-state reply when the server requests it via `ReportFullState` flag
/// without sending a config update.
fn full_state_reply_message(
    session_state: &SessionState,
    config: &Config,
    context: &ControllerExtensionContext,
) -> AgentToServer {
    let status_snapshot = context.observed_state.snapshot();
    let mut remote_config_status = RemoteConfigStatus {
        last_remote_config_hash: session_state.last_config_hash.clone().unwrap_or_default(),
        ..Default::default()
    };
    set_remote_config_status_from_reconcile_result(
        &mut remote_config_status,
        session_state.last_reconcile_result.as_ref(),
    );
    AgentToServer {
        instance_uid: session_state.instance_uid.to_vec(),
        sequence_num: session_state.sequence_num,
        capabilities: capabilities(),
        health: Some(component_health(session_state, &status_snapshot)),
        effective_config: Some(effective_config(context)),
        agent_description: agent_description(config),
        custom_capabilities: Some(custom_capabilities()),
        remote_config_status: Some(remote_config_status),
        custom_message: pipeline_status_custom_message(&status_snapshot),
        connection_settings_status: Some(ConnectionSettingsStatus {
            status: ConnectionSettingsStatuses::Unset as i32,
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn heartbeat_message(
    session_state: &SessionState,
    context: &ControllerExtensionContext,
) -> AgentToServer {
    let status_snapshot = context.observed_state.snapshot();
    let mut remote_config_status = RemoteConfigStatus {
        last_remote_config_hash: session_state.last_config_hash.clone().unwrap_or_default(),
        ..Default::default()
    };
    set_remote_config_status_from_reconcile_result(
        &mut remote_config_status,
        session_state.last_reconcile_result.as_ref(),
    );
    AgentToServer {
        instance_uid: session_state.instance_uid.to_vec(),
        sequence_num: session_state.sequence_num,
        capabilities: capabilities(),
        health: Some(component_health(session_state, &status_snapshot)),
        custom_message: pipeline_status_custom_message(&status_snapshot),
        remote_config_status: Some(remote_config_status),
        ..Default::default()
    }
}

fn set_remote_config_status_from_reconcile_result(
    remote_config_status: &mut RemoteConfigStatus,
    reconcile_result: Option<&Result<EngineConfigReconcileStatus, ControlPlaneError>>,
) {
    match reconcile_result {
        Some(Ok(status)) => match status.state {
            EngineConfigReconcileState::Succeeded => {
                remote_config_status.status = RemoteConfigStatuses::Applied as i32;
            }
            EngineConfigReconcileState::Pending | EngineConfigReconcileState::Running => {
                remote_config_status.status = RemoteConfigStatuses::Applying as i32;
            }
            EngineConfigReconcileState::Failed => {
                remote_config_status.status = RemoteConfigStatuses::Failed as i32;
                remote_config_status.error_message =
                    status.failure_reason.clone().unwrap_or_default();
            }
        },
        Some(Err(e)) => {
            remote_config_status.status = RemoteConfigStatuses::Failed as i32;
            remote_config_status.error_message =
                format!("Reconciliation failed with control plane error: {e:?}");
        }
        None => remote_config_status.status = RemoteConfigStatuses::Unset as i32,
    }
}

/// Report supported capabilities for [`AgentToServer::capabilities`] field
fn capabilities() -> u64 {
    (AgentCapabilities::ReportsStatus as u64)
        | (AgentCapabilities::AcceptsRemoteConfig as u64)
        | (AgentCapabilities::ReportsEffectiveConfig as u64)
        | (AgentCapabilities::ReportsHealth as u64)
        | (AgentCapabilities::ReportsRemoteConfig as u64)
        | (AgentCapabilities::ReportsHeartbeat as u64)
}

/// Derive the health of each pipeline/group as a [`ComponentHealth`] object
fn component_health(
    session_state: &SessionState,
    status_snapshot: &HashMap<PipelineKey, PipelineStatus>,
) -> ComponentHealth {
    let mut group_health_map = HashMap::new();

    for (pipeline_key, pipeline_status) in status_snapshot {
        let group_id = pipeline_key.pipeline_group_id().to_string();
        let group_health = group_health_map
            .entry(group_id)
            .or_insert(ComponentHealth::default());

        let pipeline_health = pipeline_health(pipeline_status);
        _ = group_health
            .component_health_map
            .insert(pipeline_key.pipeline_id().to_string(), pipeline_health);
    }

    for group_health in group_health_map.values_mut() {
        set_health_and_status_from_components(group_health);
    }

    let mut agent_health = ComponentHealth {
        start_time_unix_nano: session_state.start_time_unix_nano,
        status_time_unix_nano: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("after epoch")
            .as_nanos() as u64,
        component_health_map: group_health_map,
        // we don't fill in the last error, but this can be determined by controller from the
        // pipeline status which we report as a custom message
        last_error: "".into(),
        ..Default::default()
    };
    set_health_and_status_from_components(&mut agent_health);

    agent_health
}

/// Derive the health of a pipeline as a [`ComponentHealth`] object`
fn pipeline_health(pipeline_status: &PipelineStatus) -> ComponentHealth {
    if pipeline_status.readiness() && pipeline_status.liveness() {
        ComponentHealth {
            healthy: true,
            status: health_status::RUNNING.into(),
            ..Default::default()
        }
    } else {
        let mut all_running = true;
        let mut all_stopped = true;
        let mut any_shutting_down = false;
        let mut any_pending = false;
        let mut all_failed = true;
        for status in pipeline_status.per_instance().values() {
            match status.phase() {
                PipelinePhase::Deleted | PipelinePhase::Stopped => {
                    all_running = false;
                    all_failed = false;
                }
                PipelinePhase::Deleting(_) | PipelinePhase::Draining => {
                    // shutting down
                    all_running = false;
                    all_stopped = false;
                    all_failed = false;
                    any_shutting_down = true;
                }
                PipelinePhase::Pending
                | PipelinePhase::Starting
                | PipelinePhase::Updating
                | PipelinePhase::RollingBack => {
                    any_pending = true;
                    all_running = false;
                    all_stopped = false;
                    all_failed = false;
                }

                PipelinePhase::Failed(_) | PipelinePhase::Rejected(_) => {
                    all_running = false;
                    all_stopped = false;
                }
                PipelinePhase::Running => {
                    all_stopped = false;
                    all_failed = false;
                }
            }
        }

        // If there are no instances, nothing has failed
        if pipeline_status.per_instance().is_empty() {
            all_failed = false;
        }

        let status = if all_running {
            health_status::RUNNING
        } else if all_stopped {
            health_status::STOPPED
        } else if any_shutting_down {
            health_status::STOPPING
        } else if any_pending {
            health_status::STARTING
        } else if all_failed {
            health_status::FAILED
        } else {
            health_status::DEGRADED
        }
        .into();

        ComponentHealth {
            healthy: false,
            status,
            ..Default::default()
        }
    }
}

/// Sets the `health` and `status` field on the passed [`ComponentHealth`] by inspecting
/// the health and status of its children.
fn set_health_and_status_from_components(group_health: &mut ComponentHealth) {
    let mut all_running = true;
    let mut all_stopped = true;
    let mut all_failed = true;
    let mut any_shutting_down = false;
    let mut healthy = true;

    for pipeline_health in group_health.component_health_map.values() {
        match pipeline_health.status.as_str() {
            health_status::RUNNING => {
                all_stopped = false;
                all_failed = false;
            }
            health_status::STOPPED => {
                all_running = false;
                all_failed = false;
                healthy = false;
            }
            health_status::STOPPING => {
                all_running = false;
                all_stopped = false;
                all_failed = false;
                any_shutting_down = true;
                healthy = false;
            }
            health_status::FAILED => {
                all_running = false;
                all_stopped = false;
                healthy = false;
            }
            health_status::STARTING => {
                all_running = false;
                all_stopped = false;
                all_failed = false;
                healthy = false;
            }
            _ => {
                all_running = false;
                all_stopped = false;
                all_failed = false;
                healthy = false;
            }
        }
    }

    // If there are no components, nothing has failed
    if group_health.component_health_map.is_empty() {
        all_failed = false;
    }

    let status = if all_running {
        health_status::RUNNING
    } else if all_stopped {
        health_status::STOPPED
    } else if any_shutting_down {
        health_status::STOPPING
    } else if all_failed {
        health_status::FAILED
    } else {
        health_status::DEGRADED
    }
    .into();

    group_health.healthy = healthy;
    group_health.status = status;
}

/// Serialize the current applied config as the effective config for the AgentToServer message
fn effective_config(context: &ControllerExtensionContext) -> EffectiveConfig {
    let config_result = context
        .control_plane
        .engine_config_snapshot()
        .map_err(|e| format!("Failed to get engine config snapshot: {e:?}"))
        .and_then(|config| {
            serde_json::to_vec(&config)
                .map_err(|e| format!("Failed to serialize engine config: {e}"))
        });

    match config_result {
        Ok(body) => {
            let config_file = AgentConfigFile {
                content_type: "application/json".into(),
                body,
            };

            let mut config_map = HashMap::new();
            _ = config_map.insert("state".into(), config_file);

            EffectiveConfig {
                config_map: Some(AgentConfigMap { config_map }),
            }
        }
        Err(e) => {
            otel_error!(
                "opamp.controller_extension.effective_config.error",
                message = "Could not build effective config",
                error = e.as_str(),
            );
            EffectiveConfig { config_map: None }
        }
    }
}

fn custom_capabilities() -> CustomCapabilities {
    CustomCapabilities {
        capabilities: vec![CUSTOM_CAPABILITY_STATUS.into()],
    }
}

/// Derive a custom status message that contains the full state of the pipeline serialized as JSON
/// The custom message type is "report".
///
/// Returns `None` if, for some unexpected reason, the config snapshot could not be converted to
/// the message.
fn pipeline_status_custom_message(
    config_snapshot: &HashMap<PipelineKey, PipelineStatus>,
) -> Option<CustomMessage> {
    let data = match serde_json::to_vec(config_snapshot) {
        Ok(data) => data,
        Err(e) => {
            otel_warn!(
                "opamp.controller_extension.status.generate_failed",
                message = "Error serializing status report",
                error =? e,
            );

            return None;
        }
    };

    Some(CustomMessage {
        capability: CUSTOM_CAPABILITY_STATUS.into(),
        r#type: "report".into(),
        data,
    })
}

#[cfg(test)]
mod test {
    use std::{borrow::Cow, sync::Arc};

    use otap_df_config::{
        extension::{ExtensionUrn, ExtensionUserConfig},
        observed_state::ObservedStateSettings,
    };
    use otap_df_state::store::ObservedStateStore;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    use crate::extension::opamp::proto::opamp::v1::{
        AgentRemoteConfig, AnyValue, RetryInfo, ServerErrorResponse, any_value::Value,
    };
    use crate::extension::opamp::test::util::{
        EXPECTED_INSTANCE_UID_BYTES, EXPECTED_INSTANCE_UID_STR, MockControlPlane,
        MockWebSocketServer, empty_engine_config, server_to_agent_with_config, test_config,
    };

    use super::*;

    mod util;

    /// Start a mock server, and the agent controller with the passed config. Messages will then
    /// be exchanged until the number of expected exchanges have occurred. Afterward, the requests
    /// sent by the OpAMP agent are returned.
    ///
    /// For the config, there is no need to set the endpoint -- it can be left as an empty string
    /// and this test will set it to the correct IP/port of the mock server. Any endpoint value
    /// that is actually passed will be overridden.
    async fn run_web_socket_test_with_config(
        mock_server_responses: Vec<Option<ServerToAgent>>,
        control_plane: Arc<MockControlPlane>,
        expected_exchanges: usize,
        mut config: Config,
    ) -> Vec<AgentToServer> {
        let port = portpicker::pick_unused_port().expect("free port");
        config.endpoint = format!("ws://127.0.0.1:{port}/v1/opamp");

        let cancellation_token = CancellationToken::new();
        let mut server = MockWebSocketServer::new(port, mock_server_responses);
        let server_state = server.state();
        let server_cancellation_token = cancellation_token.clone();

        let server_handle =
            tokio::spawn(async move { server.serve(server_cancellation_token).await });

        let telemetry_registry_handle = TelemetryRegistryHandle::default();
        let observed_state_store = ObservedStateStore::new(
            &ObservedStateSettings::default(),
            telemetry_registry_handle.clone(),
        );

        let extension_urn = ExtensionUrn::parse(CONTROL_EXTENSION_URN).unwrap();
        let context = ControllerExtensionContext {
            extension_id: Cow::Borrowed("opamp"),
            extension: Arc::new(ExtensionUserConfig::new(
                extension_urn,
                serde_json::to_value(&config).unwrap(),
            )),
            control_plane,
            observed_state: observed_state_store.handle(),
            telemetry_registry: telemetry_registry_handle,
            engine_config: empty_engine_config(),
        };

        let client_cancellation_token = cancellation_token.clone();
        let client_handle = tokio::spawn(async move {
            run_websocket_connect_loop(context, config, client_cancellation_token).await
        });

        server_state
            .wait_for_exchanges(expected_exchanges, Duration::from_secs(10))
            .await;
        cancellation_token.cancel();
        server_handle.await.unwrap();
        client_handle.await.unwrap().unwrap();

        server_state.take_requests().await
    }

    #[tokio::test]
    async fn test_config_update() {
        let control_plane = Arc::new(MockControlPlane::new(empty_engine_config()));

        let responses = vec![
            Some(server_to_agent_with_config(&test_config(), vec![5, 1, 4])),
            None,
            None,
        ];

        let config: Config = serde_json::from_value(serde_json::json!({
            "instance_uid": EXPECTED_INSTANCE_UID_STR,
            "endpoint": ""
        }))
        .unwrap();
        let requests = run_web_socket_test_with_config(responses, control_plane, 3, config).await;
        assert_eq!(requests.len(), 3);

        let uid = EXPECTED_INSTANCE_UID_BYTES.to_vec();
        let expected_capabilities = (AgentCapabilities::ReportsStatus as u64)
            | (AgentCapabilities::AcceptsRemoteConfig as u64)
            | (AgentCapabilities::ReportsEffectiveConfig as u64)
            | (AgentCapabilities::ReportsHealth as u64)
            | (AgentCapabilities::ReportsRemoteConfig as u64)
            | (AgentCapabilities::ReportsHeartbeat as u64);

        // Message 1: initial full-state
        let initial = &requests[0];
        assert_eq!(initial.instance_uid, uid);
        assert_eq!(initial.sequence_num, 0);
        assert!(initial.agent_description.is_none());
        assert!(initial.health.is_some());
        assert!(initial.effective_config.is_some());
        assert!(initial.custom_capabilities.is_some());
        assert_eq!(initial.capabilities, expected_capabilities);
        assert_eq!(
            initial.remote_config_status.as_ref().unwrap().status,
            RemoteConfigStatuses::Unset as i32
        );

        // Message 2: applying
        let applying = &requests[1];
        assert_eq!(applying.instance_uid, uid);
        assert_eq!(applying.sequence_num, 1);
        let applying_status = applying.remote_config_status.as_ref().unwrap();
        assert_eq!(
            applying_status.status,
            RemoteConfigStatuses::Applying as i32
        );
        assert_eq!(applying_status.last_remote_config_hash, vec![5, 1, 4]);

        // Message 3: applied (succeeded)
        let applied = &requests[2];
        assert_eq!(applied.instance_uid, uid);
        assert_eq!(applied.sequence_num, 2);
        let applied_status = applied.remote_config_status.as_ref().unwrap();
        assert_eq!(applied_status.status, RemoteConfigStatuses::Applied as i32);
        assert_eq!(applied_status.last_remote_config_hash, vec![5, 1, 4]);
        assert!(applied.effective_config.is_some());
    }

    #[tokio::test]
    async fn test_reports_agent_description_if_configured() {
        let control_plane = Arc::new(MockControlPlane::new(empty_engine_config()));

        let responses = vec![
            Some(server_to_agent_with_config(&test_config(), vec![5, 1, 4])),
            None,
            None,
        ];

        let config: Config = serde_json::from_value(serde_json::json!({
            "instance_uid": EXPECTED_INSTANCE_UID_STR,
            "endpoint": "",
            "agent_description": {
                "identifying_attributes": {
                    "attr1": "hello",
                    "attr2": 514,
                    "attr3": 418.0,
                    "attr4": true,
                },
                "non_identifying_attributes": {
                    "attr5": "world"
                }
            }
        }))
        .unwrap();
        let requests = run_web_socket_test_with_config(responses, control_plane, 3, config).await;
        assert_eq!(requests.len(), 3);

        // Message 1: initial full-state - should contain the agent description since is in config
        let initial = &requests[0];
        assert!(initial.agent_description.is_some());

        let agent_description = initial.agent_description.clone().unwrap();
        assert_eq!(
            agent_description,
            AgentDescription {
                identifying_attributes: vec![
                    KeyValue {
                        key: "attr1".into(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("hello".into())),
                        })
                    },
                    KeyValue {
                        key: "attr2".into(),
                        value: Some(AnyValue {
                            value: Some(Value::IntValue(514)),
                        })
                    },
                    KeyValue {
                        key: "attr3".into(),
                        value: Some(AnyValue {
                            value: Some(Value::DoubleValue(418.0))
                        })
                    },
                    KeyValue {
                        key: "attr4".into(),
                        value: Some(AnyValue {
                            value: Some(Value::BoolValue(true))
                        })
                    }
                ],
                non_identifying_attributes: vec![KeyValue {
                    key: "attr5".into(),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue("world".into())),
                    })
                }],
            }
        )
    }

    #[tokio::test]
    async fn test_config_update_failed() {
        let control_plane = Arc::new(MockControlPlane::new(empty_engine_config()));

        let mut reconcile_status = EngineConfigReconcileStatus::new(
            "test".into(),
            EngineConfigReconcileState::Failed,
            None,
            "2026-06-01T00:00:00Z".into(),
        );
        reconcile_status.failure_reason = Some("failed to apply config".into());
        control_plane.set_reconcile_result(Ok(reconcile_status));

        let responses = vec![
            Some(server_to_agent_with_config(&test_config(), vec![5, 1, 4])),
            None,
            None,
        ];

        let config: Config = serde_json::from_value(serde_json::json!({
            "instance_uid": EXPECTED_INSTANCE_UID_STR,
            "endpoint": ""
        }))
        .unwrap();
        let requests = run_web_socket_test_with_config(responses, control_plane, 3, config).await;
        assert_eq!(requests.len(), 3);

        let applied = &requests[2];
        let status = applied.remote_config_status.as_ref().unwrap();
        assert_eq!(status.status, RemoteConfigStatuses::Failed as i32);
        assert!(status.error_message.contains("failed to apply config"));
    }

    #[tokio::test]
    async fn test_config_update_error() {
        let control_plane = Arc::new(MockControlPlane::new(empty_engine_config()));
        control_plane.set_reconcile_result(Err(ControlPlaneError::Internal {
            message: "error happen".into(),
        }));

        let responses = vec![
            Some(server_to_agent_with_config(&test_config(), vec![5, 1, 4])),
            None,
            None,
        ];

        let config: Config = serde_json::from_value(serde_json::json!({
            "instance_uid": EXPECTED_INSTANCE_UID_STR,
            "endpoint": ""
        }))
        .unwrap();
        let requests = run_web_socket_test_with_config(responses, control_plane, 3, config).await;
        assert_eq!(requests.len(), 3);

        let applied = &requests[2];
        let status = applied.remote_config_status.as_ref().unwrap();
        assert_eq!(status.status, RemoteConfigStatuses::Failed as i32);
        assert!(status.error_message.contains("error happen"));
    }

    #[tokio::test]
    async fn test_unparsable_config() {
        let control_plane = Arc::new(MockControlPlane::new(empty_engine_config()));
        let responses = vec![
            Some(ServerToAgent {
                remote_config: Some(AgentRemoteConfig {
                    config_hash: vec![0, 1, 2],
                    config: Some(AgentConfigMap {
                        config_map: HashMap::from_iter([(
                            "".into(),
                            AgentConfigFile {
                                content_type: "application/json".into(),
                                body: "not valid json".to_string().encode_to_vec(),
                            },
                        )]),
                    }),
                }),
                instance_uid: EXPECTED_INSTANCE_UID_BYTES.to_vec(),
                ..Default::default()
            }),
            None,
        ];

        let config: Config = serde_json::from_value(serde_json::json!({
            "instance_uid": EXPECTED_INSTANCE_UID_STR,
            "endpoint": "",
        }))
        .unwrap();

        let requests = run_web_socket_test_with_config(responses, control_plane, 2, config).await;
        assert_eq!(requests.len(), 2);

        let applied = &requests[1];
        let status = applied.remote_config_status.as_ref().unwrap();
        assert_eq!(status.status, RemoteConfigStatuses::Failed as i32);
        assert!(status.error_message.contains("Could not deserialize JSON"));
    }

    #[tokio::test]
    async fn test_heartbeat_sent_after_applied_config() {
        let control_plane = Arc::new(MockControlPlane::new(empty_engine_config()));
        let responses = vec![
            Some(server_to_agent_with_config(&test_config(), vec![5, 1, 4])),
            None,
            None,
            None,
        ];

        let config: Config = serde_json::from_value(serde_json::json!({
            "instance_uid": EXPECTED_INSTANCE_UID_STR,
            "endpoint": "",
            "heartbeat_interval": "200ms"
        }))
        .unwrap();

        let requests = run_web_socket_test_with_config(responses, control_plane, 4, config).await;
        assert_eq!(requests.len(), 4);

        let heartbeat = &requests[3];
        assert_eq!(heartbeat.instance_uid, EXPECTED_INSTANCE_UID_BYTES.to_vec());
        assert!(heartbeat.sequence_num > 2);
        assert!(heartbeat.health.is_some());
        assert!(heartbeat.custom_message.is_some());
        let custom = heartbeat.custom_message.as_ref().unwrap();
        assert_eq!(custom.capability, CUSTOM_CAPABILITY_STATUS);
    }

    #[tokio::test]
    async fn test_server_unavailable_triggers_retry() {
        let control_plane = Arc::new(MockControlPlane::new(empty_engine_config()));
        let responses = vec![
            Some(ServerToAgent {
                error_response: Some(ServerErrorResponse {
                    r#type: ServerErrorResponseType::Unavailable as i32,
                    error_message: "please try again".into(),
                    details: Some(Details::RetryInfo(RetryInfo {
                        retry_after_nanoseconds: 1000,
                    })),
                }),
                instance_uid: EXPECTED_INSTANCE_UID_BYTES.to_vec(),
                ..Default::default()
            }),
            Some(server_to_agent_with_config(&test_config(), vec![5, 1, 4])),
            None,
            None,
        ];

        let config: Config = serde_json::from_value(serde_json::json!({
            "instance_uid": EXPECTED_INSTANCE_UID_STR,
            "endpoint": ""
        }))
        .unwrap();
        let requests = run_web_socket_test_with_config(responses, control_plane, 4, config).await;
        assert_eq!(requests.len(), 4);

        // assert the first request was retried - same message and same sequence number
        assert_eq!(requests[0].sequence_num, 0);
        assert_eq!(requests[1].sequence_num, 0);
        assert_eq!(requests[0], requests[1]);
    }

    #[tokio::test]
    async fn test_client_reconnects_after_server_drops() {
        let control_plane = Arc::new(MockControlPlane::new(empty_engine_config()));
        let responses = vec![];

        let config: Config = serde_json::from_value(serde_json::json!({
            "instance_uid": EXPECTED_INSTANCE_UID_STR,
            "endpoint": "",
            "connect_retry": {
                "initial": "50ms",
                "max": "200ms"
            }
        }))
        .unwrap();
        let requests = run_web_socket_test_with_config(responses, control_plane, 2, config).await;
        assert_eq!(requests.len(), 2);
    }

    #[tokio::test]
    async fn test_presents_consistent_view_of_last_applied_status() {
        let control_plane = Arc::new(MockControlPlane::new(empty_engine_config()));
        let mut reconcile_status = EngineConfigReconcileStatus::new(
            "test".into(),
            EngineConfigReconcileState::Failed,
            None,
            "2026-06-01T00:00:00Z".into(),
        );
        reconcile_status.failure_reason = Some("failed to apply config".into());
        control_plane.set_reconcile_result(Ok(reconcile_status));

        let responses = vec![
            Some(server_to_agent_with_config(&test_config(), vec![5, 1, 4])),
            None,
            None,
        ];

        let config: Config = serde_json::from_value(serde_json::json!({
            "instance_uid": EXPECTED_INSTANCE_UID_STR,
            "endpoint": "",
            "heartbeat_interval": "200ms"
        }))
        .unwrap();
        let requests = run_web_socket_test_with_config(responses, control_plane, 4, config).await;
        assert_eq!(requests.len(), 4);

        // expect that the 3rd message (the one w/ the applying status) has presented that the last
        // applied config has failed
        let applied_result_msg = &requests[2];
        let first_remote_config_status = applied_result_msg.remote_config_status.as_ref().unwrap();
        assert_eq!(
            first_remote_config_status.status,
            RemoteConfigStatuses::Failed as i32
        );

        // also expect that the heartbeat following this message has status failed
        let heartbeat = &requests[3];
        let heartbeat_remote_config_status = heartbeat.remote_config_status.as_ref().unwrap();
        assert_eq!(
            heartbeat_remote_config_status.status,
            RemoteConfigStatuses::Failed as i32
        );
    }
}
