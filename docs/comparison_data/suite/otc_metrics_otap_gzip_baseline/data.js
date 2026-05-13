window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_metrics_otap_gzip_baseline"] = {
  "name": "OTC OTAP Baseline w/ Gzip (Metrics)",
  "slug": "otc_metrics_otap_gzip_baseline",
  "description": "OpenTelemetry Collector baseline for OTAP metrics with gzip compression",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otap"
    ],
    "signals": [
      "metrics"
    ],
    "compression": "gzip"
  },
  "env": null,
  "tests": []
};
