window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_metrics_otlp_gzip_baseline"] = {
  "name": "OTC OTLP Baseline w/ Gzip (Metrics)",
  "slug": "otc_metrics_otlp_gzip_baseline",
  "description": "OpenTelemetry Collector baseline for OTLP metrics with gzip compression",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otlp"
    ],
    "signals": [
      "metrics"
    ],
    "compression": "gzip"
  },
  "env": null,
  "tests": []
};
