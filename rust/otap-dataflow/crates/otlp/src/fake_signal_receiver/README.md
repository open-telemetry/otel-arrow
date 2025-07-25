# Fake Signal Receiver

Status: **WIP**

This crate will contain the implementation of the fake signal receiver

The fake signal receiver will allow users to test their pipelines by defining
a scenario to run, this scenario will be made up of various steps where each
step will describe a batch of signals to send

## Example configuration

```json
{
    "steps": [
        {
            "delay_between_batches_ms": 0,
            "batches_to_generate": 1,
            "config": {
                "signal_type": "metric",
                "resource_count": 1,
                "scope_count": 1,
                "metric_count": 1,
                "datapoint_count": 1,
                "datapoint_type": "gauge",
                "attribute_count": 1
            }
        },
        {
            "delay_between_batches_ms": 0,
            "batches_to_generate": 1,
            "config": {
                "signal_type": "span",
                "resource_count": 1,
                "scope_count": 1,
                "span_count": 1,
                "event_count": 1,
                "link_count": 1,
                "attribute_count": 1
            }
        },
        {
            "delay_between_batches_ms": 0,
            "batches_to_generate": 1,
            "config": {
                "signal_type": "log",
                "resource_count": 1,
                "scope_count": 1,
                "log_count": 1,
                "attribute_count": 1
            }
        }
    ]
}
```
