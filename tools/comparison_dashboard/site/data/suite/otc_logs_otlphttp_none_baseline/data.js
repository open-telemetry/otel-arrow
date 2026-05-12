window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otlphttp_none_baseline"] = {
  "name": "OTC OTLP/HTTP Baseline (Logs)",
  "slug": "otc_logs_otlphttp_none_baseline",
  "description": "OpenTelemetry Collector baseline for OTLP/HTTP logs with no compression",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otlphttp"
    ],
    "signals": [
      "logs"
    ],
    "compression": "none"
  },
  "tests": []
};
