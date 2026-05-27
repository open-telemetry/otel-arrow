#!/usr/bin/env bash
# Trigger for `syslog_cef_receiver.start`.
#
# This event fires unconditionally during receiver startup, so the
# trigger is "have the receiver in the pipeline" (see
# `configs/internal-events-otlp.yaml`). No runtime action required.
set -euo pipefail
:
