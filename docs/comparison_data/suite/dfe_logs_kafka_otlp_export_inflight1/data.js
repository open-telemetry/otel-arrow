window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_kafka_otlp_export_inflight1"] = {
  "name": "DFE Kafka Exporter OTLP max_in_flight=1 (Logs)",
  "slug": "dfe_logs_kafka_otlp_export_inflight1",
  "description": "Exporter-isolation benchmark with the DFE Kafka exporter pinned to the serial in-flight depth (max_in_flight = 1). Identical to dfe_logs_kafka_otlp_export except for the in-flight depth, so the two suites give an apples-to-apples comparison of max_in_flight 1 vs 1000. A DFE loadgen sends OTLP logs to the benchmarked DFE Kafka exporter, which produces to the broker; a fixed DFE Kafka receiver in the backend consumes them.",
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
