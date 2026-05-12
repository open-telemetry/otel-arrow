window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otlp_gzip_baseline"] = {
  "name": "OTC OTLP Baseline w/ Gzip (Logs)",
  "slug": "otc_logs_otlp_gzip_baseline",
  "description": "OpenTelemetry Collector baseline for OTLP logs with gzip compression",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otlp"
    ],
    "signals": [
      "logs"
    ],
    "compression": "gzip"
  },
  "tests": []
};
