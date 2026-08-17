window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_kafka_otap_export"] = {
  "name": "DFE Kafka Exporter OTAP (Logs)",
  "slug": "dfe_logs_kafka_otap_export",
  "description": "Exporter-isolation benchmark with OTAP Arrow encoding. A DFE loadgen sends OTLP logs to the benchmarked DFE Kafka exporter, which produces OTAP messages to the broker; a fixed DFE Kafka receiver in the backend consumes them. Isolates DFE Kafka exporter performance for the OTAP encoding.",
  "meta": {
    "binary": "dfe",
    "protocols": [],
    "signals": [
      "logs"
    ],
    "compression": "none"
  },
  "env": null,
  "tests": []
};
