from typing import Optional
from docker.errors import APIError
from .....core.context.base import ExecutionStatus, BaseContext
from .....core.context import (ComponentHookContext, FrameworkElementHookContext)
from .....core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from .....runner.registry import hook_registry, PluginMeta
from ...common.docker import get_or_create_docker_client, stop_and_remove_container


# TODO: These all assume this is a lifecycle hook event on a component. Need to support test element hooks.
@hook_registry.register_config("tidy_existing_container")
class TidyExistingContainerConfig(HookStrategyConfig):
    """
    Configuration for the 'tidy_existing_container' hook.
    """
    component: Optional[str] = None


@hook_registry.register_class("tidy_existing_container")
class TidyExistingContainer(HookStrategy):
    """
    Hook strategy to remove an existing Docker container with the same name as the current component.

    This hook is useful during the pre-deployment phase of a component lifecycle to ensure that
    any previously running container (possibly left over from an earlier run) is cleaned up before
    starting a new one. This avoids container name conflicts and ensures consistent environment setup.

    Behavior:
        - If a container is found with the same name as the component, it is stopped and removed.
        - If no container is found (404), the hook is skipped silently.
        - If no component is available in the context, the hook fails.

    Hook status:
        - FAILURE if no component is found in context.
        - SKIPPED if the container is not found or no action is needed.

    Raises:
        docker.errors.APIError: Only if an unexpected Docker API issue occurs (currently suppressed for 404s).

    Typical usage:
        - Pre-deployment cleanup to avoid naming conflicts.
        - Resetting environment between pipeline/test runs where container reuse is not desired.
    """
    PLUGIN_META = PluginMeta(
        supported_contexts=[ComponentHookContext.__name__, FrameworkElementHookContext.__name__],
        installs_hooks=[],
        notes="This hook is automatically installed by the docker deployment strategy and doesn't need to be added explicitly",
        yaml_example="""
components:
  otel-collector:
    deployment:
      docker: ...
    hooks:
        deploy:
            pre:
            - tidy_existing_container: {}
"""
    )

    def __init__(self, config: TidyExistingContainerConfig):
        self.config = config

    def execute(self, ctx: BaseContext):
        logger = ctx.get_logger(__name__)
        client = get_or_create_docker_client(ctx)

        component = self.config.component
        if not component:
            component = ctx.get_step_component()
            if not component:
                logger.warning("No component found in context.")
                ctx.status = ExecutionStatus.FAILURE
                return None
        try:
            container = client.containers.get(component.name)
            if container:
                stop_and_remove_container(ctx, client, container.id)
        except APIError:
            # A missing container throws a 404 error, skip since nothing to do.
            ctx.status = ExecutionStatus.SKIPPED
