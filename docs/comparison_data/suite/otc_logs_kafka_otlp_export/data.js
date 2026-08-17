window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_kafka_otlp_export"] = {
  "name": "OTC Kafka Exporter OTLP (Logs)",
  "slug": "otc_logs_kafka_otlp_export",
  "description": "Exporter-isolation benchmark. A DFE loadgen sends OTLP logs to the benchmarked OpenTelemetry Collector Kafka exporter, which produces to the broker; a fixed DFE Kafka receiver in the backend consumes them. Isolates the Go collector Kafka exporter performance.",
  "meta": {
    "binary": "otc",
    "protocols": [],
    "signals": [
      "logs"
    ],
    "compression": "none"
  },
  "env": null,
  "tests": []
};
