// This module provides facilities for testing round-trip fidelity of telemetry data
// through a Golang OTel Collector. It can create test data, send it to a collector,
// receive the exported data, and verify that the data matches what was sent.

mod collector;
mod otlp;
mod service_type;
