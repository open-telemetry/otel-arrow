// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Transport-independent defaults for outbound connections.

use std::time::Duration;

pub(crate) const fn default_concurrency_limit() -> usize {
    256
}

pub(crate) const fn default_connect_timeout() -> Duration {
    Duration::from_secs(3)
}

pub(crate) const fn default_tcp_nodelay() -> bool {
    true
}

pub(crate) const fn default_tcp_keepalive() -> Option<Duration> {
    Some(Duration::from_secs(45))
}
