// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Asynchronous OTLP exporter implementation.
//!
//! The exporter receives pipeline messages on a single-threaded Tokio runtime. Each payload is
//! encoded (when necessary) and handed off to a gRPC export RPC. We keep the gRPC futures in a
//! lightweight in-flight queue which enforces the configured concurrency limit. As soon as a
//! request finishes we forward the Ack/Nack to the pipeline runtime so the dataflow can make
//! progress.

otap_df_telemetry::otel_component_scope!(
    urn = OTLP_EXPORTER_URN,
    target = "otel.exporter.otlp_grpc",
);

use async_trait::async_trait;
use bytes::Bytes;
use futures::future::FutureExt;
use futures::stream::{FuturesUnordered, StreamExt};
use http::HeaderValue;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::{Error, ExporterErrorKind, format_error_sources};
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::capability::auth::bearer_token_provider::BearerTokenProvider;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::otap_grpc::client_settings::GrpcClientSettings;
use otap_df_otap::otap_grpc::otlp::client::{
    LogsServiceClient, MetricsServiceClient, TraceServiceClient,
};
use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_otap::transport_headers::ValueKind;
use otap_df_pdata::otlp::logs::LogsProtoBytesEncoder;
use otap_df_pdata::otlp::metrics::MetricsProtoBytesEncoder;
use otap_df_pdata::otlp::traces::TracesProtoBytesEncoder;
use otap_df_pdata::otlp::{ProtoBuffer, ProtoBytesEncoder};
use otap_df_pdata::{OtapArrowRecords, OtapPayload, OtapPayloadHelpers, OtlpProtoBytes};
use serde::Deserialize;
use std::collections::VecDeque;
use std::future::Future;
use std::sync::Arc;
use std::time::Instant;
use tonic::Code;
use tonic::codec::CompressionEncoding;
use tonic::metadata::{MetadataKey, MetadataMap, MetadataValue};
use tonic::transport::Channel;

use otap_df_otap::bearer_auth::{BearerAuth, BearerAuthEvents, apply_auth_rejection};

mod metrics;

use metrics::{OtlpGrpcExporterErrorType, OtlpGrpcExporterMetrics};

/// The URN for the OTLP gRPC exporter
pub const OTLP_EXPORTER_URN: &str = "urn:otel:exporter:otlp_grpc";

/// Raises the shared bearer-auth warnings under this exporter's event namespace.
const GRPC_BEARER_AUTH_EVENTS: BearerAuthEvents = BearerAuthEvents {
    invalid_token: |error| {
        otel_warn!("otlp.exporter.grpc.invalid_bearer_token", error = %error);
    },
    token_stream_closed: || {
        otel_warn!(
            "otlp.exporter.grpc.token_stream_closed",
            message = "bearer token provider closed its stream; \
                no further token refreshes will arrive"
        );
    },
};

/// Configuration for the OTLP Exporter
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Shared gRPC client settings reused across OTLP exports.
    #[serde(flatten)]
    pub grpc: GrpcClientSettings,
    /// Maximum number of concurrent in-flight export RPCs.
    #[serde(default = "default_max_in_flight")]
    pub max_in_flight: usize,
    /// Number of separate gRPC channels (TCP connections) to open to the
    /// endpoint. Multiple connections improve load distribution when the
    /// receiver uses `SO_REUSEPORT` across cores. Defaults to 1.
    #[serde(default = "default_num_connections")]
    pub num_connections: usize,
}

pub(crate) const fn default_max_in_flight() -> usize {
    5
}

pub(crate) const fn default_num_connections() -> usize {
    1
}

/// Exporter that sends OTLP data via gRPC
pub struct OTLPExporter {
    config: Config,
    metrics: OtlpGrpcExporterMetrics,
    /// Optional bearer token provider resolved from the
    /// `bearer_token_provider` capability. When bound, a fresh
    /// `authorization: Bearer <token>` is injected on every outgoing request;
    /// when absent, the exporter behaves exactly as before.
    token_provider: Option<Box<dyn BearerTokenProvider>>,
}

/// Declare the OTLP Exporter as a local exporter factory
#[allow(unsafe_code)]
#[otap_df_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static OTLP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: OTLP_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig,
             capabilities: &otap_df_engine::capability::registry::Capabilities| {
        // Optionally resolve a bound bearer token provider. Absent binding keeps
        // the default (no-auth) behavior; a bound provider (e.g. the
        // `oauth2_client_auth` extension) supplies refreshed OAuth tokens.
        let token_provider = capabilities
            .optional_local::<otap_df_engine::capability::auth::bearer_token_provider::BearerTokenProvider>()
            .map_err(|e| otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            })?;
        Ok(ExporterWrapper::local(
            OTLPExporter::from_config(pipeline, &node_config.config, token_provider)?,
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config,
};

/// Validates the OTLP gRPC exporter configuration at config load time.
///
/// Runs before any node is started (initial load and live reconfigure), so bad
/// configuration is rejected fast and attributed to the offending node rather
/// than surfacing as an opaque client error at startup.
fn validate_config(config: &serde_json::Value) -> Result<(), otap_df_config::error::Error> {
    let cfg: Config = serde_json::from_value(config.clone()).map_err(|e| {
        otap_df_config::error::Error::InvalidUserConfig {
            error: e.to_string(),
        }
    })?;
    cfg.grpc
        .validate()
        .map_err(|e| otap_df_config::error::Error::InvalidUserConfig {
            error: e.to_string(),
        })?;
    Ok(())
}

impl OTLPExporter {
    /// create a new instance of the `[OTLPExporter]` from json config value
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
        token_provider: Option<Box<dyn BearerTokenProvider>>,
    ) -> Result<Self, otap_df_config::error::Error> {
        let metrics = OtlpGrpcExporterMetrics::register(&pipeline_ctx);

        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        Ok(Self {
            config,
            metrics,
            token_provider,
        })
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for OTLPExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        otel_info!(
            "otlp.exporter.grpc.start",
            grpc_endpoint = self.config.grpc.grpc_endpoint.as_str()
        );

        self.config.grpc.log_proxy_info();

        let exporter_id = effect_handler.exporter_id();

        // Run the optional startup check (dns resolution or eager connect) before creating the
        // lazy channels used for normal runtime traffic.
        self.config.grpc.run_startup_check().await.map_err(|e| {
            let source_detail = format_error_sources(&e);
            Error::ExporterError {
                exporter: exporter_id.clone(),
                kind: ExporterErrorKind::Connect,
                error: format!("startup check failed: {e}"),
                source_detail,
            }
        })?;

        let num_connections = self.config.num_connections.max(1);
        let mut channels = Vec::with_capacity(num_connections);
        for _ in 0..num_connections {
            let channel = self
                .config
                .grpc
                .connect_channel_lazy(None)
                .await
                .map_err(|e| {
                    let source_detail = format_error_sources(&e);
                    Error::ExporterError {
                        exporter: exporter_id.clone(),
                        kind: ExporterErrorKind::Connect,
                        error: format!("grpc channel error {e}"),
                        source_detail,
                    }
                })?;
            channels.push(channel);
        }

        otel_info!(
            "otlp.exporter.grpc.channels",
            num_connections = num_connections,
            endpoint = self.config.grpc.grpc_endpoint.as_str()
        );

        let compression = self.config.grpc.compression_encoding();
        let max_in_flight = self.config.max_in_flight.max(1);

        // Pre-build the static gRPC metadata template ONCE, outside the hot loop.
        // Returns `None` when no static headers are configured, which preserves
        // the zero-allocation fast path in `build_grpc_metadata`.
        let static_metadata = self.config.grpc.build_static_metadata();

        // reuse the encoder and the buffer across pdatas
        let mut logs_proto_encoder = LogsProtoBytesEncoder::new();
        let mut metrics_proto_encoder = MetricsProtoBytesEncoder::new();
        let mut traces_proto_encoder = TracesProtoBytesEncoder::new();

        let mut logs_proto_buffer = ProtoBuffer::with_capacity(8 * 1024);
        let mut metrics_proto_buffer = ProtoBuffer::with_capacity(8 * 1024);
        let mut traces_proto_buffer = ProtoBuffer::with_capacity(8 * 1024);

        let mut grpc_clients = GrpcClientPool::new(max_in_flight, channels, compression);
        grpc_clients.prepopulate_clients();

        let mut inflight_exports = InFlightExports::new();
        let mut pending_msg: Option<(OtapPdata, Instant)> = None;

        // Consumer-side bearer-token adapter, if a provider is bound. It owns
        // the token subscription, the cached `authorization` header, and token
        // usability; the loop below stays auth-agnostic -- it only asks whether
        // it may send and stamps the header the adapter hands back.
        let mut auth = self
            .token_provider
            .take()
            .map(|provider| BearerAuth::new(provider, GRPC_BEARER_AUTH_EVENTS));

        // Timer that fires when the cached token crosses its usability margin.
        // Hoisted out of the loop and re-armed only when the deadline actually
        // moves (i.e. when a refresh is cached), so a busy exporter does not pay
        // a timer-wheel registration per message. It starts already elapsed and
        // is only polled once armed, since the `select!` arm below is guarded on
        // a deadline being present.
        let margin_sleep = tokio::time::sleep_until(tokio::time::Instant::now());
        tokio::pin!(margin_sleep);
        let mut armed_margin_deadline: Option<Instant> = None;

        // Main loop: 1) finish ready completions, 2) biased wait for a token
        // event, a completion, or the next message, 3) dispatch work while
        // respecting the in-flight budget.
        loop {
            // Backpressure guard: when full and a message is parked, only drain completions.
            if inflight_exports.len() >= max_in_flight && pending_msg.is_some() {
                if let Some(completed) = inflight_exports.next_completion().await {
                    let (client, rejected_generation) =
                        finalize_completed_export(completed, &effect_handler, &mut self.metrics)
                            .await;
                    apply_auth_rejection(&mut auth, rejected_generation);
                    grpc_clients.release(client);
                }
                continue;
            }

            // Opportunistically drain completions before we park on a recv.
            while let Some(completed) = inflight_exports.next_completion().now_or_never().flatten()
            {
                let (client, rejected_generation) =
                    finalize_completed_export(completed, &effect_handler, &mut self.metrics).await;
                apply_auth_rejection(&mut auth, rejected_generation);
                grpc_clients.release(client);
            }

            // Admit pdata only when auth is ready (a usable token is cached, or no
            // provider is bound). While a bound provider has no usable token we
            // stop pulling pdata, so it back-pressures upstream instead of being
            // accepted and NACK'd. A token is guaranteed to eventually arrive --
            // the extension's readiness probe holds data-path startup until the
            // first publish, and its watch stream stays live while we hold the
            // provider handle -- so waiting (not dropping) is always correct here.
            let accepting_pdata = auth.as_ref().is_none_or(BearerAuth::is_ready);

            // Instant at which a currently-usable token crosses the usability
            // margin. Used to wake the loop so `accepting_pdata` re-evaluates
            // (and gates) before a near-expiry batch is admitted, since the recv
            // arm below may already be parked when the margin is reached.
            let token_margin_deadline = auth.as_ref().and_then(BearerAuth::refresh_deadline);
            if token_margin_deadline != armed_margin_deadline {
                if let Some(deadline) = token_margin_deadline {
                    margin_sleep
                        .as_mut()
                        .reset(tokio::time::Instant::from_std(deadline));
                }
                armed_margin_deadline = token_margin_deadline;
            }

            // A batch parked for in-flight capacity is un-parked only once auth is
            // ready again. Taking it while `accepting_pdata` is false would NACK it
            // below -- exactly the outcome the gate exists to avoid -- so instead it
            // stays parked and the loop keeps servicing token refreshes and
            // completions until a usable token arrives. Shutdown is the escape
            // hatch: it force-drains, and its arm NACKs whatever is still parked.
            let parked_msg = if accepting_pdata {
                pending_msg.take()
            } else {
                None
            };

            // Prefer token events, then completions, then the next message.
            let mut parked_export_started_at = None;
            let msg = if let Some((pdata, export_started_at)) = parked_msg {
                parked_export_started_at = Some(export_started_at);
                Message::PData(pdata)
            } else {
                tokio::select! {
                    biased;

                    // Wake when the cached token reaches its usability margin so the
                    // next loop iteration gates intake. Guarded because the timer is
                    // left elapsed whenever nothing is armed; once it fires,
                    // `refresh_deadline` returns `None`, which closes the guard and
                    // keeps the arm from busy-looping.
                    () = &mut margin_sleep, if token_margin_deadline.is_some() => {
                        continue;
                    }

                    // Pick up token refreshes (initial + subsequent) even while pdata
                    // intake is gated, so a pending token can arrive and unblock us.
                    // The `async` block keeps this lazy: `select!` evaluates a branch
                    // expression even when its `if` guard is false, and `auth` is
                    // `None` when no provider is bound. The `None` arm is unreachable
                    // while the guard holds; it pends rather than panics.
                    () = async {
                        match auth.as_mut() {
                            Some(a) => a.poll_refresh().await,
                            None => std::future::pending().await,
                        }
                    }, if auth.as_ref().is_some_and(BearerAuth::is_active) => {
                        // A refresh was drained (the adapter caches it and logs any
                        // anomaly); loop to re-evaluate intake readiness.
                        continue;
                    }

                    // Drain a finished export. Guarded because an empty in-flight set
                    // is immediately ready (`FuturesUnordered::next` yields `None`),
                    // which would otherwise busy-loop.
                    completed = inflight_exports.next_completion(), if !inflight_exports.is_empty() => {
                        if let Some(completed) = completed {
                            let (client, rejected_generation) = finalize_completed_export(
                                completed,
                                &effect_handler,
                                &mut self.metrics,
                            )
                            .await;
                            // Server rejected the token this request used
                            // (UNAUTHENTICATED); drop exactly that generation so intake
                            // back-pressures until `token_stream` delivers a fresh one,
                            // and the retry never reuses the rejected token. A stale
                            // rejection (a newer token was already cached) is ignored by
                            // the generation guard.
                            apply_auth_rejection(&mut auth, rejected_generation);
                            grpc_clients.release(client);
                        }
                        continue;
                    }

                    // Inbound message. Control always flows; pdata is admitted only
                    // when `accepting_pdata` (and force-drained during shutdown).
                    msg = msg_chan.recv_when(accepting_pdata) => {
                        let msg = msg?;
                        otel_debug!("otlp.exporter.grpc.receive");
                        msg
                    }
                }
            };

            match msg {
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    otel_info!("otlp.exporter.grpc.shutdown");
                    // A batch can only still be parked here because the auth gate
                    // was shut when it last came up for dispatch: un-parking is
                    // gated on `accepting_pdata`, and every other `select!` arm
                    // loops rather than falling through to this match, so reaching
                    // Shutdown with a parked batch implies a provider is bound and
                    // its token is still unusable. Shutdown cannot wait for a
                    // refresh, so NACK it as retryable -- the same policy the
                    // force-drained batches get below. Without this the parked batch
                    // would be dropped silently.
                    if let Some((pdata, export_started_at)) = pending_msg.take() {
                        debug_assert!(
                            auth.as_ref().is_some_and(|a| !a.is_ready()),
                            "a batch stays parked only while a bound token is unusable"
                        );
                        let reason = auth
                            .as_ref()
                            .map_or("no usable bearer token", BearerAuth::not_ready_reason);
                        nack_without_usable_token(
                            pdata,
                            reason,
                            export_started_at,
                            &effect_handler,
                            &mut self.metrics,
                        )
                        .await;
                    }
                    while !inflight_exports.is_empty() {
                        if let Some(completed) = inflight_exports.next_completion().await {
                            let (client, rejected_generation) = finalize_completed_export(
                                completed,
                                &effect_handler,
                                &mut self.metrics,
                            )
                            .await;
                            // Honor a rejection even while draining, so a later
                            // force-drained request cannot reuse the rejected token.
                            apply_auth_rejection(&mut auth, rejected_generation);
                            grpc_clients.release(client);
                        }
                    }
                    return Ok(TerminalState::new(
                        deadline,
                        self.metrics.terminal_snapshots(),
                    ));
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = self.metrics.report(&mut metrics_reporter);
                }
                Message::PData(pdata) => {
                    let export_started_at = parked_export_started_at.unwrap_or_else(Instant::now);
                    if inflight_exports.len() >= max_in_flight {
                        // The guard at the top of the loop stops receiving while a
                        // batch is parked and the budget is full, so parking here
                        // can never overwrite (and lose) an earlier batch.
                        debug_assert!(
                            pending_msg.is_none(),
                            "a parked batch must be dispatched before another is parked"
                        );
                        pending_msg = Some((pdata, export_started_at));
                        continue;
                    }

                    // We only reach here with a usable token: intake is gated on
                    // `accepting_pdata`, and a parked batch is un-parked only while
                    // that gate is open. The exception is shutdown, which
                    // force-drains buffered pdata even while auth is pending: with no
                    // usable token we cannot send, so NACK it as retryable -- a token
                    // may yet arrive, so nothing is dropped.
                    if let Some(a) = auth.as_ref() {
                        if !a.is_ready() {
                            let reason = a.not_ready_reason();
                            nack_without_usable_token(
                                pdata,
                                reason,
                                export_started_at,
                                &effect_handler,
                                &mut self.metrics,
                            )
                            .await;
                            continue;
                        }
                    }

                    let signal_type = pdata.signal_type();
                    let (context, payload) = pdata.into_parts();

                    // The cached bearer header, together with the generation of the
                    // token it was built from. The generation is echoed back on
                    // completion so an UNAUTHENTICATED response can be matched to the
                    // exact token used and a stale rejection ignored.
                    let (auth_header, token_generation) =
                        match auth.as_ref().and_then(BearerAuth::header) {
                            Some((header, generation)) => (Some(header), Some(generation)),
                            None => (None, None),
                        };

                    // Build gRPC metadata from configured static headers, any
                    // propagated transport headers, and the refreshed bearer
                    // token. Computed once before signal dispatch; the static
                    // template is cloned only when present so the no-metadata
                    // case stays allocation-free.
                    let metadata = RequestMetadata {
                        metadata: build_grpc_metadata(
                            &effect_handler,
                            &context,
                            static_metadata.as_ref(),
                            auth_header,
                        ),
                        token_generation,
                    };

                    // Dispatch based on signal type and the concrete payload representation.
                    match (signal_type, payload) {
                        (SignalType::Logs, OtapPayload::OtapArrowRecords(otap_batch)) => {
                            dispatch_otap_export(
                                otap_batch,
                                context,
                                metadata,
                                SignalType::Logs,
                                export_started_at,
                                &exporter_id,
                                &mut logs_proto_buffer,
                                &mut logs_proto_encoder,
                                |encoded| {
                                    let client = SignalClient::Logs(grpc_clients.take_logs());
                                    make_export_future(encoded, client)
                                },
                                &mut inflight_exports,
                                &mut self.metrics,
                                &effect_handler,
                            )
                            .await;
                        }
                        (SignalType::Metrics, OtapPayload::OtapArrowRecords(otap_batch)) => {
                            dispatch_otap_export(
                                otap_batch,
                                context,
                                metadata,
                                SignalType::Metrics,
                                export_started_at,
                                &exporter_id,
                                &mut metrics_proto_buffer,
                                &mut metrics_proto_encoder,
                                |encoded| {
                                    let client = SignalClient::Metrics(grpc_clients.take_metrics());
                                    make_export_future(encoded, client)
                                },
                                &mut inflight_exports,
                                &mut self.metrics,
                                &effect_handler,
                            )
                            .await;
                        }
                        (SignalType::Traces, OtapPayload::OtapArrowRecords(otap_batch)) => {
                            dispatch_otap_export(
                                otap_batch,
                                context,
                                metadata,
                                SignalType::Traces,
                                export_started_at,
                                &exporter_id,
                                &mut traces_proto_buffer,
                                &mut traces_proto_encoder,
                                |encoded| {
                                    let client = SignalClient::Traces(grpc_clients.take_traces());
                                    make_export_future(encoded, client)
                                },
                                &mut inflight_exports,
                                &mut self.metrics,
                                &effect_handler,
                            )
                            .await;
                        }
                        (_, OtapPayload::OtlpBytes(service_req)) => {
                            let prepared = match service_req {
                                OtlpProtoBytes::ExportLogsRequest(bytes) => prepare_otlp_export(
                                    bytes,
                                    context,
                                    metadata,
                                    SignalType::Logs,
                                    export_started_at,
                                    |b| OtlpProtoBytes::ExportLogsRequest(b).into(),
                                ),
                                OtlpProtoBytes::ExportMetricsRequest(bytes) => prepare_otlp_export(
                                    bytes,
                                    context,
                                    metadata,
                                    SignalType::Metrics,
                                    export_started_at,
                                    |b| OtlpProtoBytes::ExportMetricsRequest(b).into(),
                                ),
                                OtlpProtoBytes::ExportTracesRequest(bytes) => prepare_otlp_export(
                                    bytes,
                                    context,
                                    metadata,
                                    SignalType::Traces,
                                    export_started_at,
                                    |b| OtlpProtoBytes::ExportTracesRequest(b).into(),
                                ),
                            };

                            let client = match signal_type {
                                SignalType::Logs => SignalClient::Logs(grpc_clients.take_logs()),
                                SignalType::Metrics => {
                                    SignalClient::Metrics(grpc_clients.take_metrics())
                                }
                                SignalType::Traces => {
                                    SignalClient::Traces(grpc_clients.take_traces())
                                }
                            };
                            let future = make_export_future(prepared, client);
                            inflight_exports.push(future);
                        }
                    }
                }
                _ => {
                    // ignore unhandled messages
                }
            }
        }
    }
}

