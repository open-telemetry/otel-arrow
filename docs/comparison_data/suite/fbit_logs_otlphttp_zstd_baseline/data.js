window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["fbit_logs_otlphttp_zstd_baseline"] = {
  "name": "FB OTLP/HTTP Baseline (Logs, Zstd)",
  "slug": "fbit_logs_otlphttp_zstd_baseline",
  "description": "Fluent Bit baseline passthrough for OTLP/HTTP logs with zstd compression",
  "meta": {
    "binary": "fbit",
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
