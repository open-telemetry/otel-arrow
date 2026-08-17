window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["rotel_logs_kafka_otlp_recv"] = {
  "name": "Rotel Kafka Receiver OTLP (Logs)",
  "slug": "rotel_logs_kafka_otlp_recv",
  "description": "Receiver-isolation benchmark. A fixed DFE Kafka exporter produces OTLP protobuf logs to the broker; the benchmarked rotel Kafka receiver consumes them and forwards to a DFE backend. Isolates rotel Kafka receiver performance.",
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
