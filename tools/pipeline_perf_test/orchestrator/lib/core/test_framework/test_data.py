from dataclasses import dataclass, field
from typing import Dict

from ..component.component_data import ComponentData
from ..context.test_contexts import TestExecutionContext


@dataclass
class TestData:
    """This class holds data about the test run, generally to be consumed by a reporting strategy."""

    context: TestExecutionContext
    component_data: Dict[str, ComponentData] = field(default_factory=dict)