/// Helper function to handle export result and send Ack/Nack accordingly.
///
/// `auth_failure` marks a rejection of the bearer token this request carried; it
/// forces the NACK to be retryable even though `UNAUTHENTICATED` is otherwise a
/// permanent status.
///
/// This returns Ok(()) if the result Ack/Nack was successfully routed regardless of
/// whether the request actually succeeded. E.g. it does not return `Err` for an
/// unsuccessful request.
async fn route_export_result<T>(
    result: &Result<T, tonic::Status>,
    context: Context,
    saved_payload: OtapPayload,
    effect_handler: &EffectHandler<OtapPdata>,
    auth_failure: bool,
) -> Result<(), Error> {
    match result {
        Ok(_) => {
            effect_handler
                .notify_ack(AckMsg::new(OtapPdata::new(context, saved_payload)))
                .await?;
        }
        Err(e) => {
            let retryable = is_retryable_grpc_status(e) || auth_failure;
            let error_msg = e.to_string();

            // TODO(https://github.com/open-telemetry/otel-arrow/issues/3404):
            // NackMsg has no structured retry-after field yet, so we fold the
            // server's advisory RetryInfo delay into the human-readable reason.
            // Replace this with a structured field once #3404 lands.
            let mut reason = error_msg.clone();
            if let Some(delay) = retry_after(e) {
                reason.push_str(&format!(" (retry after {})", format_retry_delay(&delay)));
            }

            let mut nack = NackMsg::new(&reason, OtapPdata::new(context, saved_payload));
            nack.permanent = !retryable;
            effect_handler.notify_nack(nack).await?;
        }
    }

    Ok(())
}

/// Prost-generated struct for `google.rpc.Status`.
///
/// See: <https://github.com/googleapis/googleapis/blob/master/google/rpc/status.proto>
///
/// According to the OTLP spec, servers may attach `google.rpc.Status` details for certain
/// failures. In particular, `RESOURCE_EXHAUSTED` may include a `google.rpc.RetryInfo` entry.
///
/// See: <https://opentelemetry.io/docs/specs/otlp/#failures>
#[derive(Clone, PartialEq, ::prost::Message)]
struct RpcStatus {
    #[prost(int32, tag = "1")]
    pub code: i32,
    #[prost(string, tag = "2")]
    pub message: String,
    #[prost(message, repeated, tag = "3")]
    pub details: Vec<prost_types::Any>,
}

/// The `type.googleapis.com` URL for `google.rpc.RetryInfo`.
const RETRY_INFO_TYPE_URL: &str = "type.googleapis.com/google.rpc.RetryInfo";

/// Prost-generated struct for `google.rpc.RetryInfo` (subset).
///
/// Servers may attach this detail to signal how long the client should wait
/// before retrying. The hint is advisory.
///
/// See: <https://github.com/googleapis/googleapis/blob/master/google/rpc/error_details.proto>
#[derive(Clone, PartialEq, ::prost::Message)]
struct RetryInfo {
    #[prost(message, optional, tag = "1")]
    pub retry_delay: Option<prost_types::Duration>,
}

/// Determines whether a gRPC status represents a retryable error according to
/// the OTLP specification.
///
/// The retryability mapping follows the table at
/// <https://opentelemetry.io/docs/specs/otlp/#otlpgrpc-response>.
///
/// `RESOURCE_EXHAUSTED` is always treated as retryable. The `google.rpc.RetryInfo`
/// detail the server may attach is advisory only, so callers are not required to
/// honor it and its absence must not turn the failure permanent. See
/// [`retry_after`], which surfaces that advisory delay to callers.
///
/// `UNKNOWN` is treated as retryable because the generated gRPC client maps
/// temporary client-side readiness and transport failures, such as a channel
/// that is still reconnecting, to `UNKNOWN` before the RPC is sent. Treating
/// that as permanent would drop payloads that could succeed on retry.
fn is_retryable_grpc_status(status: &tonic::Status) -> bool {
    match status.code() {
        // Retryable per the OTLP spec, plus RESOURCE_EXHAUSTED (advisory
        // RetryInfo) and UNKNOWN (client-side readiness/transport failures).
        Code::Cancelled
        | Code::DeadlineExceeded
        | Code::Aborted
        | Code::OutOfRange
        | Code::Unavailable
        | Code::DataLoss
        | Code::ResourceExhausted
        | Code::Unknown => true,

        // All other codes (INVALID_ARGUMENT, NOT_FOUND, ALREADY_EXISTS,
        // PERMISSION_DENIED, UNAUTHENTICATED, FAILED_PRECONDITION,
        // UNIMPLEMENTED, INTERNAL, OK) are not retryable.
        _ => false,
    }
}

/// Extracts the server-suggested retry delay from a `google.rpc.RetryInfo`
/// detail carried in the status `grpc-status-details-bin` trailer, if present.
///
/// The details bytes are a serialized `google.rpc.Status` message whose
/// `details` field is a `repeated google.protobuf.Any`. We decode this, locate
/// the `RetryInfo` entry, and return its `retry_delay`.
///
/// This advisory hint is not consulted for the retry/permanent decision, which
/// is driven solely by the status code in [`is_retryable_grpc_status`].
fn retry_after(status: &tonic::Status) -> Option<prost_types::Duration> {
    use prost::Message;

    let detail_bytes = status.details();
    if detail_bytes.is_empty() {
        return None;
    }

    let rpc_status = RpcStatus::decode(detail_bytes).ok()?;
    rpc_status
        .details
        .iter()
        .find(|any| any.type_url == RETRY_INFO_TYPE_URL)
        .and_then(|any| RetryInfo::decode(any.value.as_slice()).ok())
        .and_then(|info| info.retry_delay)
}

/// Formats a `google.rpc.RetryInfo` retry delay as a compact, human-readable
/// duration for inclusion in a NACK reason string.
fn format_retry_delay(delay: &prost_types::Duration) -> String {
    // Normalize to whole seconds plus fractional milliseconds; RetryInfo delays
    // are advisory and typically coarse, so millisecond precision is sufficient.
    let millis = delay.seconds * 1_000 + (delay.nanos as i64) / 1_000_000;
    format!("{}ms", millis)
}

struct EncodedExport {
    bytes: Bytes,
    context: Context,
    saved_payload: OtapPayload,
    signal_type: SignalType,
    export_started_at: Instant,
    /// Per-request metadata plus the bearer token generation it carries.
    metadata: RequestMetadata,
}

