"""Initialization for the core.test_framework package."""

from .test_definition import TestDefinition
from .test_step import TestStep, TestStepActionConfig, TestStepAction
from .test_suite import TestSuite
from .test_element import TestFrameworkElement, TestLifecyclePhase
from .test_report import TestReport

__all__ = [
    "TestFrameworkElement",
    "TestSuite",
    "TestDefinition",
    "TestStep",
    "TestStepActionConfig",
    "TestStepAction",
    "TestLifecyclePhase",
    "TestReport",
]
