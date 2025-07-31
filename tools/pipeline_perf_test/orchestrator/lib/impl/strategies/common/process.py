
import subprocess
from typing import ClassVar, Literal, Optional, Union

from pydantic import BaseModel, ConfigDict

from ....core.context.framework_element_contexts import StepContext
from ....core.component.component import (
    Component,
    ComponentHookContext,
)


class ComponentProcessRuntime(BaseModel):
    """Base Model for component process runtime information."""

    type: ClassVar[Literal["component_process_runtime"]] = "component_process_runtime"
    pid: Optional[int] = None
    process: Optional[subprocess.Popen[bytes]] = None
    std_out_logs: Optional[list[str]] = None
    std_err_logs: Optional[list[str]] = None
    # Support Popen[bytes]
    model_config = ConfigDict(arbitrary_types_allowed=True)


def get_component_process_runtime(
    ctx: Union[ComponentHookContext, StepContext],
) -> ComponentProcessRuntime:
    """Get runtime process information from the context.

    Args:
        ctx: The current context

    Returns: The existing process runtime or a new one"""
    component = ctx.get_step_component()
    assert isinstance(component, Component), "Expected Component"
    return component.get_or_create_runtime(
        ComponentProcessRuntime.type, ComponentProcessRuntime
    )