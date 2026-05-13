window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_traces_otlp_zstd_baseline"] = {
  "name": "DFE OTLP Baseline w/ Zstd (Traces)",
  "slug": "dfe_traces_otlp_zstd_baseline",
  "description": "Dataflow Engine baseline for OTLP traces with zstd compression",
  "meta": {
    "binary": "dfe",
    "protocols": [
      "otlp"
    ],
    "signals": [
      "traces"
    ],
    "compression": "zstd"
  },
  "tests": []
};
