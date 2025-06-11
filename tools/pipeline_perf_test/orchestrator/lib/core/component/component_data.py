"""
component_data.py

This module defines the `ComponentData` class, which encapsulates monitoring-related
data collected from a system component during test execution. This includes both
metrics (as arbitrary key-value pairs) and runtime information, allowing consistent
representation of component state for reporting, analysis, or serialization.

The `ComponentData` class provides a `from_component` factory method to construct
instances directly from a `Component` and a `TestExecutionContext`.

Typical usage:
    data = ComponentData.from_component(component, context)
"""

from dataclasses import dataclass, field
from typing import Any, Dict, TYPE_CHECKING

from ..runtime.runtime import Runtime

if TYPE_CHECKING:
    from ..context.test_contexts import TestExecutionContext
    from .component import Component


@dataclass
class ComponentData:
    """The class holds data about a component including arbitrary runtime data and metrics"""

    metrics: Dict[str, Any] = field(default_factory=dict)
    runtime: Runtime = None

    @classmethod
    def from_component(
        cls, component: "Component", context: "TestExecutionContext"
    ) -> "ComponentData":
        """Create a ComponentData instance from a component and context."""
        return cls(
            metrics=component.collect_monitoring_data(context),
            runtime=component.runtime,
        )
