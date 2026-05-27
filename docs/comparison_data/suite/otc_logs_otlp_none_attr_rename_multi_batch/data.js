window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otlp_none_attr_rename_multi_batch"] = {
  "name": "OTC OTLP Attr Rename Multi Batch (Logs)",
  "slug": "otc_logs_otlp_none_attr_rename_multi_batch",
  "description": "OpenTelemetry Collector OTLP logs, attributes processor renaming exception.type to exception.kind (from_attribute + delete), swept across loadgen batch sizes at 400k signals/sec",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otlp"
    ],
    "signals": [
      "logs"
    ],
    "compression": "none"
  },
  "env": null,
  "tests": []
};
