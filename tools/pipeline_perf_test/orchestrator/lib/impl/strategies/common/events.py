"""
Utilities for querying and handling telemetry span events within execution strategies.

This module provides helper functions to retrieve specific telemetry span events
based on configuration criteria, facilitating measurement or coordination
between defined start and end events during a test or monitoring workflow.

Key features:
- Querying span events using a TelemetryClient based on event names and attributes.
- Extracting earliest start event and latest end event within configured criteria.

The primary function `get_start_end_event` accepts a configuration specifying
the target start and end events, along with a TelemetryClient instance to perform
queries on span telemetry data. It returns the matching start and end events
as data frames for downstream processing.

This utility supports precise event boundary detection needed for performance
measurement, event correlation, or conditional workflow execution.
"""

from typing import Optional, Tuple

from ....core.telemetry.span import SpanEventDataFrame
from ....core.telemetry.telemetry_client import TelemetryClient
from ....runner.schema.events import BetweenEventsConfig


def get_start_end_event(
    between_events: BetweenEventsConfig, tc: TelemetryClient
) -> Tuple[Optional[SpanEventDataFrame], Optional[SpanEventDataFrame]]:
    """
    Retrieve the earliest start event and latest end event span data frames based on configuration.

    This function queries telemetry span events from the provided TelemetryClient
    according to the event names and attributes defined in the BetweenEventsConfig.
    It extracts the earliest start event and the latest end event matching the criteria.

    Args:
        between_events (BetweenEventsConfig): Configuration specifying start and end event criteria.
        tc (TelemetryClient): Telemetry client used to query span events.

    Returns:
        Tuple[Optional[SpanEventDataFrame], Optional[SpanEventDataFrame]]:
            A tuple containing two optional SpanEventDataFrames:
            - The first element is the earliest start event matching the configuration (or None if not found).
            - The second element is the latest end event matching the configuration (or None if not found).
    """
    start_event = None
    end_event = None
    start_cfg = between_events.start
    if start_cfg:
        events = tc.spans.query_span_events(
            name=start_cfg.name, attributes=start_cfg.attributes
        )
        if not events.empty:
            start_event_time = events["timestamp"].min()
            start_event = events[events["timestamp"] == start_event_time]
    end_cfg = between_events.end
    if end_cfg:
        events = tc.spans.query_span_events(
            name=end_cfg.name, attributes=end_cfg.attributes
        )
        if not events.empty:
            end_event_time = events["timestamp"].max()
            end_event = events[events["timestamp"] == end_event_time]
    return (start_event, end_event)
