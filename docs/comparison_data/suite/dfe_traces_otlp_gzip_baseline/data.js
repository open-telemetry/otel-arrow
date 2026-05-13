window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_traces_otlp_gzip_baseline"] = {
  "name": "DFE OTLP Baseline w/ Gzip (Traces)",
  "slug": "dfe_traces_otlp_gzip_baseline",
  "description": "Dataflow Engine baseline for OTLP traces with gzip compression",
  "meta": {
    "binary": "dfe",
    "protocols": [
      "otlp"
    ],
    "signals": [
      "traces"
    ],
    "compression": "gzip"
  },
  "tests": []
};
