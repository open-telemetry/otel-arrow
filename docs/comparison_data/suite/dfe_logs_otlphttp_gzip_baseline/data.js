window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otlphttp_gzip_baseline"] = {
  "name": "DFE OTLP/HTTP Baseline w/ Gzip (Logs)",
  "slug": "dfe_logs_otlphttp_gzip_baseline",
  "description": "Dataflow Engine baseline for OTLP/HTTP logs with gzip compression",
  "meta": {
    "binary": "dfe",
    "protocols": [
      "otlphttp"
    ],
    "signals": [
      "logs"
    ],
    "compression": "gzip"
  },
  "tests": []
};