/// Per-request gRPC metadata paired with the bearer token generation stamped
/// into it, so a later `UNAUTHENTICATED` can be matched to the exact token used.
struct RequestMetadata {
    /// gRPC metadata built from static headers, the propagation policy, and the
    /// bearer token. `None` when there is nothing to send (zero overhead).
    metadata: Option<MetadataMap>,
    /// Generation of the bearer token stamped into `metadata`. `None` when no
    /// token provider is bound.
    token_generation: Option<u64>,
}

/// Encoding failed before the request was sent; we still need to surface a Nack with payload.
struct EncodingFailure {
    error: Error,
    context: Context,
    saved_payload: OtapPayload,
}

fn prepare_otap_export<Enc: ProtoBytesEncoder>(
    mut otap_batch: OtapArrowRecords,
    context: Context,
    metadata: RequestMetadata,
    proto_buffer: &mut ProtoBuffer,
    encoder: &mut Enc,
    exporter: &NodeId,
    signal_type: SignalType,
    export_started_at: Instant,
) -> Result<EncodedExport, Box<EncodingFailure>> {
    proto_buffer.clear();
    if let Err(e) = encoder.encode(&mut otap_batch, proto_buffer) {
        let error = Error::ExporterError {
            exporter: exporter.clone(),
            kind: ExporterErrorKind::Other,
            error: format!("encoding error: {}", e),
            source_detail: "".to_string(),
        };

        if !context.may_return_payload() {
            let _drop = otap_batch.take_payload();
        }
        let saved_payload: OtapPayload = otap_batch.into();

        return Err(Box::new(EncodingFailure {
            error,
            context,
            saved_payload,
        }));
    }

    // Maintain the buffer's capacity across repeated calls.
    let (bytes, next_capacity) = proto_buffer.take_into_bytes();
    proto_buffer.ensure_capacity(next_capacity);

    if !context.may_return_payload() {
        // drop before the export, payload not requested
        let _drop = otap_batch.take_payload();
    }
    let saved_payload: OtapPayload = otap_batch.into();

    Ok(EncodedExport {
        bytes,
        context,
        saved_payload,
        signal_type,
        export_started_at,
        metadata,
    })
}

fn prepare_otlp_export(
    bytes: Bytes,
    context: Context,
    metadata: RequestMetadata,
    signal_type: SignalType,
    export_started_at: Instant,
    save_payload_fn: impl FnOnce(Bytes) -> OtapPayload,
) -> EncodedExport {
    let saved_payload = if context.may_return_payload() {
        save_payload_fn(bytes.clone())
    } else {
        save_payload_fn(Bytes::new())
    };

    EncodedExport {
        bytes,
        context,
        saved_payload,
        signal_type,
        export_started_at,
        metadata,
    }
}

/// Encode an OTAP Arrow batch and enqueue the export task; on encoding failure, emit a Nack.
#[allow(clippy::too_many_arguments)]
async fn dispatch_otap_export<Enc, Fut, MakeFuture>(
    otap_batch: OtapArrowRecords,
    context: Context,
    metadata: RequestMetadata,
    signal_type: SignalType,
    export_started_at: Instant,
    exporter_id: &NodeId,
    proto_buffer: &mut ProtoBuffer,
    encoder: &mut Enc,
    make_future: MakeFuture,
    inflight: &mut InFlightExports<Fut, CompletedExport>,
    metrics: &mut OtlpGrpcExporterMetrics,
    effect_handler: &EffectHandler<OtapPdata>,
) where
    Enc: ProtoBytesEncoder,
    Fut: Future<Output = CompletedExport>,
    MakeFuture: FnOnce(EncodedExport) -> Fut,
{
    match prepare_otap_export(
        otap_batch,
        context,
        metadata,
        proto_buffer,
        encoder,
        exporter_id,
        signal_type,
        export_started_at,
    ) {
        Ok(encoded) => {
            inflight.push(make_future(encoded));
        }
        Err(error) => {
            metrics.record_failure(
                signal_type,
                OtlpGrpcExporterErrorType::Encoding,
                export_started_at.elapsed(),
            );
            _ = notify_prepare_error(error, effect_handler).await;
        }
    }
}

async fn notify_prepare_error(
    error: Box<EncodingFailure>,
    effect_handler: &EffectHandler<OtapPdata>,
) -> Result<(), Error> {
    let EncodingFailure {
        error,
        context,
        saved_payload,
    } = *error;

    // Encoding failures are permanent: the data is malformed and retrying the
    // same payload will not succeed.
    effect_handler
        .notify_nack(NackMsg::new_permanent(
            error.to_string(),
            OtapPdata::new(context, saved_payload),
        ))
        .await?;

    Ok(())
}

/// Whether a completed export failed because the server rejected the bearer
/// token it carried.
///
/// With a bearer token provider bound, `UNAUTHENTICATED` usually means the
/// cached token lapsed or a refresh raced, so the batch can succeed once the
/// provider publishes its next token; callers therefore treat it as retryable and
/// invalidate the token generation that was used. Recovery waits for that
/// provider's own refresh schedule - invalidating only drops the exporter's
/// cached copy, it does not make the provider refresh early. `PERMISSION_DENIED`
/// is intentionally excluded: it signals a scope or permission problem that a
/// refresh will not fix. Always false when no provider is bound, since a
/// statically configured credential cannot be refreshed.
fn is_auth_failure(result: &Result<(), tonic::Status>, auth_bound: bool) -> bool {
    auth_bound && matches!(result, Err(status) if status.code() == Code::Unauthenticated)
}

/// Returns the bounded diagnostic category for a failed backend RPC.
fn export_error_type(result: &Result<(), tonic::Status>) -> Option<OtlpGrpcExporterErrorType> {
    result
        .as_ref()
        .err()
        .map(OtlpGrpcExporterErrorType::from_status)
}

/// NACKs `pdata` because no usable bearer token is available, and records the
/// failure.
///
/// Only for the paths that cannot wait for a token: shutdown force-draining
/// buffered pdata, and a batch left parked for in-flight capacity when the
/// cached token went unusable. Everywhere else the exporter back-pressures
/// instead. The NACK is retryable ([`NackMsg::new`] is non-permanent by default)
/// because a refreshed token may still arrive, so the batch is deferred rather
/// than dropped.
async fn nack_without_usable_token(
    pdata: OtapPdata,
    reason: &'static str,
    export_started_at: Instant,
    effect_handler: &EffectHandler<OtapPdata>,
    metrics: &mut OtlpGrpcExporterMetrics,
) {
    let signal_type = pdata.signal_type();
    let export_duration = export_started_at.elapsed();
    _ = effect_handler
        .notify_nack(NackMsg::new(reason, pdata))
        .await;
    metrics.record_failure(
        signal_type,
        OtlpGrpcExporterErrorType::Authentication,
        export_duration,
    );
}

/// Applies the Ack/Nack side effects for a completed gRPC export and returns the
/// reusable client, plus the bearer token generation the server rejected (if any).
async fn finalize_completed_export(
    completed: CompletedExport,
    effect_handler: &EffectHandler<OtapPdata>,
    metrics: &mut OtlpGrpcExporterMetrics,
) -> (SignalClient, Option<u64>) {
    let CompletedExport {
        result,
        context,
        saved_payload,
        signal_type,
        export_started_at,
        client,
        token_generation,
    } = completed;
    let export_duration = export_started_at.elapsed();

    // Record the rejected generation so the caller invalidates exactly the token
    // that was used, before the batch is retried. A stamped generation is what
    // "a provider is bound" means for this request: the dispatch path only
    // reaches a send with a usable token cached, so the generation is `Some`
    // exactly when the request carried a refreshable credential.
    let auth_failure = is_auth_failure(&result, token_generation.is_some());
    let rejected_generation = if auth_failure { token_generation } else { None };

    // The shared outcome describes the backend RPC, independently of whether
    // its Ack/Nack notification can be delivered to the upstream subscriber.
    if let Some(error_type) = export_error_type(&result) {
        metrics.record_failure(signal_type, error_type, export_duration);
    } else {
        metrics.record_success(signal_type, export_duration);
    }

    if let Err(e) = route_export_result(
        &result,
        context,
        saved_payload,
        effect_handler,
        auth_failure,
    )
    .await
    {
        otel_warn!(
            "otlp.exporter.grpc.export_error",
            message = "error routing export Ack/Nack",
            error = %e
        );
    } else if let Err(status) = &result {
        otel_warn!(
            "otlp.exporter.grpc.export_error",
            message = "service request error",
            code = %status.code(),
            error_msg = status.message(),
            source = format_error_sources(status)
        );
    }

    (client, rejected_generation)
}

/// Builds the per-request gRPC metadata by merging the pre-built static
/// `static_metadata` template with any headers propagated from the incoming
/// transport context and the refreshed bearer token, if one is cached.
///
/// Hot path: when there is neither static metadata, nor a propagation source,
/// nor a bearer token this returns `None` without allocating. The static
/// template is cloned only when present (each tonic request needs its own owned
/// metadata); propagated headers are appended on top so static and propagated
/// headers coexist.
///
/// Precedence, strongest first: a bound bearer token, then static config, then
/// propagated transport headers. So a propagated header whose key matches a
/// statically configured one is dropped -- a configured backend credential
/// (e.g. `authorization`) can never be overridden or duplicated by inbound
/// transport headers -- and a refreshed bearer token in turn replaces any
/// `authorization` from either source.
fn build_grpc_metadata(
    effect_handler: &EffectHandler<OtapPdata>,
    context: &Context,
    static_metadata: Option<&MetadataMap>,
    auth_header: Option<HeaderValue>,
) -> Option<MetadataMap> {
    let propagation = effect_handler
        .propagation_policy()
        .zip(context.transport_headers());

    // Zero-alloc fast path: nothing static configured, nothing to propagate, no token.
    if static_metadata.is_none() && propagation.is_none() && auth_header.is_none() {
        return None;
    }

    let mut metadata = match static_metadata {
        Some(static_metadata) => static_metadata.clone(),
        None => MetadataMap::new(),
    };

    if let Some((policy, transport_headers)) = propagation {
        for header in policy.propagate(transport_headers) {
            match header.value_kind {
                ValueKind::Text => {
                    // ASCII metadata: parse the header name and value.
                    let Ok(key) = header
                        .header_name
                        .parse::<MetadataKey<tonic::metadata::Ascii>>()
                    else {
                        otel_debug!(
                            "otlp.exporter.grpc.header_skip",
                            reason = "invalid ascii metadata key",
                            header_name = header.header_name
                        );
                        continue;
                    };
                    let Ok(value) = MetadataValue::try_from(header.value) else {
                        otel_debug!(
                            "otlp.exporter.grpc.header_skip",
                            reason = "invalid ascii metadata value",
                            header_name = header.header_name
                        );
                        continue;
                    };
                    // Static config wins: a statically configured header (e.g. an
                    // `authorization` backend credential) must not be duplicated or
                    // overridden by a propagated header with the same key. Static
                    // metadata is ASCII-only, so only text headers can collide.
                    if static_metadata.is_some_and(|s| s.contains_key(key.as_str())) {
                        otel_debug!(
                            "otlp.exporter.grpc.header_skip",
                            reason = "static header takes precedence over propagated header",
                            header_name = header.header_name
                        );
                        continue;
                    }
                    let _ = metadata.append(key, value);
                }
                ValueKind::Binary => {
                    // Binary metadata: gRPC binary metadata keys must end with `-bin`.
                    // Metadata map will error if attempting to insert key without `-bin`.
                    let key_name = if header.header_name.ends_with("-bin") {
                        header.header_name.to_string()
                    } else {
                        format!("{}-bin", header.header_name)
                    };
                    let Ok(key) = key_name.parse::<MetadataKey<tonic::metadata::Binary>>() else {
                        otel_debug!(
                            "otlp.exporter.grpc.header_skip",
                            reason = "invalid binary metadata key",
                            header_name = header.header_name
                        );
                        continue;
                    };
                    let value = MetadataValue::from_bytes(header.value);
                    let _ = metadata.append_bin(key, value);
                }
            }
        }
    }

    // The refreshed bearer token replaces any `authorization` from static config
    // or propagation. Going through the backing `HeaderMap` keeps the value's
    // `sensitive` flag, which excludes the credential from HPACK indexing, and
    // avoids re-validating and copying the token bytes on every request.
    if let Some(auth_header) = auth_header {
        let mut headers = metadata.into_headers();
        let _ = headers.insert(http::header::AUTHORIZATION, auth_header);
        metadata = MetadataMap::from_headers(headers);
    }

    if metadata.is_empty() {
        None
    } else {
        Some(metadata)
    }
}

/// Builds an export future for the provided payload, borrowing a signal-specific client from the pool.
///
/// When `metadata` is present in the [`EncodedExport`], the outbound gRPC
/// request carries those entries as request metadata (HTTP/2 headers).
/// Otherwise the request is sent with an empty metadata map (zero overhead).
fn make_export_future(
    prepared: EncodedExport,
    client: SignalClient,
) -> impl Future<Output = CompletedExport> {
    let EncodedExport {
        bytes,
        context,
        saved_payload,
        signal_type,
        export_started_at,
        metadata: RequestMetadata {
            metadata,
            token_generation,
        },
    } = prepared;

    // Build a tonic::Request that carries the propagated metadata.
    let request = match metadata {
        Some(md) => tonic::Request::from_parts(md, tonic::Extensions::new(), bytes),
        None => tonic::Request::new(bytes),
    };

    async move {
        match client {
            SignalClient::Logs(mut client) => {
                let result = client.export(request).await.map(|_| ());
                CompletedExport {
                    result,
                    context,
                    saved_payload,
                    signal_type,
                    export_started_at,
                    client: SignalClient::Logs(client),
                    token_generation,
                }
            }
            SignalClient::Metrics(mut client) => {
                let result = client.export(request).await.map(|_| ());
                CompletedExport {
                    result,
                    context,
                    saved_payload,
                    signal_type,
                    export_started_at,
                    client: SignalClient::Metrics(client),
                    token_generation,
                }
            }
            SignalClient::Traces(mut client) => {
                let result = client.export(request).await.map(|_| ());
                CompletedExport {
                    result,
                    context,
                    saved_payload,
                    signal_type,
                    export_started_at,
                    client: SignalClient::Traces(client),
                    token_generation,
                }
            }
        }
    }
}

