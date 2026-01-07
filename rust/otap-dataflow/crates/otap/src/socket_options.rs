// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! TCP socket option helpers shared across servers/clients.

use socket2::{Socket, TcpKeepalive};
use std::io;
use std::time::Duration;
use tokio::net::TcpStream;

/// Applies TCP socket options (nodelay, keepalive) to a tokio [`TcpStream`].
///
/// This helper performs the necessary conversions (tokio -> std -> socket2 -> std -> tokio)
/// to configure socket options that tokio does not expose directly (keepalive interval/retries).
pub(crate) fn apply_socket_options(
    stream: TcpStream,
    tcp_nodelay: bool,
    tcp_keepalive: Option<Duration>,
    tcp_keepalive_interval: Option<Duration>,
    tcp_keepalive_retries: Option<u32>,
) -> io::Result<TcpStream> {
    stream.set_nodelay(tcp_nodelay)?;

    let std_stream = stream.into_std()?;
    let socket: Socket = std_stream.into();

    if let Some(keepalive_time) = tcp_keepalive {
        let mut keepalive = TcpKeepalive::new().with_time(keepalive_time);

        if let Some(interval) = tcp_keepalive_interval {
            keepalive = keepalive.with_interval(interval);
        }

        #[cfg(not(target_os = "windows"))]
        if let Some(retries) = tcp_keepalive_retries {
            keepalive = keepalive.with_retries(retries);
        }

        #[cfg(target_os = "windows")]
        if tcp_keepalive_retries.is_some() {
            otap_df_telemetry::otel_warn!(
                "Socket.KeepaliveRetriesIgnored",
                platform = "windows",
                message = "tcp_keepalive_retries is configured but ignored on Windows: TcpKeepalive::with_retries is not available on this platform"
            );
        }

        socket.set_tcp_keepalive(&keepalive)?;
    }

    let std_stream: std::net::TcpStream = socket.into();
    std_stream.set_nonblocking(true)?;
    TcpStream::from_std(std_stream)
}
