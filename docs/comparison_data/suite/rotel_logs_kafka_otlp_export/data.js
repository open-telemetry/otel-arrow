window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["rotel_logs_kafka_otlp_export"] = {
  "name": "Rotel Kafka Exporter OTLP (Logs)",
  "slug": "rotel_logs_kafka_otlp_export",
  "description": "Exporter-isolation benchmark. A DFE loadgen sends OTLP logs to the benchmarked rotel Kafka exporter, which produces to the broker; a fixed DFE Kafka receiver in the backend consumes them. Isolates rotel Kafka exporter performance.",
  "meta": {
    "binary": "rotel",
    "protocols": [],
    "signals": [
      "logs"
    ],
    "compression": "none"
  },
  "env": null,
  "tests": []
};
