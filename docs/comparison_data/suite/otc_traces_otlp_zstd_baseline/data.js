window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_traces_otlp_zstd_baseline"] = {
  "name": "OTC OTLP Baseline w/ Zstd (Traces)",
  "slug": "otc_traces_otlp_zstd_baseline",
  "description": "OpenTelemetry Collector baseline for OTLP traces with zstd compression",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otlp"
    ],
    "signals": [
      "traces"
    ],
    "compression": "zstd"
  },
  "env": null,
  "tests": []
};
