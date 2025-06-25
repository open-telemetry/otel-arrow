"""
Module: event_config.py

Description:
    This module defines configuration models for event-based filtering using Pydantic.
    It includes data structures to specify filtering criteria for individual events,
    as well as configurations for specifying relationships between pairs of events.

Classes:
    EventFilterConfig: Represents filtering criteria for a single event.
    BetweenEventsConfig: Represents a configuration for specifying a range between two events.
"""
from typing import Dict, Optional, Any
from pydantic import BaseModel


class EventFilterConfig(BaseModel):
    """
    Represents filtering criteria for a single event.

    Attributes:
        name (Optional[str]): The name of the event to filter on.
        attributes (Optional[Dict[str, Any]]): A dictionary of attribute filters
            to match specific properties of the event.
    """
    name: Optional[str] = None
    attributes: Optional[Dict[str, Any]] = None


class BetweenEventsConfig(BaseModel):
    """
    Configuration for defining a relationship between two events.

    Attributes:
        start (EventFilterConfig): The filter configuration for the starting event.
        end (EventFilterConfig): The filter configuration for the ending event.
    """
    start: EventFilterConfig
    end: EventFilterConfig
