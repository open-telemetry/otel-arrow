window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["fbit_logs_otlp_none_attr_rename"] = {
  "name": "FB OTLP Attr Rename (Logs)",
  "slug": "fbit_logs_otlp_none_attr_rename",
  "description": "Fluent Bit OTLP logs, modify filter renaming exception.type to exception.kind",
  "meta": {
    "binary": "fbit",
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
