import time
from docker.errors import DockerException
from typing import Union
from .....core.context.base import ExecutionStatus, BaseContext
from .....core.context import ComponentHookContext
from .....core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from .....runner.registry import hook_registry, PluginMeta
from ...common.docker import (
    get_component_docker_runtime,
    get_or_create_docker_client,
    DockerContainerStatus,
)


@hook_registry.register_config("wait_for_status")
class WaitForDockerStatusConfig(HookStrategyConfig):
    """
    Configuration for the 'wait_for_status' hook.

    Attributes:
        status (Union[str, DockerContainerStatus]): The desired container status to wait for.
            Defaults to 'running'. Can be a string or a DockerContainerStatus enum.
        timeout (float): Maximum duration (in seconds) to wait for the container to reach
            the target status. Defaults to 30 seconds.
        interval (float): Polling interval (in seconds) between status checks. Defaults to 1.0.
    """

    status: Union[str, DockerContainerStatus] = DockerContainerStatus.RUNNING
    timeout: float = 30
    interval: float = 1.0


@hook_registry.register_class("wait_for_status")
class WaitForDockerStatus(HookStrategy):
    """
    Hook strategy to wait for a Docker container to reach a specific status.

    This hook polls the Docker daemon at a configurable interval until the container
    associated with the current component reaches the desired lifecycle state (e.g., "running").
    It is typically used in post-deploy hooks to ensure a container is healthy before proceeding.

    Behavior:
        - Retrieves the container ID from the component's runtime.
        - Polls the Docker client for the current status.
        - If the container reaches the target status within the timeout, the hook succeeds.
        - If the timeout is exceeded, raises a TimeoutError and sets context status to TIMEOUT.
        - If no container ID is found, the hook fails immediately.

    Hook status:
        - SUCCESS if desired container status is reached.
        - FAILURE if no container ID is available.
        - TIMEOUT if container does not reach the desired state in time.

    Raises:
        TimeoutError: If the container fails to reach the desired status within the timeout.
        RuntimeError: If no container ID is available in the component's runtime.
        docker.errors.DockerException: If there are errors communicating with Docker.

    Typical usage:
        - As a post-deploy hook to block until the container is `running`.
        - To coordinate pipeline steps based on container readiness.
    """

    PLUGIN_META = PluginMeta(
        supported_contexts=[ComponentHookContext.__name__],
        installs_hooks=[],
        notes="This hook is automatically installed by the docker deployment strategy and doesn't need to be added explicitly",
        yaml_example="""
components:
  otel-collector:
    deployment:
      docker: ...
    hooks:
        deploy:
            post:
            - wait_for_status:
                status: running
                timeout: 30
                interval: 1
""",
    )

    def __init__(self, config: WaitForDockerStatusConfig):
        self.config = config

    def execute(self, ctx: BaseContext):

        logger = ctx.get_logger(__name__)
        runtime = get_component_docker_runtime(ctx)
        container_id = runtime.container_id

        if not container_id:
            ctx.status = ExecutionStatus.FAILURE
            raise RuntimeError("No container ID available to check status.")

        client = get_or_create_docker_client(ctx)
        deadline = time.time() + self.config.timeout
        desired_status = (
            self.config.status.value
            if isinstance(self.config.status, DockerContainerStatus)
            else self.config.status
        )

        while time.time() < deadline:
            try:
                container = client.containers.get(container_id)
                current_status = container.status

                logger.debug(
                    f"Current status for {container_id[:12]}: {current_status}, (want {str(desired_status)})"
                )

                if current_status == desired_status:
                    logger.debug(
                        f"Container {container_id[:12]} reached status '{desired_status}'."
                    )
                    ctx.status = ExecutionStatus.SUCCESS
                    return

            except DockerException as e:
                logger.error(f"Error waiting for docker container status: {e}")

            time.sleep(self.config.interval)

        ctx.status = ExecutionStatus.TIMEOUT
        raise TimeoutError(
            f"Container {container_id} did not reach status '{desired_status}' within {self.config.timeout} seconds."
        )