/// FIFO-ish wrapper around the in-flight export RPCs.
pub(crate) struct InFlightExports<Fut, Output>
where
    Fut: Future<Output = Output>,
{
    futures: FuturesUnordered<Fut>,
}

impl<Fut, Output> InFlightExports<Fut, Output>
where
    Fut: Future<Output = Output>,
{
    pub(crate) fn new() -> Self {
        Self {
            futures: FuturesUnordered::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.futures.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.futures.is_empty()
    }

    pub(crate) fn push(&mut self, future: Fut) {
        self.futures.push(future);
    }

    /// Returns a future that resolves once the next export finishes.
    pub(crate) fn next_completion(&mut self) -> impl Future<Output = Option<Output>> + '_ {
        self.futures.next()
    }
}

/// Keeps a small stash of gRPC clients so each export can reuse an existing connection.
/// When multiple channels are configured, clients are distributed across them in
/// round-robin order so that exports spread evenly across TCP connections.
struct GrpcClientPool {
    channels: Vec<Channel>,
    compression: Option<CompressionEncoding>,
    logs: VecDeque<LogsServiceClient<Channel>>,
    metrics: VecDeque<MetricsServiceClient<Channel>>,
    traces: VecDeque<TraceServiceClient<Channel>>,
}

impl GrpcClientPool {
    fn new(
        max_in_flight: usize,
        channels: Vec<Channel>,
        compression: Option<CompressionEncoding>,
    ) -> Self {
        // Pool must hold at least one client per channel so every TCP connection
        // gets exercised, even when max_in_flight is smaller than the channel count.
        let pool_size = max_in_flight.max(channels.len());
        Self {
            channels,
            compression,
            logs: VecDeque::with_capacity(pool_size),
            metrics: VecDeque::with_capacity(pool_size),
            traces: VecDeque::with_capacity(pool_size),
        }
    }

    /// Eagerly build up to `max_in_flight` clients per signal, distributing them
    /// round-robin across the available channels.
    fn prepopulate_clients(&mut self) {
        let logs_cap = self.logs.capacity();
        for i in 0..logs_cap {
            let channel = self.channels[i % self.channels.len()].clone();
            self.logs.push_back(self.make_logs_client_with(channel));
        }

        let metrics_cap = self.metrics.capacity();
        for i in 0..metrics_cap {
            let channel = self.channels[i % self.channels.len()].clone();
            self.metrics
                .push_back(self.make_metrics_client_with(channel));
        }

        let traces_cap = self.traces.capacity();
        for i in 0..traces_cap {
            let channel = self.channels[i % self.channels.len()].clone();
            self.traces.push_back(self.make_traces_client_with(channel));
        }
    }

    #[inline(always)]
    fn take_logs(&mut self) -> LogsServiceClient<Channel> {
        self.logs
            .pop_front()
            .expect("client pool underflow: take_logs called with empty pool")
    }

    #[inline(always)]
    fn take_metrics(&mut self) -> MetricsServiceClient<Channel> {
        self.metrics
            .pop_front()
            .expect("client pool underflow: take_metrics called with empty pool")
    }

    #[inline(always)]
    fn take_traces(&mut self) -> TraceServiceClient<Channel> {
        self.traces
            .pop_front()
            .expect("client pool underflow: take_traces called with empty pool")
    }

    fn release(&mut self, client: SignalClient) {
        match client {
            SignalClient::Logs(client) => self.logs.push_back(client),
            SignalClient::Metrics(client) => self.metrics.push_back(client),
            SignalClient::Traces(client) => self.traces.push_back(client),
        }
    }

    fn make_logs_client_with(&self, channel: Channel) -> LogsServiceClient<Channel> {
        let mut client = LogsServiceClient::new(channel);
        if let Some(encoding) = self.compression {
            client = client.send_compressed(encoding);
            client = client.accept_compressed(encoding);
        }
        client
    }

    fn make_metrics_client_with(&self, channel: Channel) -> MetricsServiceClient<Channel> {
        let mut client = MetricsServiceClient::new(channel);
        if let Some(encoding) = self.compression {
            client = client.send_compressed(encoding);
            client = client.accept_compressed(encoding);
        }
        client
    }

    fn make_traces_client_with(&self, channel: Channel) -> TraceServiceClient<Channel> {
        let mut client = TraceServiceClient::new(channel);
        if let Some(encoding) = self.compression {
            client = client.send_compressed(encoding);
            client = client.accept_compressed(encoding);
        }
        client
    }
}

enum SignalClient {
    Logs(LogsServiceClient<Channel>),
    Metrics(MetricsServiceClient<Channel>),
    Traces(TraceServiceClient<Channel>),
}

/// Captures everything we need once a single export RPC has completed.
struct CompletedExport {
    result: Result<(), tonic::Status>,
    context: Context,
    saved_payload: OtapPayload,
    signal_type: SignalType,
    export_started_at: Instant,
    client: SignalClient,
    /// Generation of the bearer token this request carried, echoed back so an
    /// `UNAUTHENTICATED` response invalidates exactly that token and a stale
    /// rejection is ignored. `None` when no token provider is bound.
    token_generation: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use otap_df_config::node::NodeUserConfig;
    use otap_df_otap::bearer_auth::test_support::MockTokenProvider;
    use std::collections::HashMap;

    use otap_df_config::transport_headers::{TransportHeader, TransportHeaders};
    use otap_df_config::transport_headers_policy::PropagationSelectorType;
    use otap_df_config::transport_headers_policy::{
        HeaderPropagationPolicy, PropagationAction, PropagationDefault, PropagationMatch,
        PropagationOverride, PropagationSelector,
    };
    use otap_df_engine::Interests;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::PipelineCompletionMsg;
    use otap_df_engine::control::{
        Controllable, PipelineCompletionMsgSender, RuntimeCtrlMsgSender,
        pipeline_completion_msg_channel, runtime_ctrl_msg_channel,
    };
    use otap_df_engine::error::Error;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::local::message::{LocalReceiver, LocalSender};
    use otap_df_engine::message::{Receiver, Sender};
    use otap_df_engine::node::NodeWithPDataReceiver;
    use otap_df_engine::testing::create_not_send_channel;
    use otap_df_engine::testing::{
        exporter::{TestContext, TestRuntime},
        test_node,
    };
    use otap_df_otap::otlp_grpc::OTLPData;
    use otap_df_otap::otlp_mock::{LogsServiceMock, MetricsServiceMock, TraceServiceMock};
    use otap_df_otap::pdata::OtapPdata;
    use otap_df_otap::testing::{TestCallData, next_ack, next_nack};
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::logs_service_server::LogsServiceServer;
    use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::metrics_service_server::MetricsServiceServer;
    use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    use otap_df_pdata::proto::opentelemetry::collector::trace::v1::trace_service_server::TraceServiceServer;
    use otap_df_telemetry::metrics::MetricSetSnapshot;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_telemetry::reporter::MetricsReporter;
    use prost::Message;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::time::Instant;
    use tokio::net::TcpListener;
    use tokio::runtime::Runtime;
    use tokio::time::{Duration, timeout};
    use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
    use tonic::transport::Server;

    /// Helper function to wait for and validate an Ack or Nack message with the expected node_id
    async fn wait_for_ack_or_nack(
        pipeline_completion_rx: &mut otap_df_engine::control::PipelineCompletionMsgReceiver<
            OtapPdata,
        >,
        expect_ack: bool,
        expected_node_id: usize,
        context: &str,
    ) -> Result<(), String> {
        let result = timeout(Duration::from_secs(1), async {
            match pipeline_completion_rx.recv().await {
                Ok(PipelineCompletionMsg::DeliverAck { ack }) => {
                    if !expect_ack {
                        return Err(format!("Got Ack but expected Nack {}", context));
                    }
                    let (node_id, _ack) = next_ack(ack)
                        .ok_or_else(|| format!("No ack subscriber found {}", context))?;
                    if node_id != expected_node_id {
                        return Err(format!(
                            "Expected node_id {} but got {} {}",
                            expected_node_id, node_id, context
                        ));
                    }
                    Ok(())
                }
                Ok(PipelineCompletionMsg::DeliverNack { nack }) => {
                    if expect_ack {
                        return Err(format!("Got Nack but expected Ack {}", context));
                    }
                    let (node_id, _nack) = next_nack(nack)
                        .ok_or_else(|| format!("No nack subscriber found {}", context))?;
                    if node_id != expected_node_id {
                        return Err(format!(
                            "Expected node_id {} but got {} {}",
                            expected_node_id, node_id, context
                        ));
                    }
                    Ok(())
                }
                Err(_) => Err(format!("Channel closed {}", context)),
            }
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(format!("Timeout waiting for Ack/Nack {}", context)),
        }
    }

    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    fn scenario() -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {
                // Send a data message
                let req = ExportLogsServiceRequest::default();
                let mut req_bytes = vec![];
                req.encode(&mut req_bytes).unwrap();
                let logs_pdata = OtapPdata::new_default(
                    OtlpProtoBytes::ExportLogsRequest(Bytes::from(req_bytes)).into(),
                )
                .test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::default().into(),
                    123,
                );
                ctx.send_pdata(logs_pdata)
                    .await
                    .expect("Failed to send log message");

                let req = ExportMetricsServiceRequest::default();
                let mut req_bytes = vec![];
                req.encode(&mut req_bytes).unwrap();
                let metrics_pdata = OtapPdata::new_default(
                    OtlpProtoBytes::ExportMetricsRequest(Bytes::from(req_bytes)).into(),
                )
                .test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::default().into(),
                    123,
                );
                ctx.send_pdata(metrics_pdata)
                    .await
                    .expect("Failed to send metric message");

