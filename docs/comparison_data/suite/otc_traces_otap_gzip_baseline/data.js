window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_traces_otap_gzip_baseline"] = {
  "name": "OTC OTAP Baseline w/ Gzip (Traces)",
  "slug": "otc_traces_otap_gzip_baseline",
  "description": "OpenTelemetry Collector baseline for OTAP traces with gzip compression",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otap"
    ],
    "signals": [
      "traces"
    ],
    "compression": "gzip"
  },
  "env": null,
  "tests": []
};
