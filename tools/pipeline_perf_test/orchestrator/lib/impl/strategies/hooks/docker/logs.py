from docker.errors import NotFound, APIError
from .....core.context.base import ExecutionStatus, BaseContext
from .....core.context.component_hook_context import ComponentHookContext
from .....core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from .....runner.registry import hook_registry, PluginMeta
from ...deployment.process import ProcessRuntime

from ...common.docker import get_component_docker_runtime, get_or_create_docker_client


@hook_registry.register_config("get_docker_logs")
class GetDockerLogsConfig(HookStrategyConfig):
    """Configuration for Get Docker Logs Hook."""


@hook_registry.register_class("get_docker_logs")
class GetDockerLogs(HookStrategy):
    """
    Hook strategy to retrieve logs and store them in the component's process_runtime.

    This hook is intended to be used as part of a pipeline or test framework where
    container logs are collected for debugging, auditing, or analysis. When executed,
    it locates the Docker container associated with the current component and stores
    the logs in the component's `ProcessRuntime`, making them accessible for later inspection.

    Typical usage:
        - Collect logs before or after component execution (e.g., in pre/post hooks)
        - Enhance observability in test pipelines using Docker-based deployments

    Attributes:
        config (GetDockerLogsConfig): Configuration for the log retrieval behavior.
    """
    PLUGIN_META = PluginMeta(
        supported_contexts=[ComponentHookContext.__name__],
        installs_hooks=[],
        yaml_example="""
components:
  otel-collector:
    deployment:
      docker: ...
    hooks:
        destroy:
            pre:
            - get_docker_logs: {}
"""
    )
    def __init__(self, config: GetDockerLogsConfig):
        self.config = config

    def execute(self, ctx: BaseContext):
        """
        Get logs from a Docker container and add them to the component runtime.

        Args:
            ctx: the current execution context

        Raises:
            docker.errors.APIError: If there is an error communicating with the docker server.
            docker.errors.NotFound: If the container cannot be found.
        """
        logger = ctx.get_logger(__name__)
        client = get_or_create_docker_client(ctx)
        component = ctx.get_step_component()
        if not component:
            logger.warn("No component found in context.")
            ctx.status = ExecutionStatus.FAILURE
            return None
        docker_runtime = get_component_docker_runtime(ctx)
        process_runtime: ProcessRuntime = component.get_or_create_runtime(
            ProcessRuntime.type, ProcessRuntime
        )
        try:
            container = client.containers.get(docker_runtime.container_id)
            logs = container.logs(
                stdout=True, stderr=True, stream=False, timestamps=False
            )
            process_runtime.logs = (
                logs.decode("utf-8") if isinstance(logs, bytes) else str(logs)
            )
            component.set_runtime_data(ProcessRuntime.type, process_runtime)
        except (NotFound, APIError) as e:
            logger.error(f"Error getting Docker container logs: {e}")
            raise
