window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otlphttp_zstd_baseline"] = {
  "name": "DFE OTLP/HTTP Baseline w/ Zstd (Logs)",
  "slug": "dfe_logs_otlphttp_zstd_baseline",
  "description": "Dataflow Engine baseline for OTLP/HTTP logs with zstd compression",
  "meta": {
    "binary": "dfe",
    "protocols": [
      "otlphttp"
    ],
    "signals": [
      "logs"
    ],
    "compression": "zstd"
  },
  "env": null,
  "tests": []
};
