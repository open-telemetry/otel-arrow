#!/usr/bin/env bash
# Trigger for `otlp.exporter.http.export_recovered`.
#
# The engine's `http_exporter` sends to 127.0.0.1:14998, where nothing
# listens at startup, so its exports fail and fire
# `otlp.exporter.http.export_error`. This script then serves OTLP/HTTP on
# that port. The exporter reports recovery on the first successful export
# that completes RECOVERY_INTERVAL (30 s) after the last failure, so the
# sink stays up longer than that.
set -euo pipefail

port=14998
serve_seconds=40

# Make sure the exporter has failed at least once before the sink appears.
sleep 2

python3 - "$port" "$serve_seconds" <<'PY'
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

port, serve_seconds = int(sys.argv[1]), float(sys.argv[2])
exports = []


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_POST(self):
        self.rfile.read(int(self.headers.get("Content-Length", 0)))
        exports.append(self.path)
        # An empty body decodes as an export response without partial success.
        self.send_response(200)
        self.send_header("Content-Type", "application/x-protobuf")
        self.send_header("Content-Length", "0")
        self.end_headers()

    def log_message(self, *args):
        pass


server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
threading.Timer(serve_seconds, server.shutdown).start()
server.serve_forever()
print(f"OTLP/HTTP sink on 127.0.0.1:{port} accepted {len(exports)} exports")
if not exports:
    print("::warning::the OTLP/HTTP exporter never reached the sink")
PY
