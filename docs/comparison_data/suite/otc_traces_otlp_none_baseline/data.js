window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_traces_otlp_none_baseline"] = {
  "name": "OTC OTLP Baseline (Traces)",
  "slug": "otc_traces_otlp_none_baseline",
  "description": "OpenTelemetry Collector baseline for OTLP traces with no compression",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otlp"
    ],
    "signals": [
      "traces"
    ],
    "compression": "none"
  },
  "env": null,
  "tests": []
};
