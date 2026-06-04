window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["fbit_logs_otlp_gzip_baseline"] = {
  "name": "FB OTLP Baseline (Logs, Gzip)",
  "slug": "fbit_logs_otlp_gzip_baseline",
  "description": "Fluent Bit baseline passthrough for OTLP logs with gzip compression",
  "meta": {
    "binary": "fbit",
    "protocols": [
      "otlp"
    ],
    "signals": [
      "logs"
    ],
    "compression": "gzip"
  },
  "env": null,
  "tests": []
};
