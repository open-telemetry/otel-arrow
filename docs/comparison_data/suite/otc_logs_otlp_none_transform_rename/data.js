window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otlp_none_transform_rename"] = {
  "name": "OTC OTLP Transform Rename (Logs)",
  "slug": "otc_logs_otlp_none_transform_rename",
  "description": "OpenTelemetry Collector OTLP logs, transform processor (OTTL) renaming exception.type to exception.kind (set + delete_key)",
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
