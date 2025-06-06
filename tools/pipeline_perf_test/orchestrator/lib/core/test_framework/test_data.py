"""
Module: test_data

This module defines the `TestData` class, a container for storing information about a test run.
It aggregates execution context and per-component data to support reporting, validation, and
post-execution analysis.

The `TestData` object is typically populated during or after test execution and passed to
reporting strategies or other consumers that require a structured view of test results.

Use cases include:
    - Collecting metrics or artifacts from individual test components.
    - Generating structured reports based on the test execution context.
    - Providing a unified data interface to post-processing tools or hooks.

Classes:
    TestData: Stores the test execution context and associated component-level data.
"""

from dataclasses import dataclass, field
from typing import Dict, TYPE_CHECKING

from ..component.component_data import ComponentData

if TYPE_CHECKING:
    from ..context.test_contexts import TestExecutionContext


@dataclass
class TestData:
    """This class holds data about the test run, generally to be consumed by a reporting strategy."""

    context: "TestExecutionContext"
    component_data: Dict[str, ComponentData] = field(default_factory=dict)
