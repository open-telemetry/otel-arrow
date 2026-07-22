// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Loopback-only ephemeral port pickers for tests and validation harnesses.
//!
//! Unlike the `portpicker` crate which probes candidate ports by binding to
//! all interfaces (`0.0.0.0` / `::`) and therefore triggers a Windows Firewall
//! prompt on every freshly-built test binary. These helpers bind to the
//! loopback interface (`127.0.0.1`) only. Loopback binds are exempt from the
//! Windows Firewall, so no prompt (and no leftover firewall rule) is created,
//! while still yielding a free ephemeral port for the current process.
//!
//! Because a `127.0.0.1:0` bind conflicts with any existing all-interfaces
//! (`0.0.0.0`) bind of the same port, the returned port is guaranteed to be
//! free on loopback *and* on `0.0.0.0` at the moment it is picked. As with any
//! "pick then use" scheme there is an inherent time-of-check/time-of-use race:
//! the probe socket is closed before the port is returned, so callers should
//! bind the port promptly.

use std::net::{TcpListener, UdpSocket};

/// Picks a currently-unused TCP port on the loopback interface (`127.0.0.1`).
///
/// # Errors
///
/// Returns any I/O error raised while binding the probe socket or reading its
/// local address.
pub fn try_pick_unused_loopback_tcp_port() -> std::io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

/// Picks a currently-unused UDP port on the loopback interface (`127.0.0.1`).
///
/// # Errors
///
/// Returns any I/O error raised while binding the probe socket or reading its
/// local address.
pub fn try_pick_unused_loopback_udp_port() -> std::io::Result<u16> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    Ok(socket.local_addr()?.port())
}

/// Picks a currently-unused TCP port on the loopback interface (`127.0.0.1`).
///
/// # Panics
///
/// Panics if no loopback TCP port could be bound. Intended for test code.
#[must_use]
pub fn pick_unused_loopback_tcp_port() -> u16 {
    try_pick_unused_loopback_tcp_port().expect("failed to allocate an ephemeral loopback TCP port")
}

/// Picks a currently-unused UDP port on the loopback interface (`127.0.0.1`).
///
/// # Panics
///
/// Panics if no loopback UDP port could be bound. Intended for test code.
#[must_use]
pub fn pick_unused_loopback_udp_port() -> u16 {
    try_pick_unused_loopback_udp_port().expect("failed to allocate an ephemeral loopback UDP port")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tcp_port_is_nonzero() {
        assert_ne!(pick_unused_loopback_tcp_port(), 0);
    }

    #[test]
    fn udp_port_is_nonzero() {
        assert_ne!(pick_unused_loopback_udp_port(), 0);
    }

    #[test]
    fn try_variants_succeed() {
        assert!(try_pick_unused_loopback_tcp_port().is_ok());
        assert!(try_pick_unused_loopback_udp_port().is_ok());
    }
}