                let req = ExportTraceServiceRequest::default();
                let mut req_bytes = vec![];
                req.encode(&mut req_bytes).unwrap();
                let traces_pdata = OtapPdata::new_default(
                    OtlpProtoBytes::ExportTracesRequest(Bytes::from(req_bytes)).into(),
                )
                .test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::default().into(),
                    123,
                );
                ctx.send_pdata(traces_pdata)
                    .await
                    .expect("Failed to send metric message");

                // Send shutdown
                ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the expected counter values
    fn validation_procedure(
        mut receiver: tokio::sync::mpsc::Receiver<OTLPData>,
    ) -> impl FnOnce(TestContext<OtapPdata>, Result<(), Error>) -> Pin<Box<dyn Future<Output = ()>>>
    {
        |_, exporter_result| {
            Box::pin(async move {
                exporter_result.unwrap();

                // check that the message was properly sent from the exporter
                let logs_received = timeout(Duration::from_secs(3), receiver.recv())
                    .await
                    .expect("Timed out waiting for message");
                // Assert that the message received is what the exporter sent
                let _expected_logs_message = ExportLogsServiceRequest::default();
                assert!(matches!(logs_received, _expected_logs_message));

                let metrics_received = timeout(Duration::from_secs(3), receiver.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

                let _expected_metrics_message = ExportMetricsServiceRequest::default();
                assert!(matches!(metrics_received, _expected_metrics_message));

                let traces_received = timeout(Duration::from_secs(3), receiver.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");
                let _expected_trace_message = ExportTraceServiceRequest::default();
                assert!(matches!(traces_received, _expected_trace_message));
            })
        }
    }

    #[test]
    fn test_otlp_exporter() {
        let test_runtime = TestRuntime::new();
        let (sender, receiver) = tokio::sync::mpsc::channel(32);
        let (shutdown_sender, shutdown_signal) = tokio::sync::oneshot::channel();
        let (ready_sender, ready_receiver) = tokio::sync::oneshot::channel();
        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let listening_addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        // tokio runtime to run grpc server in the background
        let tokio_rt = Runtime::new().unwrap();

        // run a gRPC concurrently to receive data from the exporter
        _ = tokio_rt.spawn(async move {
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            // Signal that the server is ready to accept connections
            let _ = ready_sender.send(());
            let tcp_stream = TcpListenerStream::new(tcp_listener);
            let mock_logs_service = LogsServiceServer::new(LogsServiceMock::new(sender.clone()));
            let mock_metrics_service =
                MetricsServiceServer::new(MetricsServiceMock::new(sender.clone()));
            let mock_trace_service = TraceServiceServer::new(TraceServiceMock::new(sender.clone()));
            Server::builder()
                .add_service(mock_logs_service)
                .add_service(mock_metrics_service)
                .add_service(mock_trace_service)
                .serve_with_incoming_shutdown(tcp_stream, async {
                    // Wait for the shutdown signal
                    let _ = shutdown_signal.await;
                })
                .await
                .expect("Test gRPC server has failed");
        });

        // Wait for the server to be ready before creating the exporter
        tokio_rt
            .block_on(ready_receiver)
            .expect("Server failed to start");

        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTLP_EXPORTER_URN));

        // Create a proper pipeline context for the test
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let exporter = ExporterWrapper::local(
            OTLPExporter {
                config: Config {
                    grpc: GrpcClientSettings {
                        grpc_endpoint: grpc_endpoint.clone(),
                        ..Default::default()
                    },
                    max_in_flight: 32,
                    num_connections: default_num_connections(),
                },
                metrics: OtlpGrpcExporterMetrics::register(&pipeline_ctx),
                token_provider: None,
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    // Validate that we received 3 Acks
                    let mut ack_count = 0;
                    let mut pipeline_completion_rx =
                        ctx.take_pipeline_completion_receiver().unwrap();

                    // Validate that we received 3 Acks with correct node_id
                    for i in 0..3 {
                        wait_for_ack_or_nack(
                            &mut pipeline_completion_rx,
                            true,
                            123,
                            &format!("for export #{}", i + 1),
                        )
                        .await
                        .expect("Failed to receive Ack");
                        ack_count += 1;
                    }

                    assert_eq!(ack_count, 3, "Expected 3 Acks for 3 successful exports");
                    validation_procedure(receiver)(ctx, result).await;
                })
            });

        _ = shutdown_sender.send("Shutdown");
    }

    #[test]
    fn test_otlp_exporter_sends_configured_static_headers() {
        // End-to-end proof that a configured static header (here an
        // `authorization` token) is actually transmitted as gRPC metadata on
        // every outbound export, driving the real `OTLPExporter`.
        let test_runtime = TestRuntime::new();
        let (sender, receiver) = tokio::sync::mpsc::channel(32);
        let (shutdown_sender, shutdown_signal) = tokio::sync::oneshot::channel();
        let (ready_sender, ready_receiver) = tokio::sync::oneshot::channel();
        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let listening_addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let tokio_rt = Runtime::new().unwrap();

        // The server-side interceptor records the inbound `authorization`
        // metadata so the test can assert the configured header reached the wire.
        let captured_auth: Arc<std::sync::Mutex<Option<String>>> =
            Arc::new(std::sync::Mutex::new(None));
        let captured_auth_srv = captured_auth.clone();

        _ = tokio_rt.spawn(async move {
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            let _ = ready_sender.send(());
            let tcp_stream = TcpListenerStream::new(tcp_listener);

            let interceptor =
                move |req: tonic::Request<()>| -> Result<tonic::Request<()>, tonic::Status> {
                    if let Some(value) = req.metadata().get("authorization") {
                        if let Ok(value) = value.to_str() {
                            *captured_auth_srv.lock().unwrap() = Some(value.to_string());
                        }
                    }
                    Ok(req)
                };

            let mock_logs_service = LogsServiceServer::with_interceptor(
                LogsServiceMock::new(sender.clone()),
                interceptor.clone(),
            );
            let mock_metrics_service = MetricsServiceServer::with_interceptor(
                MetricsServiceMock::new(sender.clone()),
                interceptor.clone(),
            );
            let mock_trace_service = TraceServiceServer::with_interceptor(
                TraceServiceMock::new(sender.clone()),
                interceptor,
            );
            Server::builder()
                .add_service(mock_logs_service)
                .add_service(mock_metrics_service)
                .add_service(mock_trace_service)
                .serve_with_incoming_shutdown(tcp_stream, async {
                    let _ = shutdown_signal.await;
                })
                .await
                .expect("Test gRPC server has failed");
        });

        tokio_rt
            .block_on(ready_receiver)
            .expect("Server failed to start");

        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTLP_EXPORTER_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let mut headers = HashMap::new();
        _ = headers.insert(
            "authorization".to_string(),
            "Bearer secret-token-123".into(),
        );

        let exporter = ExporterWrapper::local(
            OTLPExporter {
                config: Config {
                    grpc: GrpcClientSettings {
                        grpc_endpoint: grpc_endpoint.clone(),
                        headers,
                        ..Default::default()
                    },
                    max_in_flight: 32,
                    num_connections: default_num_connections(),
                },
                metrics: OtlpGrpcExporterMetrics::register(&pipeline_ctx),
                token_provider: None,
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    let mut pipeline_completion_rx =
                        ctx.take_pipeline_completion_receiver().unwrap();
                    for i in 0..3 {
                        wait_for_ack_or_nack(
                            &mut pipeline_completion_rx,
                            true,
                            123,
                            &format!("for export #{}", i + 1),
                        )
                        .await
                        .expect("Failed to receive Ack");
                    }
                    validation_procedure(receiver)(ctx, result).await;
                })
            });

        _ = shutdown_sender.send("Shutdown");

        let captured = captured_auth.lock().unwrap().clone();
        assert_eq!(
            captured.as_deref(),
            Some("Bearer secret-token-123"),
            "the configured authorization header must reach the gRPC server"
        );
    }

    /// Drives the real `OTLPExporter` end to end against a mock gRPC server with
    /// `provider` bound and `static_headers` configured, running the standard
    /// three-export-then-shutdown scenario and asserting every export is Ack'd.
    /// Returns every `authorization` metadata value the server observed, in
    /// arrival order, so a test can assert both which credential arrived and
    /// that exactly one did per request.
    fn run_bearer_wire_test(
        provider: MockTokenProvider,
        static_headers: &[(&str, &str)],
    ) -> Vec<String> {
        let test_runtime = TestRuntime::new();
        let (sender, receiver) = tokio::sync::mpsc::channel(32);
        let (shutdown_sender, shutdown_signal) = tokio::sync::oneshot::channel();
        let (ready_sender, ready_receiver) = tokio::sync::oneshot::channel();
        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let listening_addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let tokio_rt = Runtime::new().unwrap();

        let captured_auth: Arc<std::sync::Mutex<Vec<String>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let captured_auth_srv = captured_auth.clone();

        _ = tokio_rt.spawn(async move {
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            let _ = ready_sender.send(());
            let tcp_stream = TcpListenerStream::new(tcp_listener);

            let interceptor =
                move |req: tonic::Request<()>| -> Result<tonic::Request<()>, tonic::Status> {
                    let values: Vec<String> = req
                        .metadata()
                        .get_all("authorization")
                        .iter()
                        .filter_map(|v| v.to_str().ok().map(str::to_string))
                        .collect();
                    captured_auth_srv.lock().unwrap().extend(values);
                    Ok(req)
                };

            let mock_logs_service = LogsServiceServer::with_interceptor(
                LogsServiceMock::new(sender.clone()),
                interceptor.clone(),
            );
            let mock_metrics_service = MetricsServiceServer::with_interceptor(
                MetricsServiceMock::new(sender.clone()),
                interceptor.clone(),
            );
            let mock_trace_service = TraceServiceServer::with_interceptor(
                TraceServiceMock::new(sender.clone()),
                interceptor,
            );
            Server::builder()
                .add_service(mock_logs_service)
                .add_service(mock_metrics_service)
                .add_service(mock_trace_service)
                .serve_with_incoming_shutdown(tcp_stream, async {
                    let _ = shutdown_signal.await;
                })
                .await
                .expect("Test gRPC server has failed");
        });

        tokio_rt
            .block_on(ready_receiver)
            .expect("Server failed to start");

        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTLP_EXPORTER_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let headers: HashMap<String, _> = static_headers
            .iter()
            .map(|(name, value)| ((*name).to_string(), (*value).into()))
            .collect();

        let exporter = ExporterWrapper::local(
            OTLPExporter {
                config: Config {
                    grpc: GrpcClientSettings {
                        grpc_endpoint: grpc_endpoint.clone(),
                        headers,
                        ..Default::default()
                    },
                    max_in_flight: 32,
                    num_connections: default_num_connections(),
                },
                metrics: OtlpGrpcExporterMetrics::register(&pipeline_ctx),
                token_provider: Some(Box::new(provider)),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    let mut pipeline_completion_rx =
                        ctx.take_pipeline_completion_receiver().unwrap();
                    for i in 0..3 {
                        wait_for_ack_or_nack(
                            &mut pipeline_completion_rx,
                            true,
                            123,
                            &format!("for export #{}", i + 1),
                        )
                        .await
                        .expect("Failed to receive Ack");
                    }
                    validation_procedure(receiver)(ctx, result).await;
                })
            });

        _ = shutdown_sender.send("Shutdown");

        captured_auth.lock().unwrap().clone()
    }

    /// Scenario: a `bearer_token_provider` is bound and has published a token,
    /// while the config also carries a static `authorization` header.
    /// Guarantees: every outbound export carries the provider's refreshed token
    /// as its only `authorization` metadata, so a stale configured credential
    /// can neither override the live token nor be sent alongside it.
    #[test]
    fn bearer_token_reaches_the_grpc_server_and_overrides_a_static_header() {
        let captured = run_bearer_wire_test(
            MockTokenProvider::new("provider-token"),
            &[("authorization", "Basic static")],
        );

        assert_eq!(
            captured,
            vec!["Bearer provider-token".to_string(); 3],
            "each export must carry the provider's token as its only authorization"
        );
    }

    /// Scenario: the provider's first publication cannot form a header value
    /// (it contains a newline) and a valid token follows on the same stream.
    /// Guarantees: the malformed token is skipped rather than aborting the
    /// exporter or being sent, and exports proceed with the next valid token, so
    /// one bad publication costs a refresh rather than the pipeline.
    #[test]
    fn an_invalid_bearer_token_is_skipped_and_the_next_valid_one_is_used() {
        let captured = run_bearer_wire_test(
            MockTokenProvider {
                tokens: vec!["bad\nvalue".to_string(), "good-token".to_string()],
                keep_open: true,
                expires_on: None,
            },
            &[],
        );

        assert_eq!(
            captured,
            vec!["Bearer good-token".to_string(); 3],
            "the malformed token must be skipped and the next valid one used"
        );
    }

    /// Scenario: the provider publishes one token and then closes its stream, so
    /// no further refreshes can arrive.
    /// Guarantees: the exporter keeps using the last token instead of treating
    /// the closure as a loss of credentials, so a provider that stops refreshing
    /// degrades to a static credential rather than stalling the pipeline.
    #[test]
    fn the_last_bearer_token_is_reused_after_the_provider_closes_its_stream() {
        let captured = run_bearer_wire_test(
            MockTokenProvider {
                tokens: vec!["final-token".to_string()],
                keep_open: false,
                expires_on: None,
            },
            &[],
        );

        assert_eq!(
            captured,
            vec!["Bearer final-token".to_string(); 3],
            "the last token must keep being used after the stream closes"
        );
    }

    /// Scenario: a `bearer_token_provider` is bound but never publishes a token,
    /// then the pipeline shuts down with a batch still buffered.
    /// Guarantees: nothing is sent unauthenticated -- intake stays gated so the
    /// server sees no request at all -- and the batch shutdown force-drains is
    /// NACK'd retryably with its payload intact, so it is deferred rather than
    /// dropped.
    #[test]
    fn an_unavailable_token_gates_intake_and_shutdown_nacks_retryably() {
        let test_runtime = TestRuntime::new();
        let (sender, mut receiver) = tokio::sync::mpsc::channel(32);
        let (shutdown_sender, shutdown_signal) = tokio::sync::oneshot::channel();
        let (ready_sender, ready_receiver) = tokio::sync::oneshot::channel();
        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let listening_addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let tokio_rt = Runtime::new().unwrap();

        _ = tokio_rt.spawn(async move {
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            let _ = ready_sender.send(());
            let tcp_stream = TcpListenerStream::new(tcp_listener);
            Server::builder()
                .add_service(LogsServiceServer::new(LogsServiceMock::new(sender)))
                .serve_with_incoming_shutdown(tcp_stream, async {
                    let _ = shutdown_signal.await;
                })
                .await
                .expect("Test gRPC server has failed");
        });

        tokio_rt
            .block_on(ready_receiver)
            .expect("Server failed to start");

        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTLP_EXPORTER_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let exporter = ExporterWrapper::local(
            OTLPExporter {
                config: Config {
                    grpc: GrpcClientSettings {
                        grpc_endpoint: grpc_endpoint.clone(),
                        ..Default::default()
                    },
                    max_in_flight: 32,
                    num_connections: default_num_connections(),
                },
                metrics: OtlpGrpcExporterMetrics::register(&pipeline_ctx),
                token_provider: Some(Box::new(MockTokenProvider::never_publishes())),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        // A non-empty request so the NACK'd payload is distinguishable from an
        // empty one.
        let req = ExportLogsServiceRequest {
            resource_logs: vec![Default::default()],
        };
        let mut req_bytes = vec![];
        req.encode(&mut req_bytes).unwrap();
        let sent_bytes = Bytes::from(req_bytes);
        let expected_bytes = sent_bytes.clone();

        test_runtime
            .set_exporter(exporter)
            .run_test(move |ctx| {
                Box::pin(async move {
                    let logs_pdata = OtapPdata::new_default(
                        OtlpProtoBytes::ExportLogsRequest(sent_bytes).into(),
                    )
                    .test_subscribe_to(
                        Interests::ACKS | Interests::NACKS,
                        TestCallData::default().into(),
                        123,
                    );
                    ctx.send_pdata(logs_pdata)
                        .await
                        .expect("Failed to send log message");
                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .expect("Failed to send Shutdown");
                })
            })
            .run_validation(move |mut ctx, result| {
                Box::pin(async move {
                    result.unwrap();
                    let mut pipeline_completion_rx =
                        ctx.take_pipeline_completion_receiver().unwrap();
                    let msg = timeout(Duration::from_secs(3), pipeline_completion_rx.recv())
                        .await
                        .expect("timed out waiting for a completion")
                        .expect("completion channel closed");
                    match msg {
                        PipelineCompletionMsg::DeliverNack { nack } => {
                            assert!(
                                !nack.permanent,
                                "a batch refused for a missing token must stay retryable"
                            );
                            assert!(
                                nack.reason.contains("bearer token unavailable"),
                                "unexpected NACK reason: {}",
                                nack.reason
                            );
                            let (_context, payload) = (*nack.refused).into_parts();
                            match payload {
                                OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(
                                    bytes,
                                )) => assert_eq!(
                                    bytes, expected_bytes,
                                    "the refused batch must be returned intact so it can be retried"
                                ),
                                other => panic!("unexpected refused payload: {other:?}"),
                            }
                        }
                        PipelineCompletionMsg::DeliverAck { .. } => {
                            panic!("unexpected Ack: no request should have been sent")
                        }
                    }
                })
            });

        _ = shutdown_sender.send("Shutdown");

        assert!(
            receiver.try_recv().is_err(),
            "no export may reach the server while no usable token is cached"
        );
    }

    /// Scenario: The OTLP gRPC endpoint repeatedly stops and restarts while exporting logs.
    /// Guarantees: The exporter reconnects and reports one terminal outcome per export operation.
    #[test]
    fn test_receiver_not_ready_on_start_and_reconnect() {
        // the purpose of this test is to that the exporter behaves as expected in the face of
        // server that may start and stop asynchronously of the exporter. it ensures the exporter
        // doesn't exit early if it can't make the initial connection, and also that the grpc
        // client will reconnect in the event of a server shutdown

        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");

        let tokio_rt = Runtime::new().unwrap();

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTLP_EXPORTER_URN));

        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let node_id = test_node(test_runtime.config().name.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let mut exporter = ExporterWrapper::local(
            OTLPExporter {
                config: Config {
                    grpc: GrpcClientSettings {
                        grpc_endpoint: grpc_endpoint.clone(),
                        connect_timeout: Duration::from_millis(500),
                        ..Default::default()
                    },
                    max_in_flight: 32,
                    num_connections: default_num_connections(),
                },
                metrics: OtlpGrpcExporterMetrics::register(&pipeline_ctx),
                token_provider: None,
            },
            node_id.clone(),
            node_config,
            test_runtime.config(),
        );

        let control_sender = exporter.control_sender();
        let (pdata_tx, pdata_rx) = create_not_send_channel::<OtapPdata>(1);
        let pdata_tx = Sender::Local(LocalSender::mpsc(pdata_tx));
        let pdata_rx = Receiver::Local(LocalReceiver::mpsc(pdata_rx));
        let (runtime_ctrl_msg_tx, _runtime_ctrl_msg_rx) = runtime_ctrl_msg_channel(2);
        let (pipeline_completion_msg_tx, pipeline_completion_msg_rx) =
            pipeline_completion_msg_channel(2);
        exporter
            .set_pdata_receiver(node_id.clone(), pdata_rx)
            .expect("Failed to set PData Receiver");

        // channels for coordinating the test
        let (server_startup_sender, mut server_startup_receiver) = tokio::sync::mpsc::channel(1);
        let (server_start_ack_sender, server_start_ack_receiver) = tokio::sync::mpsc::channel(1);
        let (shutdown_sender1, shutdown_signal1) = tokio::sync::oneshot::channel();
        let (shutdown_sender2, shutdown_signal2) = tokio::sync::oneshot::channel();
        let (server_shutdown_ack_sender, server_shutdown_ack_receiver) =
            tokio::sync::mpsc::channel(1);
        let (req_sender, req_receiver) = tokio::sync::mpsc::channel(32);

        async fn start_exporter(
            exporter: ExporterWrapper<OtapPdata>,
            runtime_ctrl_msg_tx: RuntimeCtrlMsgSender<OtapPdata>,
            pipeline_completion_msg_tx: PipelineCompletionMsgSender<OtapPdata>,
            metrics_reporter: MetricsReporter,
        ) -> Result<(), Error> {
            exporter
                .start(
                    runtime_ctrl_msg_tx,
                    pipeline_completion_msg_tx,
                    metrics_reporter,
                    Interests::empty(),
                )
                .await
                .map(|_| ())
        }

        async fn drive_test(
            server_startup_sender: tokio::sync::mpsc::Sender<bool>,
            mut server_startup_ack_receiver: tokio::sync::mpsc::Receiver<bool>,
            mut server_shutdown_ack_receiver: tokio::sync::mpsc::Receiver<bool>,
            server_shutdown_signal1: tokio::sync::oneshot::Sender<bool>,
            server_shutdown_signal2: tokio::sync::oneshot::Sender<bool>,
            pdata_tx: Sender<OtapPdata>,
            control_sender: Sender<NodeControlMsg<OtapPdata>>,
            mut pipeline_completion_msg_rx: otap_df_engine::control::PipelineCompletionMsgReceiver<
                OtapPdata,
            >,
            mut req_receiver: tokio::sync::mpsc::Receiver<OTLPData>,
            metrics_receiver: flume::Receiver<MetricSetSnapshot>,
            metrics_reporter: MetricsReporter,
        ) -> Result<(), Error> {
            // pdata
            let req = ExportLogsServiceRequest::default();
            let mut req_bytes = vec![];
            req.encode(&mut req_bytes).unwrap();

            // send a request while the server isn't running and check how we handle it
            let pdata = OtapPdata::new_default(OtapPayload::OtlpBytes(
                OtlpProtoBytes::ExportLogsRequest(req_bytes.clone().into()),
            ))
            .test_subscribe_to(
                Interests::ACKS | Interests::NACKS,
                TestCallData::default().into(),
                123,
            );
            pdata_tx.send(pdata).await.unwrap();
            // Wait for NACK since server is down
            wait_for_ack_or_nack(
                &mut pipeline_completion_msg_rx,
                false,
                123,
                "when server is down",
            )
            .await
            .expect("Expected Nack when server down");

            // wait a bit before starting the server. This will ensure the exporter no-longer exits
            // when start is called if the endpoint can't be reached
            tokio::time::sleep(Duration::from_millis(100)).await;
            server_startup_sender.send(true).await.unwrap();
            _ = server_startup_ack_receiver.recv().await.unwrap();

            // send a pdata
            let pdata = OtapPdata::new_default(OtapPayload::OtlpBytes(
                OtlpProtoBytes::ExportLogsRequest(req_bytes.clone().into()),
            ))
            .test_subscribe_to(
                Interests::ACKS | Interests::NACKS,
                TestCallData::default().into(),
                123,
            );
            pdata_tx.send(pdata).await.unwrap();
            // ensure server got request
            _ = req_receiver.recv().await.unwrap();
            // Wait for ACK since server is up
            wait_for_ack_or_nack(
                &mut pipeline_completion_msg_rx,
                true,
                123,
                "when server is up",
            )
            .await
            .expect("Expected Ack when server up");

            // stop the server
            server_shutdown_signal1.send(true).unwrap();
            _ = server_shutdown_ack_receiver.recv().await.unwrap();

            // send a request while the server isn't running and check that we still handle it correctly
            let pdata = OtapPdata::new_default(OtapPayload::OtlpBytes(
                OtlpProtoBytes::ExportLogsRequest(req_bytes.clone().into()),
            ))
            .test_subscribe_to(
                Interests::ACKS | Interests::NACKS,
                TestCallData::default().into(),
                123,
            );
            pdata_tx.send(pdata).await.unwrap();
            // Wait for NACK since server is down again
            wait_for_ack_or_nack(
                &mut pipeline_completion_msg_rx,
                false,
                123,
                "when server is down again",
            )
            .await
            .expect("Expected Nack when server down again");

            // restart the server
            server_startup_sender.send(true).await.unwrap();
            _ = server_startup_ack_receiver.recv().await.unwrap();

            // send another pdata. This ensures the client can reconnect after it was shut down
            let pdata = OtapPdata::new_default(OtapPayload::OtlpBytes(
                OtlpProtoBytes::ExportLogsRequest(req_bytes.clone().into()),
            ))
            .test_subscribe_to(
                Interests::ACKS | Interests::NACKS,
                TestCallData::default().into(),
                123,
            );
            pdata_tx.send(pdata).await.unwrap();
            _ = req_receiver.recv().await.unwrap();
            // Wait for ACK after reconnect
            wait_for_ack_or_nack(
                &mut pipeline_completion_msg_rx,
                true,
                123,
                "after reconnect",
            )
            .await
            .expect("Expected Ack after reconnect");

            // check the metrics:
            control_sender
                .send(NodeControlMsg::CollectTelemetry {
                    metrics_reporter: metrics_reporter.clone(),
                })
                .await
                .unwrap();
            let mut logs_exported_count = 0;
            let mut logs_failed_count = 0;
            for _ in 0..2 {
                let metrics = metrics_receiver.recv_async().await.unwrap();
                if metrics.descriptor().name == "exporter.exports"
                    && metrics.measurement_attribute_value("signal") == Some("logs")
                {
                    match metrics.measurement_attribute_value("outcome") {
                        Some("success") => {
                            logs_exported_count = metrics.get_metrics()[0].to_u64_lossy();
                        }
                        Some("failure") => {
                            logs_failed_count = metrics.get_metrics()[0].to_u64_lossy();
                        }
                        _ => {}
                    }
                }
            }
            assert_eq!(logs_exported_count, 2);
            assert_eq!(logs_failed_count, 2);

            control_sender
                .send(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_millis(10),
                    reason: "shutting down".into(),
                })
                .await
                .unwrap();

            server_shutdown_signal2.send(true).unwrap();

            Ok(())
        }

        async fn run_server(
            listening_addr: String,
            startup_ack_sender: tokio::sync::mpsc::Sender<bool>,
            shutdown_signal: tokio::sync::oneshot::Receiver<bool>,
            req_sender: tokio::sync::mpsc::Sender<OTLPData>,
        ) {
            let listening_addr: SocketAddr = listening_addr.to_string().parse().unwrap();
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            let tcp_stream = TcpListenerStream::new(tcp_listener);

            let logs_service = LogsServiceServer::new(LogsServiceMock::new(req_sender));

            Server::builder()
                .add_service(logs_service)
                .serve_with_incoming_shutdown(tcp_stream, async {
                    startup_ack_sender.send(true).await.unwrap();
                    let _ = shutdown_signal.await;
                })
                .await
                .expect("uh oh server failed");
        }

        let server_handle = tokio_rt.spawn(async move {
            // start the server when the signal is received
            let listening_addr = format!("{grpc_addr}:{grpc_port}");
            _ = server_startup_receiver.recv().await.unwrap();
            run_server(
                listening_addr.clone(),
                server_start_ack_sender.clone(),
                shutdown_signal1,
                req_sender.clone(),
            )
            .await;

            // ack server shutdown for first time
            server_shutdown_ack_sender.send(true).await.unwrap();

            // when the server shuts down, wait until it should restart & restart it
            _ = server_startup_receiver.recv().await.unwrap();
            run_server(
                listening_addr.clone(),
                server_start_ack_sender.clone(),
                shutdown_signal2,
                req_sender.clone(),
            )
            .await;
        });

        let (metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(3);

        let (exporter_result, test_drive_result) = tokio_rt.block_on(async move {
            tokio::join!(
                start_exporter(
                    exporter,
                    runtime_ctrl_msg_tx,
                    pipeline_completion_msg_tx,
                    metrics_reporter.clone(),
                ),
                drive_test(
                    server_startup_sender,
                    server_start_ack_receiver,
                    server_shutdown_ack_receiver,
                    shutdown_sender1,
                    shutdown_sender2,
                    pdata_tx,
                    control_sender,
                    pipeline_completion_msg_rx,
                    req_receiver,
                    metrics_rx,
                    metrics_reporter
                )
            )
        });

        // assert no error
        exporter_result.unwrap();
        test_drive_result.unwrap();

        tokio_rt
            .block_on(server_handle)
            .expect("server shutdown success");
    }

    /// Helper builds a [`tonic::Status`] with the given code, no details.
    fn status_with_code(code: Code) -> tonic::Status {
        tonic::Status::new(code, "test error")
    }

    /// Scenario: A dispatched OTLP gRPC request returns either success or a backend status error.
    /// Guarantees: Only the RPC error produces a diagnostic type, independently of notification delivery.
    #[test]
    fn export_error_type_is_derived_from_the_rpc_result() {
        assert_eq!(export_error_type(&Ok(())), None);
        assert_eq!(
            export_error_type(&Err(status_with_code(Code::Unavailable))),
            Some(OtlpGrpcExporterErrorType::Unavailable)
        );
    }

    /// Helper builds a [`tonic::Status`] carrying a `RetryInfo` in its
    /// `grpc-status-details-bin` trailer, as a real server would.
    fn status_with_retry_info(code: Code) -> tonic::Status {
        let retry_info = RetryInfo {
            retry_delay: Some(prost_types::Duration {
                seconds: 5,
                nanos: 0,
            }),
        };
        let mut retry_info_bytes = Vec::new();
        retry_info
            .encode(&mut retry_info_bytes)
            .expect("encode RetryInfo");

        let any = prost_types::Any {
            type_url: RETRY_INFO_TYPE_URL.to_string(),
            value: retry_info_bytes,
        };
        let rpc_status = RpcStatus {
            code: code as i32,
            message: "resource exhausted".to_string(),
            details: vec![any],
        };
        let mut detail_bytes = Vec::new();
        rpc_status
            .encode(&mut detail_bytes)
            .expect("encode RpcStatus");

        tonic::Status::with_details(code, "resource exhausted", detail_bytes.into())
    }

    #[test]
    fn test_retryable_grpc_codes() {
        // These codes MUST be retryable per the OTLP spec table, plus
        // RESOURCE_EXHAUSTED (advisory RetryInfo) and UNKNOWN (client-side
        // readiness/transport failures).
        let retryable_codes = [
            Code::Cancelled,
            Code::DeadlineExceeded,
            Code::Aborted,
            Code::OutOfRange,
            Code::Unavailable,
            Code::DataLoss,
            Code::ResourceExhausted,
            Code::Unknown,
        ];

        for code in retryable_codes {
            let status = status_with_code(code);
            assert!(
                is_retryable_grpc_status(&status),
                "expected code {code:?} to be retryable"
            );
        }
    }

    #[test]
    fn test_non_retryable_grpc_codes() {
        // These codes MUST NOT be retryable per the OTLP spec table.
        let non_retryable_codes = [
            Code::InvalidArgument,
            Code::NotFound,
            Code::AlreadyExists,
            Code::PermissionDenied,
            Code::Unauthenticated,
            Code::FailedPrecondition,
            Code::Unimplemented,
            Code::Internal,
        ];

        for code in non_retryable_codes {
            let status = status_with_code(code);
            assert!(
                !is_retryable_grpc_status(&status),
                "expected code {code:?} to be non-retryable"
            );
        }
    }

    #[test]
    fn test_resource_exhausted_is_retryable_without_retry_info() {
        // RESOURCE_EXHAUSTED is always retryable; the RetryInfo detail is only
        // advisory, so its absence must not make the failure permanent.
        let status = status_with_code(Code::ResourceExhausted);
        assert!(
            retry_after(&status).is_none(),
            "status carries no RetryInfo detail"
        );
        assert!(
            is_retryable_grpc_status(&status),
            "RESOURCE_EXHAUSTED without RetryInfo should still be retryable"
        );
    }

    #[test]
    fn test_resource_exhausted_is_retryable_with_retry_info() {
        let status = status_with_retry_info(Code::ResourceExhausted);
        assert_eq!(
            retry_after(&status),
            Some(prost_types::Duration {
                seconds: 5,
                nanos: 0,
            }),
            "status carries a RetryInfo detail with the expected delay"
        );
        assert!(
            is_retryable_grpc_status(&status),
            "RESOURCE_EXHAUSTED with RetryInfo should be retryable"
        );
    }

    #[test]
    fn test_retry_after_none_for_empty_details() {
        // Details bytes present but contain an RpcStatus with no Any entries.
        let rpc_status = RpcStatus {
            code: Code::ResourceExhausted as i32,
            message: "exhausted".to_string(),
            details: vec![],
        };
        let mut detail_bytes = Vec::new();
        rpc_status
            .encode(&mut detail_bytes)
            .expect("encode RpcStatus");

        let status =
            tonic::Status::with_details(Code::ResourceExhausted, "exhausted", detail_bytes.into());
        assert!(
            retry_after(&status).is_none(),
            "empty details should not report a RetryInfo hint"
        );
    }

    #[test]
    fn test_retry_after_none_for_non_retry_info_detail() {
        use prost::Message;

        // Details bytes contain an Any with a different type URL.
        let any = prost_types::Any {
            type_url: "type.googleapis.com/google.rpc.BadRequest".to_string(),
            value: vec![],
        };
        let rpc_status = RpcStatus {
            code: Code::ResourceExhausted as i32,
            message: "exhausted".to_string(),
            details: vec![any],
        };
        let mut detail_bytes = Vec::new();
        rpc_status
            .encode(&mut detail_bytes)
            .expect("encode RpcStatus");

        let status =
            tonic::Status::with_details(Code::ResourceExhausted, "exhausted", detail_bytes.into());
        assert!(
            retry_after(&status).is_none(),
            "a non-RetryInfo detail should not report a RetryInfo hint"
        );
    }

    #[test]
    fn test_retry_after_none_for_malformed_details() {
        // Feed garbage bytes as details - should not crash, should report no hint.
        let status = tonic::Status::with_details(
            Code::ResourceExhausted,
            "exhausted",
            Bytes::from_static(b"not valid protobuf"),
        );
        assert!(
            retry_after(&status).is_none(),
            "malformed details should not report a RetryInfo hint"
        );
    }

    #[test]
    fn test_ok_code_is_not_retryable() {
        let status = status_with_code(Code::Ok);
        assert!(
            !is_retryable_grpc_status(&status),
            "OK should not be retryable"
        );
    }

    #[test]
    fn test_format_retry_delay() {
        assert_eq!(
            format_retry_delay(&prost_types::Duration {
                seconds: 5,
                nanos: 0,
            }),
            "5000ms"
        );
        assert_eq!(
            format_retry_delay(&prost_types::Duration {
                seconds: 0,
                nanos: 250_000_000,
            }),
            "250ms"
        );
        assert_eq!(
            format_retry_delay(&prost_types::Duration {
                seconds: 1,
                nanos: 500_000_000,
            }),
            "1500ms"
        );
    }

    /// A mock `LogsService` that always returns the configured gRPC error.
    struct ErrorLogsServiceMock {
        code: Code,
        message: String,
        /// Optional serialized `google.rpc.Status` details bytes for
        /// `grpc-status-details-bin`.
        detail_bytes: Option<Bytes>,
    }

    #[tonic::async_trait]
    impl otap_df_pdata::proto::opentelemetry::collector::logs::v1::logs_service_server::LogsService
        for ErrorLogsServiceMock
    {
        async fn export(
            &self,
            _request: tonic::Request<ExportLogsServiceRequest>,
        ) -> Result<
            tonic::Response<
                otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceResponse,
            >,
            tonic::Status,
        > {
            if let Some(details) = &self.detail_bytes {
                Err(tonic::Status::with_details(
                    self.code,
                    &self.message,
                    details.clone(),
                ))
            } else {
                Err(tonic::Status::new(self.code, &self.message))
            }
        }
    }

    /// Runs an integration test that sends a logs payload to the gRPC exporter
    /// backed by a mock server returning the given status code. Returns the
    /// `NackMsg.permanent` value observed.
    fn run_grpc_error_status_test(code: Code, detail_bytes: Option<Bytes>) -> bool {
        run_grpc_error_status_test_with_provider(code, detail_bytes, None)
    }

    /// As [`run_grpc_error_status_test`], but with an optional bound bearer
    /// token provider, so a status code whose classification depends on whether
    /// the credential is refreshable can be exercised both ways.
    fn run_grpc_error_status_test_with_provider(
        code: Code,
        detail_bytes: Option<Bytes>,
        token_provider: Option<Box<dyn BearerTokenProvider>>,
    ) -> bool {
        use otap_df_pdata::proto::opentelemetry::collector::logs::v1::logs_service_server::LogsServiceServer;

        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");

        let tokio_rt = Runtime::new().unwrap();

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTLP_EXPORTER_URN));

        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let node_id = test_node(test_runtime.config().name.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let mut exporter = ExporterWrapper::local(
            OTLPExporter {
                config: Config {
                    grpc: GrpcClientSettings {
                        grpc_endpoint: grpc_endpoint.clone(),
                        connect_timeout: Duration::from_millis(500),
                        ..Default::default()
                    },
                    max_in_flight: 1,
                    num_connections: default_num_connections(),
                },
                metrics: OtlpGrpcExporterMetrics::register(&pipeline_ctx),
                token_provider,
            },
            node_id.clone(),
            node_config,
            test_runtime.config(),
        );

        let control_sender = exporter.control_sender();
        let (pdata_tx, pdata_rx) = create_not_send_channel::<OtapPdata>(1);
        let pdata_tx = Sender::Local(LocalSender::mpsc(pdata_tx));
        let pdata_rx = Receiver::Local(LocalReceiver::mpsc(pdata_rx));
        let (runtime_ctrl_msg_tx, _runtime_ctrl_msg_rx) = runtime_ctrl_msg_channel(2);
        let (pipeline_completion_msg_tx, pipeline_completion_msg_rx) =
            pipeline_completion_msg_channel(2);
        exporter
            .set_pdata_receiver(node_id.clone(), pdata_rx)
            .expect("Failed to set PData Receiver");

        let (metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let _ = metrics_rx; // not inspected in this test

        // Start the mock server that always returns the given error code.
        let listening_addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let mock_service = ErrorLogsServiceMock {
            code,
            message: format!("mock error: {code:?}"),
            detail_bytes,
        };
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        let server_handle = tokio_rt.spawn(async move {
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            let tcp_stream = TcpListenerStream::new(tcp_listener);
            Server::builder()
                .add_service(LogsServiceServer::new(mock_service))
                .serve_with_incoming_shutdown(tcp_stream, async {
                    let _ = shutdown_rx.await;
                })
                .await
                .expect("server failed");
        });

        async fn start_exporter(
            exporter: ExporterWrapper<OtapPdata>,
            runtime_ctrl_msg_tx: RuntimeCtrlMsgSender<OtapPdata>,
            pipeline_completion_msg_tx: PipelineCompletionMsgSender<OtapPdata>,
            metrics_reporter: MetricsReporter,
        ) -> Result<(), Error> {
            exporter
                .start(
                    runtime_ctrl_msg_tx,
                    pipeline_completion_msg_tx,
                    metrics_reporter,
                    Interests::empty(),
                )
                .await
                .map(|_| ())
        }

        async fn drive_test(
            pdata_tx: Sender<OtapPdata>,
            control_sender: Sender<NodeControlMsg<OtapPdata>>,
            mut pipeline_completion_msg_rx: otap_df_engine::control::PipelineCompletionMsgReceiver<
                OtapPdata,
            >,
            shutdown_tx: tokio::sync::oneshot::Sender<()>,
        ) -> bool {
            use prost::Message;

            // Give the server a moment to bind.
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Send a logs payload.
            let req = ExportLogsServiceRequest::default();
            let mut req_bytes = vec![];
            req.encode(&mut req_bytes).unwrap();
            let pdata = OtapPdata::new_default(OtapPayload::OtlpBytes(
                OtlpProtoBytes::ExportLogsRequest(Bytes::from(req_bytes)),
            ))
            .test_subscribe_to(
                Interests::ACKS | Interests::NACKS,
                TestCallData::default().into(),
                123,
            );
            pdata_tx.send(pdata).await.unwrap();

            // Wait for the NACK.
            let permanent = timeout(Duration::from_secs(5), async {
                match pipeline_completion_msg_rx.recv().await {
                    Ok(PipelineCompletionMsg::DeliverNack { nack }) => nack.permanent,
                    Ok(PipelineCompletionMsg::DeliverAck { .. }) => {
                        panic!("expected NACK but got ACK");
                    }
                    Err(_) => panic!("pipeline completion channel closed"),
                }
            })
            .await
            .expect("timed out waiting for NACK");

            // Shut everything down.
            control_sender
                .send(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_millis(100),
                    reason: "test done".into(),
                })
                .await
                .unwrap();
            shutdown_tx.send(()).unwrap();

            permanent
        }

        let (_, permanent) = tokio_rt.block_on(async move {
            tokio::join!(
                start_exporter(
                    exporter,
                    runtime_ctrl_msg_tx,
                    pipeline_completion_msg_tx,
                    metrics_reporter,
                ),
                drive_test(
                    pdata_tx,
                    control_sender,
                    pipeline_completion_msg_rx,
                    shutdown_tx,
                )
            )
        });

        tokio_rt.block_on(server_handle).expect("server join");
        permanent
    }

    #[test]
    fn test_unavailable_produces_non_permanent_nack() {
        let permanent = run_grpc_error_status_test(Code::Unavailable, None);
        assert!(
            !permanent,
            "UNAVAILABLE should produce a non-permanent (retryable) NACK"
        );
    }

    #[test]
    fn test_invalid_argument_produces_permanent_nack() {
        let permanent = run_grpc_error_status_test(Code::InvalidArgument, None);
        assert!(
            permanent,
            "INVALID_ARGUMENT should produce a permanent NACK"
        );
    }

    #[test]
    fn test_internal_produces_permanent_nack() {
        let permanent = run_grpc_error_status_test(Code::Internal, None);
        assert!(permanent, "INTERNAL should produce a permanent NACK");
    }

    #[test]
    fn test_cancelled_produces_non_permanent_nack() {
        let permanent = run_grpc_error_status_test(Code::Cancelled, None);
        assert!(
            !permanent,
            "CANCELLED should produce a non-permanent (retryable) NACK"
        );
    }

    #[test]
    fn test_resource_exhausted_without_retry_info_produces_non_permanent_nack() {
        let permanent = run_grpc_error_status_test(Code::ResourceExhausted, None);
        assert!(
            !permanent,
            "RESOURCE_EXHAUSTED without RetryInfo should still produce a non-permanent (retryable) NACK"
        );
    }

    #[test]
    fn test_resource_exhausted_with_retry_info_produces_non_permanent_nack() {
        let retry_info = RetryInfo {
            retry_delay: Some(prost_types::Duration {
                seconds: 5,
                nanos: 0,
            }),
        };
        let mut retry_info_bytes = Vec::new();
        retry_info
            .encode(&mut retry_info_bytes)
            .expect("encode RetryInfo");

        let any = prost_types::Any {
            type_url: RETRY_INFO_TYPE_URL.to_string(),
            value: retry_info_bytes,
        };
        let rpc_status = RpcStatus {
            code: Code::ResourceExhausted as i32,
            message: "resource exhausted".to_string(),
            details: vec![any],
        };
        let mut detail_bytes = Vec::new();
        rpc_status
            .encode(&mut detail_bytes)
            .expect("encode RpcStatus");

        let permanent =
            run_grpc_error_status_test(Code::ResourceExhausted, Some(detail_bytes.into()));
        assert!(
            !permanent,
            "RESOURCE_EXHAUSTED with RetryInfo should produce a non-permanent (retryable) NACK"
        );
    }

    /// Scenario: the server rejects an export with `UNAUTHENTICATED` while no
    /// bearer token provider is bound, so the credential is static config.
    /// Guarantees: the NACK is permanent, because no refresh can occur and
    /// retrying would replay the same rejected credential forever.
    #[test]
    fn unauthenticated_without_a_bound_provider_produces_permanent_nack() {
        let permanent = run_grpc_error_status_test(Code::Unauthenticated, None);
        assert!(
            permanent,
            "UNAUTHENTICATED with a static credential should produce a permanent NACK"
        );
    }

    /// Scenario: the server rejects an export with `UNAUTHENTICATED` while a
    /// bearer token provider is bound and has published a token.
    /// Guarantees: the token generation stamped into the request metadata
    /// survives all the way to the completion, so the failure is classified as
    /// refreshable and the batch is NACK'd retryably instead of being dropped
    /// because a token lapsed or a refresh raced.
    #[test]
    fn unauthenticated_with_a_bound_provider_produces_retryable_nack() {
        let permanent = run_grpc_error_status_test_with_provider(
            Code::Unauthenticated,
            None,
            Some(Box::new(MockTokenProvider::new("provider-token"))),
        );
        assert!(
            !permanent,
            "UNAUTHENTICATED with a refreshable token should produce a retryable NACK"
        );
    }

    // ---- build_grpc_metadata unit tests ----------------------------------------

    /// Helper: Creates an [`EffectHandler`] with an optional propagation policy set.
    fn make_effect_handler_with_policy(
        policy: Option<HeaderPropagationPolicy>,
    ) -> EffectHandler<OtapPdata> {
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let node_id = test_node("test-exporter");
        let mut handler = EffectHandler::new(node_id, metrics_reporter);
        handler.set_propagation_policy(policy);
        handler
    }

    /// Helper: Creates a [`Context`] that carries the given transport headers.
    fn context_with_headers(headers: TransportHeaders) -> Context {
        let pdata = OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into())
            .with_transport_headers(headers);
        let (context, _) = pdata.into_parts();
        context
    }

    /// Helper: Creates a [`Context`] without any transport headers.
    fn context_without_headers() -> Context {
        let pdata = OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into());
        let (context, _) = pdata.into_parts();
        context
    }

    /// Helper: Propagation policy that propagates all captured headers.
    fn propagate_all_policy() -> HeaderPropagationPolicy {
        HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![],
        )
    }

    #[test]
    fn test_build_grpc_metadata_returns_none_without_policy() {
        let handler = make_effect_handler_with_policy(None);
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text("x-tenant-id", "x-tenant-id", b"acme"));
        let context = context_with_headers(headers);

        let result = build_grpc_metadata(&handler, &context, None, None);
        assert!(result.is_none(), "should return None when no policy is set");
    }

    #[test]
    fn test_build_grpc_metadata_returns_none_without_headers() {
        let handler = make_effect_handler_with_policy(Some(propagate_all_policy()));
        let context = context_without_headers();

        let result = build_grpc_metadata(&handler, &context, None, None);
        assert!(
            result.is_none(),
            "should return None when context has no transport headers"
        );
    }

    #[test]
    fn test_build_grpc_metadata_propagates_text_headers() {
        let handler = make_effect_handler_with_policy(Some(propagate_all_policy()));

        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text(
            "x-tenant-id",
            "X-Tenant-Id",
            b"tenant-abc-123",
        ));
        headers.push(TransportHeader::text(
            "x-request-id",
            "X-Request-Id",
            b"req-xyz-789",
        ));
        let context = context_with_headers(headers);

        let metadata = build_grpc_metadata(&handler, &context, None, None)
            .expect("should produce metadata for text headers");

        let tenant = metadata
            .get("x-tenant-id")
            .expect("x-tenant-id should be present");
        assert_eq!(tenant.to_str().unwrap(), "tenant-abc-123");

        let request_id = metadata
            .get("x-request-id")
            .expect("x-request-id should be present");
        assert_eq!(request_id.to_str().unwrap(), "req-xyz-789");
    }

    #[test]
    fn test_build_grpc_metadata_drops_filtered_headers() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![PropagationOverride {
                match_rule: PropagationMatch {
                    stored_names: vec!["authorization".to_string()],
                },
                action: PropagationAction::Drop,
                name: None,
                on_error: None,
            }],
        );
        let handler = make_effect_handler_with_policy(Some(policy));

        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text("x-tenant-id", "X-Tenant-Id", b"acme"));
        headers.push(TransportHeader::text(
            "authorization",
            "Authorization",
            b"Bearer secret-token",
        ));
        let context = context_with_headers(headers);

        let metadata = build_grpc_metadata(&handler, &context, None, None)
            .expect("should produce metadata (authorization dropped, x-tenant-id remains)");

        assert!(
            metadata.get("x-tenant-id").is_some(),
            "x-tenant-id should be propagated"
        );
        assert!(
            metadata.get("authorization").is_none(),
            "authorization should be dropped by the override"
        );
    }

    #[test]
    fn test_build_grpc_metadata_propagates_binary_headers() {
        let handler = make_effect_handler_with_policy(Some(propagate_all_policy()));

        let binary_value: Vec<u8> = vec![0x00, 0x01, 0xFF, 0xFE, 0x80, 0x7F];
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::binary(
            "trace-context-bin",
            "trace-context-bin",
            binary_value.clone(),
        ));
        let context = context_with_headers(headers);

        let metadata = build_grpc_metadata(&handler, &context, None, None)
            .expect("should produce metadata for binary headers");

        let bin_val = metadata
            .get_bin("trace-context-bin")
            .expect("trace-context-bin should be present as binary metadata");
        assert_eq!(
            bin_val.to_bytes().unwrap(),
            binary_value.as_slice(),
            "binary value should be preserved"
        );
    }

    #[test]
    fn test_build_grpc_metadata_appends_bin_suffix_for_binary_headers() {
        let handler = make_effect_handler_with_policy(Some(propagate_all_policy()));

        let binary_value: Vec<u8> = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let mut headers = TransportHeaders::new();
        // Wire name does NOT end with -bin; build_grpc_metadata should add the suffix.
        headers.push(TransportHeader::binary(
            "custom-binary",
            "custom-binary",
            binary_value.clone(),
        ));
        let context = context_with_headers(headers);

        let metadata = build_grpc_metadata(&handler, &context, None, None)
            .expect("should produce metadata for binary header without -bin suffix");

        let bin_val = metadata
            .get_bin("custom-binary-bin")
            .expect("custom-binary-bin should be present (suffix appended)");
        assert_eq!(bin_val.to_bytes().unwrap(), binary_value.as_slice());
    }

    #[test]
    fn test_build_grpc_metadata_preserves_duplicate_headers() {
        let handler = make_effect_handler_with_policy(Some(propagate_all_policy()));

        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text(
            "x-forwarded-for",
            "X-Forwarded-For",
            b"10.0.0.1",
        ));
        headers.push(TransportHeader::text(
            "x-forwarded-for",
            "X-Forwarded-For",
            b"192.168.1.1",
        ));
        headers.push(TransportHeader::text(
            "x-forwarded-for",
            "X-Forwarded-For",
            b"172.16.0.1",
        ));
        let context = context_with_headers(headers);

        let metadata = build_grpc_metadata(&handler, &context, None, None)
            .expect("should produce metadata with duplicate headers");

        let values: Vec<&str> = metadata
            .get_all("x-forwarded-for")
            .iter()
            .map(|v| v.to_str().unwrap())
            .collect();
        assert_eq!(
            values,
            vec!["10.0.0.1", "192.168.1.1", "172.16.0.1"],
            "all duplicate header values should be preserved in order"
        );
    }

    #[test]
    fn test_build_grpc_metadata_returns_none_when_all_dropped() {
        // Policy that drops everything (selector = None means no headers are selected).
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::None,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![],
        );
        let handler = make_effect_handler_with_policy(Some(policy));

        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text("x-tenant-id", "X-Tenant-Id", b"acme"));
        let context = context_with_headers(headers);

        let result = build_grpc_metadata(&handler, &context, None, None);
        assert!(
            result.is_none(),
            "should return None when policy drops all headers"
        );
    }

    #[test]
    fn test_build_grpc_metadata_static_only() {
        // No propagation policy, but static headers are configured: the static
        // template alone must be applied to the request.
        let handler = make_effect_handler_with_policy(None);
        let context = context_without_headers();

        let mut headers = HashMap::new();
        _ = headers.insert("authorization".to_string(), "Basic abc123".into());
        let static_metadata = GrpcClientSettings {
            headers,
            ..Default::default()
        }
        .build_static_metadata()
        .expect("static metadata should be present");

        let metadata = build_grpc_metadata(&handler, &context, Some(&static_metadata), None)
            .expect("should produce metadata from static headers alone");
        assert_eq!(
            metadata.get("authorization").unwrap().to_str().unwrap(),
            "Basic abc123"
        );
    }

    #[test]
    fn test_build_grpc_metadata_static_and_propagation_coexist() {
        // Static headers and propagated transport headers must both appear.
        let handler = make_effect_handler_with_policy(Some(propagate_all_policy()));

        let mut transport = TransportHeaders::new();
        transport.push(TransportHeader::text(
            "x-tenant-id",
            "X-Tenant-Id",
            b"tenant-abc",
        ));
        let context = context_with_headers(transport);

        let mut static_headers = HashMap::new();
        _ = static_headers.insert("authorization".to_string(), "Basic abc123".into());
        let static_metadata = GrpcClientSettings {
            headers: static_headers,
            ..Default::default()
        }
        .build_static_metadata()
        .expect("static metadata should be present");

        let metadata = build_grpc_metadata(&handler, &context, Some(&static_metadata), None)
            .expect("should merge static and propagated headers");
        assert_eq!(
            metadata.get("authorization").unwrap().to_str().unwrap(),
            "Basic abc123",
            "static header must be present"
        );
        assert_eq!(
            metadata.get("x-tenant-id").unwrap().to_str().unwrap(),
            "tenant-abc",
            "propagated header must be present"
        );
    }

    #[test]
    fn test_build_grpc_metadata_static_wins_over_propagated_collision() {
        // When a propagated header collides with a statically configured one,
        // the static value must win and the propagated duplicate must be dropped
        // so we never send two `authorization` values on the wire.
        let handler = make_effect_handler_with_policy(Some(propagate_all_policy()));

        let mut transport = TransportHeaders::new();
        transport.push(TransportHeader::text(
            "authorization",
            "Authorization",
            b"Bearer propagated",
        ));
        let context = context_with_headers(transport);

        let mut static_headers = HashMap::new();
        _ = static_headers.insert("authorization".to_string(), "Basic static".into());
        let static_metadata = GrpcClientSettings {
            headers: static_headers,
            ..Default::default()
        }
        .build_static_metadata()
        .expect("static metadata should be present");

        let metadata = build_grpc_metadata(&handler, &context, Some(&static_metadata), None)
            .expect("should produce metadata");

        let values: Vec<&str> = metadata
            .get_all("authorization")
            .iter()
            .map(|v| v.to_str().unwrap())
            .collect();
        assert_eq!(
            values,
            vec!["Basic static"],
            "static header must win and the propagated duplicate must be dropped"
        );
    }

    /// Scenario: a bearer token is cached but neither static headers nor a
    /// propagation policy are configured.
    /// Guarantees: the exporter still builds request metadata carrying the
    /// `authorization` header, so the zero-alloc fast path cannot silently drop
    /// the credential and send an unauthenticated request.
    #[test]
    fn build_grpc_metadata_carries_the_bearer_token_alone() {
        let handler = make_effect_handler_with_policy(None);
        let context = context_without_headers();

        let metadata = build_grpc_metadata(
            &handler,
            &context,
            None,
            Some(HeaderValue::from_static("Bearer refreshed")),
        )
        .expect("a cached bearer token must produce request metadata");

        assert_eq!(
            metadata.get("authorization").unwrap().to_str().unwrap(),
            "Bearer refreshed"
        );
    }

    /// Scenario: a bearer token provider is bound while the config also carries
    /// a static `authorization` header and an inbound `authorization` is
    /// propagated.
    /// Guarantees: exactly one `authorization` is sent and it is the refreshed
    /// token, so a stale configured credential can neither override nor be
    /// duplicated alongside the token the provider is actively refreshing.
    #[test]
    fn build_grpc_metadata_bearer_token_replaces_other_authorization_headers() {
        let handler = make_effect_handler_with_policy(Some(propagate_all_policy()));

        let mut transport = TransportHeaders::new();
        transport.push(TransportHeader::text(
            "authorization",
            "Authorization",
            b"Bearer propagated",
        ));
        let context = context_with_headers(transport);

        let mut static_headers = HashMap::new();
        _ = static_headers.insert("authorization".to_string(), "Basic static".into());
        let static_metadata = GrpcClientSettings {
            headers: static_headers,
            ..Default::default()
        }
        .build_static_metadata()
        .expect("static metadata should be present");

        let metadata = build_grpc_metadata(
            &handler,
            &context,
            Some(&static_metadata),
            Some(HeaderValue::from_static("Bearer refreshed")),
        )
        .expect("should produce metadata");

        let values: Vec<&str> = metadata
            .get_all("authorization")
            .iter()
            .map(|v| v.to_str().unwrap())
            .collect();
        assert_eq!(
            values,
            vec!["Bearer refreshed"],
            "the refreshed bearer token must be the only authorization sent"
        );
    }

    /// Scenario: the cached bearer header (which the adapter marks sensitive) is
    /// carried into gRPC request metadata, which is built by round-tripping an
    /// `http::HeaderMap` through `MetadataMap::from_headers`.
    /// Guarantees: the sensitivity flag survives that round-trip, so tonic keeps
    /// the credential out of the HPACK dynamic table and out of `Debug` output
    /// instead of silently indexing it on every request.
    #[test]
    fn build_grpc_metadata_keeps_the_bearer_token_sensitive() {
        let handler = make_effect_handler_with_policy(Some(propagate_all_policy()));
        let context = context_without_headers();

        let mut token = HeaderValue::from_static("Bearer refreshed");
        token.set_sensitive(true);

        let metadata = build_grpc_metadata(&handler, &context, None, Some(token))
            .expect("a cached bearer token must produce request metadata");

        assert!(
            metadata
                .get("authorization")
                .expect("the token must be present")
                .is_sensitive(),
            "the bearer token must stay sensitive so it is never HPACK-indexed"
        );
    }

    /// Scenario: a bearer token provider is bound and the server answers an
    /// export with `UNAUTHENTICATED`.
    /// Guarantees: the failure is classified as an auth failure, which is what
    /// makes the NACK retryable and invalidates the token generation, so a
    /// lapsed or raced token costs a retry rather than dropping the batch.
    #[test]
    fn unauthenticated_is_an_auth_failure_when_a_provider_is_bound() {
        let result = Err(tonic::Status::unauthenticated("token expired"));

        assert!(is_auth_failure(&result, true));
    }

    /// Scenario: the server answers `UNAUTHENTICATED` but no bearer token
    /// provider is bound, so the credential is static configuration.
    /// Guarantees: the failure stays permanent, because no refresh can occur and
    /// retrying would loop on the same rejected credential.
    #[test]
    fn unauthenticated_is_permanent_without_a_bound_provider() {
        let result = Err(tonic::Status::unauthenticated("token expired"));

        assert!(!is_auth_failure(&result, false));
    }

    /// Scenario: a bearer token provider is bound and the server answers
    /// `PERMISSION_DENIED`.
    /// Guarantees: the failure is not treated as an auth failure, so the batch
    /// is NACK'd permanently and a valid token is not needlessly discarded for a
    /// scope or permission problem a refresh cannot fix.
    #[test]
    fn permission_denied_is_not_an_auth_failure() {
        let result = Err(tonic::Status::permission_denied("scope missing"));

        assert!(!is_auth_failure(&result, true));
    }

    /// Scenario: an export succeeds while a bearer token provider is bound.
    /// Guarantees: no token generation is reported as rejected, so a healthy
    /// export never invalidates the cached token and stalls intake.
    #[test]
    fn a_successful_export_is_not_an_auth_failure() {
        assert!(!is_auth_failure(&Ok(()), true));
    }
}
