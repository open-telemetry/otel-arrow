"""Initialization for the core.test_framework package."""

from .test_data import TestData
from .test_definition import TestDefinition
from .test_step import TestStep
from .test_suite import TestSuite
from .test_element import TestFrameworkElement, TestLifecyclePhase

__all__ = [
    "TestData",
    "TestFrameworkElement",
    "TestSuite",
    "TestDefinition",
    "TestStep",
    "TestLifecyclePhase",
]
