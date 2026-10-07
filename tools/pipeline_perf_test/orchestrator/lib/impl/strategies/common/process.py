import subprocess
from logging import Logger, LoggerAdapter
from typing import ClassVar, Literal, Optional, Union

import psutil
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


def terminate_process_tree(
    pid: int,
    logger: Union[Logger, LoggerAdapter],
    graceful_timeout: float = 3.0,
) -> None:
    """Terminate a process and all of its descendants.

    This kills the whole process tree rooted at ``pid`` so that a timed-out or
    wedged command cannot leave orphaned grandchildren running. This matters when
    a process is launched via a shell (``shell=True``): killing only the direct
    child would reap the shell while leaving the real command alive.

    The tree is enumerated up front (while it is still intact) and then each
    process is asked to exit gracefully. Any process still alive after
    ``graceful_timeout`` seconds is force-killed. All operations use ``psutil``
    primitives so the behavior is identical on POSIX and Windows.

    Args:
        pid: PID of the root process whose tree should be terminated.
        logger: Logger used to report progress and non-fatal issues.
        graceful_timeout: Seconds to wait for graceful termination before
            force-killing survivors. Defaults to 3.0.
    """
    try:
        parent = psutil.Process(pid)
    except psutil.NoSuchProcess:
        # Already gone; nothing to do.
        return

    # Enumerate descendants before killing so the tree is still intact.
    procs = parent.children(recursive=True) + [parent]

    # Ask each process to exit gracefully first.
    for proc in procs:
        try:
            proc.terminate()
        except psutil.NoSuchProcess:
            continue

    _, alive = psutil.wait_procs(procs, timeout=graceful_timeout)

    # Force-kill anything that ignored the graceful request.
    for proc in alive:
        logger.warning(
            f"Process {proc.pid} did not terminate within {graceful_timeout:.0f}s; "
            "force-killing it."
        )
        try:
            proc.kill()
        except psutil.NoSuchProcess:
            continue

    # Reap the force-killed processes so they do not linger as zombies.
    if alive:
        psutil.wait_procs(alive, timeout=graceful_timeout)
