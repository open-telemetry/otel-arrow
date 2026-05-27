#!/usr/bin/env bash
# Trigger for `otlp.exporter.http.export_error`.
#
# The engine config wires a `traffic_generator` receiver to an
# `otlp_grpc` exporter pointed at a port with no listener
# (127.0.0.1:14999). The exporter retries and emits this event on
# every failed request. No runtime action is required.
set -euo pipefail
:
