# Fake Signal Receiver

Status: **WIP**

This crate will contain the implementation of the fake signal receiver

## Example configuration
```json
{
    "name": "config",
    "steps": [
        {
            "config": {
                "signal_type": "Metric",
                "resource_count": 1,
                "scope_count": 1,
                "metric_count": 1,
                "datapoint_count": 1,
                "datapoint_type": "Gauge"
            }
        },
        {
            "config": {
                "signal_type": "Span",
                "resource_count": 1,
                "scope_count": 1,
                "span_count": 1,
                "event_count": 1,
                "link_count": 1
            }
        },
        {
            "config": {
                "signal_type": "Profile",
                "resource_count": 1,
                "scope_count": 1,
                "profile_count": 1
            }
        },
        {
            "config": {
                "signal_type": "Log",
                "resource_count": 1,
                "scope_count": 1,
                "log_count": 1
            }
        }
    ]
}
```
