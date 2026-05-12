window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otlp_gzip_baseline"] = {
  "name": "DFE OTLP Baseline w/ Gzip (Logs)",
  "slug": "dfe_logs_otlp_gzip_baseline",
  "description": "Dataflow Engine baseline for OTLP logs with gzip compression",
  "meta": {
    "binary": "dfe",
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
