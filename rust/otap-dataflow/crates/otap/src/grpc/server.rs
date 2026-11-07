// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared Tonic server tuning and connection metadata.

use super::settings::GrpcServerSettings;
use ::http::Extensions;
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic::transport::server::{TcpConnectInfo, TlsConnectInfo};

/// Extracts a peer address from plaintext or TLS Tonic connection information.
///
/// Returns `None` for transports without TCP connection information.
#[must_use]
pub fn peer_addr_from_extensions(extensions: &Extensions) -> Option<SocketAddr> {
    extensions
        .get::<TcpConnectInfo>()
        .and_then(|info| info.remote_addr())
        .or_else(|| {
            extensions
                .get::<TlsConnectInfo<TcpConnectInfo>>()
                .and_then(|info| info.get_ref().remote_addr())
        })
}

/// Applies transport concurrency, load shedding, HTTP/2 windows, keepalive, and timeout.
///
/// The caller configures stream limits, message sizes, compression, and TLS.
pub fn apply_server_tuning<L>(builder: Server<L>, config: &GrpcServerSettings) -> Server<L> {
    let transport_limit = config
        .transport_concurrency_limit
        .filter(|&limit| limit != 0)
        .unwrap_or(config.max_concurrent_requests)
        .max(1);

    let mut builder = builder
        .concurrency_limit_per_connection(transport_limit)
        .load_shed(config.load_shed)
        .initial_stream_window_size(config.initial_stream_window_size)
        .initial_connection_window_size(config.initial_connection_window_size)
        .max_frame_size(config.max_frame_size)
        .http2_adaptive_window(Some(config.http2_adaptive_window))
        .http2_keepalive_interval(config.http2_keepalive_interval)
        .http2_keepalive_timeout(config.http2_keepalive_timeout);

    if let Some(timeout) = config.timeout {
        builder = builder.timeout(timeout);
    }

    builder
}
