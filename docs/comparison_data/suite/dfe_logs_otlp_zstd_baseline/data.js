window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otlp_zstd_baseline"] = {
  "name": "DFE OTLP Baseline w/ Zstd (Logs)",
  "slug": "dfe_logs_otlp_zstd_baseline",
  "description": "Dataflow Engine baseline for OTLP logs with zstd compression",
  "meta": {
    "binary": "dfe",
    "protocols": [
      "otlp"
    ],
    "signals": [
      "logs"
    ],
    "compression": "zstd"
  },
  "env": null,
  "tests": []
};
