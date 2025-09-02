"""
hook_config.py

This module defines the configuration schema for managing lifecycle hooks
attached to components or test phases in the framework.

Key Elements:
- HooksConfig: A Pydantic model representing pre- and post-phase hooks that can
  be applied to a component or test. Each phase supports a list of hook wrappers
  and an associated strategy for how hooks should be added (e.g., append or replace).

Types:
- HookAddStrategy: A literal type defining allowed strategies for modifying the
  existing hook list ("append" or "replace").

Usage:
- Use `HooksConfig` as part of higher-level configurations (e.g., test steps,
  test definitions, components) to declaratively define lifecycle hooks.
- Hooks are wrapped using `HookWrapper` to defer instantiation until build time.

This configuration model enables flexible and declarative control over how hooks
are injected into different phases of execution.
"""

from typing import Dict, Optional

from pydantic import Field, model_validator

from ...core.strategies.reporting_hook_strategy import (
    OutputPipelineConfig,
    ReportFormatterConfig,
    DestinationWriterConfig,
    ReportingHookStrategyConfig,
)
from ..registry import report_formatter_registry, report_writer_registry
from .events import BetweenEventsConfig


class HookOutputPipelineConfig(OutputPipelineConfig):
    """Model for mapping format and destination strategies together."""

    format: Optional[Dict[str, ReportFormatterConfig]] = Field(default_factory=Dict)
    destination: Optional[Dict[str, DestinationWriterConfig]] = Field(
        default_factory=Dict
    )

    @model_validator(mode="before")
    @classmethod
    def load_plugin_configs(cls, values):
        format_dict = values.get("format", {})
        loaded_format = {}
        for plugin_name, config in format_dict.items():
            formatter_cls = report_formatter_registry.config.get(plugin_name)
            loaded_format[plugin_name] = formatter_cls(**config)
        values["format"] = loaded_format

        destination_dict = values.get("destination", {})
        loaded_destination = {}
        for plugin_name, config in destination_dict.items():
            destination_cls = report_writer_registry.config.get(plugin_name)
            loaded_destination[plugin_name] = destination_cls(**config)
        values["destination"] = loaded_destination

        return values


class StandardReportingHookStrategyConfig(ReportingHookStrategyConfig):
    output: Optional[list[HookOutputPipelineConfig]] = Field(default_factory=list)
    between_events: Optional[BetweenEventsConfig] = None
