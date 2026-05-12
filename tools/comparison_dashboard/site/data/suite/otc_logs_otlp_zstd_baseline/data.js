window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otlp_zstd_baseline"] = {
  "name": "OTC OTLP Baseline w/ Zstd (Logs)",
  "slug": "otc_logs_otlp_zstd_baseline",
  "description": "OpenTelemetry Collector baseline for OTLP logs with zstd compression",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otlp"
    ],
    "signals": [
      "logs"
    ],
    "compression": "zstd"
  },
  "tests": []
};
