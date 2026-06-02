#!/usr/bin/env bash
# Trigger for `tls.handshake.failed`.
#
# The engine's OTLP receiver is configured with TLS on 127.0.0.1:14317.
# Sending plaintext HTTP at that port causes the server-side handshake
# to fail (no ClientHello), which is the emission path for this event.
set -euo pipefail

for _ in 1 2 3 4 5; do
  curl --max-time 2 -sS http://127.0.0.1:14317/ 2>&1 | head -1 || true
  sleep 0.3
done
