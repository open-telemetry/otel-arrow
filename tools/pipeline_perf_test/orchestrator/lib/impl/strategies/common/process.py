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


def wait_or_terminate_process_tree(
    pid: int,
    logger: Union[Logger, LoggerAdapter],
    normal_timeout: float = 5.0,
    graceful_timeout: float = 3.0,
) -> None:
    """Wait for a process tree to exit, escalating to termination if needed.

    This operates on the whole process tree rooted at ``pid`` (the process and all
    of its descendants) so that a wedged or shell-launched command cannot leave
    orphaned grandchildren running: killing only the direct child would reap a
    shell while leaving the real command alive.

    The escalation proceeds in up to three stages, and the path taken is logged so
    teardown behavior is observable:

    1. Normal: wait up to ``normal_timeout`` seconds for the tree to exit on its
       own (e.g. after an earlier graceful stop request). Pass ``normal_timeout=0``
       to skip this stage when the tree is already known to be stuck.
    2. Graceful: ask any survivors to terminate (SIGTERM / TerminateProcess) and
       wait up to ``graceful_timeout`` seconds.
    3. Force: kill anything still alive (SIGKILL / forced) and reap it.

    All operations use ``psutil`` primitives so the behavior is identical on POSIX
    and Windows.

    Args:
        pid: PID of the root process whose tree should be waited on / terminated.
        logger: Logger used to report which escalation path was taken.
        normal_timeout: Seconds to wait for the tree to exit normally before
            escalating. Use 0 to skip the normal wait. Defaults to 5.0.
        graceful_timeout: Seconds to wait after a graceful terminate (and after a
            force kill) for survivors to exit. Defaults to 3.0.
    """
    try:
        parent = psutil.Process(pid)
        # Enumerate descendants before killing so the tree is still intact.
        procs = parent.children(recursive=True) + [parent]
    except psutil.NoSuchProcess:
        logger.debug(f"Process {pid} already exited; nothing to wait for.")
        return

    # Stage 1: wait for the tree to exit on its own.
    _, alive = psutil.wait_procs(procs, timeout=normal_timeout)
    if not alive:
        logger.debug(f"Process tree for {pid} exited within {normal_timeout:.0f}s.")
        return

    # Stage 2: ask survivors to terminate gracefully.
    logger.warning(
        f"Process tree for {pid} did not exit within {normal_timeout:.0f}s; "
        f"terminating {len(alive)} remaining process(es)."
    )
    for proc in alive:
        try:
            proc.terminate()
        except psutil.NoSuchProcess:
            continue
    _, alive = psutil.wait_procs(alive, timeout=graceful_timeout)
    if not alive:
        logger.info(f"Process tree for {pid} terminated gracefully.")
        return

    # Stage 3: force-kill anything that ignored the graceful request.
    logger.warning(
        f"Process tree for {pid} did not terminate within {graceful_timeout:.0f}s; "
        f"force-killing {len(alive)} remaining process(es)."
    )
    for proc in alive:
        try:
            proc.kill()
        except psutil.NoSuchProcess:
            continue
    # Reap the force-killed processes so they do not linger as zombies.
    _, alive = psutil.wait_procs(alive, timeout=graceful_timeout)
    if alive:
        logger.warning(
            f"Process tree for {pid} still has {len(alive)} live process(es) "
            "after force-kill."
        )
