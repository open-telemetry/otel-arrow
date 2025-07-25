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
                "datapoints": {
                    "datapoint_count": 3,
                    "datapoint_type": [
                        "gauge",
                        "histogram"
                    ],
                    "attributes": {
                        "cpu_mhz": [
                            "2.4",
                            "4.2"
                        ],
                        "cpu": [
                            0
                        ],
                        "cpu_cores": [
                            "4",
                            "8",
                            "16"
                        ],
                        "cpu_vendor": [
                            "intel"
                        ],
                        "cpu_id": [
                            "cpu-0",
                            "cpu-1",
                            "cpu-2",
                            "cpu-3"
                        ],
                        "cpu_arch": [
                            "x86-64"
                        ],
                        "cpu_model": [
                            "i7",
                            "i5"
                        ],
                        "cpu_logical_processors": [
                            "8"
                        ]
                    },
                    "top_value": 50.0,
                    "bottom_value": 0.0
                }
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
                "attributes": {
                    "hostname": [
                        "host3.thedomain.edu",
                        "host1.mydomain.com",
                        "host4.gov",
                        "host2.org"
                    ],
                    "up": [
                        true,
                        false
                    ],
                    "version": [
                        "2.0.0",
                        "1.0.0",
                        "1.5.2"
                    ],
                    "status": [
                        400,
                        404,
                        200
                    ]
                },
                "span_names": [
                    "dns-lookup",
                    "message-send",
                    "http-close",
                    "unknown",
                    "http-send",
                    "http-close"
                ],
                "events": {
                    "event_count": 0,
                    "event_names": [],
                    "attributes": {}
                },
                "links": {
                    "link_count": 0,
                    "trace_states": [],
                    "attributes": {}
                }
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
                "attributes": {
                    "hostname": [
                        "host3.thedomain.edu",
                        "host1.mydomain.com",
                        "host4.gov",
                        "host2.org"
                    ],
                    "up": [
                        true,
                        false
                    ],
                    "version": [
                        "2.0.0",
                        "1.0.0",
                        "1.5.2"
                    ],
                    "status": [
                        400,
                        404,
                        200
                    ]
                },
                "event_names": [
                    "unknown",
                    "message-receive",
                    "message-send",
                    "http-receive",
                    "http-send"
                ]
            }
        }
    ]
}
```
