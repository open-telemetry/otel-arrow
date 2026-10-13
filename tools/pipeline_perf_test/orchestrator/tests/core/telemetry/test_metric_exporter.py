"""Regression coverage for benchmark gauge collection."""

import math

from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader

from lib.core.telemetry.metric import FrameworkMetricBackend, FrameworkMetricExporter


# Scenario: A new container is monitored after the previous container stops updating gauges.
# Guarantees: Only fresh gauge samples are exported, values stay absolute, and counters stay cumulative.
def test_exporter_does_not_replay_stale_gauges():
    backend = FrameworkMetricBackend()
    reader = PeriodicExportingMetricReader(
        FrameworkMetricExporter(backend), export_interval_millis=math.inf
    )
    provider = MeterProvider(metric_readers=[reader])
    meter = provider.get_meter("regression")
    cpu = meter.create_gauge("container.cpu.usage")
    network = meter.create_gauge("container.network.tx")
    counter = meter.create_counter("requests")
    previous = {"component_name": "df-engine", "container_id": "previous"}
    current = {"component_name": "df-engine", "container_id": "current"}

    try:
        cpu.set(0.65, previous)
        network.set(1000000, previous)
        counter.add(1)
        reader.collect()

        cpu.set(0.38, current)
        network.set(100, current)
        counter.add(4)
        reader.collect()
        reader.collect()  # No new measurements: no gauge samples should appear.

        network.set(200, current)
        reader.collect()
        metrics = backend.get_metrics_df()

        def samples(name):
            rows = metrics[metrics["metric_name"] == name]
            return [
                (row.metric_attributes.get("container_id"), row.value)
                for row in rows.itertuples()
            ]

        assert samples("container.cpu.usage") == [("previous", 0.65), ("current", 0.38)]
        assert samples("container.network.tx") == [
            ("previous", 1000000),
            ("current", 100),
            ("current", 200),
        ]
        assert samples("requests") == [(None, 1), (None, 5), (None, 5), (None, 5)]
    finally:
        provider.shutdown()
