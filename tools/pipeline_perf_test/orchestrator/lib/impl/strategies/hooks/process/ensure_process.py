import time

from typing import Optional

from .....core.context.base import ExecutionStatus, BaseContext
from .....core.context import ComponentHookContext
from .....core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from .....runner.registry import hook_registry, PluginMeta
from ...common.process import get_component_process_runtime


@hook_registry.register_config("ensure_process")
class EnsureProcessConfig(HookStrategyConfig):
    """
    Configuration for the 'ensure_process' hook.

    Attributes:
        delay: Number of seconds to wait before checking the process.
    """

    delay: Optional[float] = 1


@hook_registry.register_class("ensure_process")
class EnsureProcess(HookStrategy):
    """
    Hook strategy to ensure component specified is running and hasn't crashed after start.

    This hook checks the host process table for the id registered during deployment.

    Behavior:
        - Retrieves the process ID from the component's runtime.
        - Polls the os with psutil for the current status.
        - If no container ID is found, the hook fails immediately.

    Hook status:
        - SUCCESS if all specified component processes are running
        - FAILURE if any specified component process is not running

    Raises:
        RuntimeError: If no process ID is available in the component's runtime or the process isn't running.

    Typical usage:
        - As a post-deploy hook to ensure the process didn't crash immediately.
        - To coordinate pipeline steps based on process readiness.
    """

    PLUGIN_META = PluginMeta(
        supported_contexts=[ComponentHookContext.__name__],
        installs_hooks=[],
        notes="This hook is automatically installed by the process deployment strategy and doesn't need to be added explicitly",
        yaml_example="""
components:
  otel-collector:
    deployment:
      process: ...
    hooks:
        deploy:
            post:
            - ensure_process:
                delay: 1
""",
    )

    def __init__(self, config: EnsureProcessConfig):
        self.config = config

    def execute(self, ctx: BaseContext):
        """Execute the strategy to ensure the component process is still running."""
        logger = ctx.get_logger(__name__)
        runtime = get_component_process_runtime(ctx)
        component = ctx.get_step_component()

        if not runtime.process:
            ctx.status = ExecutionStatus.FAILURE
            raise RuntimeError(f"No process available for component: {component.name}")

        if self.config.delay:
            time.sleep(self.config.delay)

        if runtime.process.poll() is None:
            ctx.status = ExecutionStatus.SUCCESS
            logger.debug(
                f"Component process for {component.name} is still "
                f"running after {self.config.delay} seconds... continuing"
            )
            return

        else:
            ctx.status = ExecutionStatus.FAILURE
            logger.error(f"Process for component is no longer running: {component.name}, checking logs...")
            try:
                stdout_logs, stderr_logs = runtime.process.communicate(timeout=5)
                if stdout_logs:
                    decoded = (
                        stdout_logs.decode("utf-8")
                        if isinstance(stdout_logs, bytes)
                        else str(stdout_logs)
                    )
                    runtime.std_out_logs = decoded.splitlines()
                    logger.error("Process std out For %s:\n%s", component.name, decoded)
                if stderr_logs:
                    decoded = (
                        stderr_logs.decode("utf-8")
                        if isinstance(stderr_logs, bytes)
                        else str(stderr_logs)
                    )
                    runtime.std_err_logs = decoded.splitlines()
                    logger.error("Process std err For %s:\n%s", component.name, decoded)
            except TimeoutError:
                logger.error(f"Failed to fetch logs for: {component.name}.")

            raise RuntimeError(f"Process for component is no longer running: {component.name}")
