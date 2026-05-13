window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_metrics_otlp_zstd_baseline"] = {
  "name": "DFE OTLP Baseline w/ Zstd (Metrics)",
  "slug": "dfe_metrics_otlp_zstd_baseline",
  "description": "Dataflow Engine baseline for OTLP metrics with zstd compression",
  "meta": {
    "binary": "dfe",
    "protocols": [
      "otlp"
    ],
    "signals": [
      "metrics"
    ],
    "compression": "zstd"
  },
  "env": null,
  "tests": []
};
