# otap-df-test-net

Loopback-only ephemeral port pickers for tests and validation harnesses.

These helpers bind probe sockets to the loopback interface (`127.0.0.1`) only,
avoiding the Windows Firewall prompt that the `portpicker` crate triggers by
binding to all interfaces (`0.0.0.0` / `::`).

## API

- `pick_unused_loopback_tcp_port() -> u16` - panics on failure (for tests).
- `pick_unused_loopback_udp_port() -> u16` - panics on failure (for tests).
- `try_pick_unused_loopback_tcp_port() -> std::io::Result<u16>`
- `try_pick_unused_loopback_udp_port() -> std::io::Result<u16>`
