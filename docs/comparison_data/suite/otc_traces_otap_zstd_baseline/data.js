window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_traces_otap_zstd_baseline"] = {
  "name": "OTC OTAP Baseline w/ Zstd (Traces)",
  "slug": "otc_traces_otap_zstd_baseline",
  "description": "OpenTelemetry Collector baseline for OTAP traces with zstd compression",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otap"
    ],
    "signals": [
      "traces"
    ],
    "compression": "zstd"
  },
  "env": null,
  "tests": []
};
