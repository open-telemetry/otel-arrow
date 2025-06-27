from typing import Optional
from docker.errors import NotFound, APIError
from .....core.context.base import ExecutionStatus, BaseContext
from .....core.context import ComponentHookContext, FrameworkElementHookContext
from .....core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from .....runner.registry import hook_registry, PluginMeta
from ....component.managed_component import ManagedComponent
from ...common.docker import (
    get_component_docker_runtime,
    get_or_create_docker_client,
    set_component_docker_runtime_data,
)


@hook_registry.register_config("create_docker_network")
class CreateDockerNetworkConfig(HookStrategyConfig):
    """
    Configuration for the 'create_docker_network' hook.

    Attributes:
        network (Optional[str]): The name of the Docker network to create. If not provided,
            the network name will be inferred from the component's deployment configuration.
    """

    network: Optional[str] = None


@hook_registry.register_class("create_docker_network")
class CreateDockerNetwork(HookStrategy):
    """
    Hook strategy to create a Docker network for a component if it does not already exist.

    This hook ensures that a Docker network is available for the component prior to
    deployment. It first checks whether the specified or inferred network already exists;
    if not, it creates the network and marks it in the component's runtime so it can be
    cleaned up later if needed.

    Network name resolution:
        - If explicitly provided in the config, that name is used.
        - If not provided, the network name is inferred from the component's deployment config.
        - If no network name can be determined, the hook is skipped.

    Hook status:
        - SKIPPED if no network is needed or it already exists.
        - FAILURE if Docker API errors occur during creation.

    Raises:
        docker.errors.APIError: If the Docker daemon encounters an error during network creation.

    Typical usage:
        - Automatically creating isolated Docker networks in test environments.
        - Supporting component-based Docker orchestration with isolated networking.
    """

    PLUGIN_META = PluginMeta(
        supported_contexts=[
            ComponentHookContext.__name__,
            FrameworkElementHookContext.__name__,
        ],
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
            - create_docker_network:
                network: foo-network
""",
    )

    def __init__(self, config: CreateDockerNetworkConfig):
        self.config = config

    def execute(self, ctx: BaseContext):
        logger = ctx.get_logger(__name__)
        client = get_or_create_docker_client(ctx)

        network_name = self.config.network

        if not network_name:
            if hasattr(ctx, "get_step_component"):
                component = ctx.get_step_component()
                if not isinstance(component, ManagedComponent):
                    return
                conf = component.get_deployment_config()
                if not conf:
                    ctx.status = ExecutionStatus.SKIPPED
                    return
                network_name = conf.network if conf else None

        # Use the default network
        if not network_name:
            ctx.status = ExecutionStatus.SKIPPED
            return

        try:
            # Check if the network already exists
            existing_networks = client.networks.list(names=[network_name])
            if existing_networks:
                logger.debug(f"Using existing Docker network: {network_name}")
                ctx.status = ExecutionStatus.SKIPPED
                return

            client.networks.create(name=network_name)
            runtime = get_component_docker_runtime(ctx)

            # Record that the network is created and needs to be destroyed.
            runtime.network_created = True
            set_component_docker_runtime_data(ctx, runtime)
            logger.debug(f"Created Docker network: {network_name}")
        except APIError as e:
            logger.error(f"Error creating network: {e}")
            raise


@hook_registry.register_config("delete_docker_network")
class DeleteDockerNetworkConfig(HookStrategyConfig):
    """
    Configuration for the 'delete_docker_network' hook.

    Attributes:
        network (Optional[str]): The name of the Docker network to delete. If not explicitly
            provided, the network name is inferred from the component's deployment config.
    """

    network: Optional[str] = None


@hook_registry.register_class("delete_docker_network")
class DeleteDockerNetwork(HookStrategy):
    """
    Hook strategy to delete a Docker network associated with a component.

    This hook is typically run during the teardown (post-destroy) phase of a component's
    lifecycle. It attempts to remove the specified Docker network, but only if the network
    was created during execution (i.e., not a pre-existing or default network).

    Network name resolution:
        - If explicitly specified in the config, that name is used.
        - If not provided, it is inferred from the component's deployment configuration.
        - If no network name can be resolved, the hook is skipped.

    Runtime checks:
        - The network is only removed if `network_created` is set in the component's
          Docker runtime. This prevents accidental deletion of shared or external networks.

    Hook status:
        - SKIPPED if no valid network is found or deletion is unnecessary.
        - FAILURE if the network is expected but cannot be found.
        - Raises if the Docker daemon returns an API error.

    Raises:
        docker.errors.APIError: If there is a problem communicating with the Docker daemon.
        docker.errors.NotFound: If the specified Docker network does not exist.

    Typical usage:
        - Cleaning up isolated Docker networks created during automated tests or temporary deployments.
        - Ensuring that dynamically created networks do not persist beyond their intended lifecycle.
    """

    PLUGIN_META = PluginMeta(
        supported_contexts=[
            ComponentHookContext.__name__,
            FrameworkElementHookContext.__name__,
        ],
        installs_hooks=[],
        notes="This hook is automatically installed by the docker deployment strategy and doesn't need to be added explicitly",
        yaml_example="""
components:
  otel-collector:
    deployment:
      docker: ...
    hooks:
        destroy:
            post:
            - delete_docker_network:
                network: foo-network
""",
    )

    def __init__(self, config: DeleteDockerNetworkConfig):
        self.config = config

    def execute(self, ctx: BaseContext):
        """
        Deletes a Docker network using the Docker API.

        Args:
            ctx: the current execution context

        Raises:
            docker.errors.APIError: If there is an error communicating with the docker server.
            docker.errors.NotFound: If the network cannot be found.
        """
        logger = ctx.get_logger(__name__)
        client = get_or_create_docker_client(ctx)

        network_name = self.config.network

        if not network_name:
            if hasattr(ctx, "get_step_component"):
                component = ctx.get_step_component()
                if not isinstance(component, ManagedComponent):
                    ctx.status = ExecutionStatus.SKIPPED
                    return
                conf = component.get_deployment_config()
                if not conf:
                    ctx.status = ExecutionStatus.SKIPPED
                    return
                network_name = conf.network if conf else None

        # Use the default network
        if not network_name:
            ctx.status = ExecutionStatus.SKIPPED
            return

        runtime = get_component_docker_runtime(ctx)
        if not network_name or not runtime.network_created:
            logger.debug("Default or pre-existing network in use, skip removal")
            ctx.status = ExecutionStatus.SKIPPED
            return
        try:
            network = client.networks.get(network_name)
            # TODO: Get runtime flags into context to support this.
            # if log_cli:
            #     print(f"CLI_COMMAND = docker network rm -f {network_name}")
            network.remove()
            logger.debug(f"Deleted Docker network: {network_name}")
        except NotFound:
            logger.error(f"Docker network '{network_name}' not found.")
            ctx.status = ExecutionStatus.FAILURE
            ctx.error = f"Docker network '{network_name}' not found."
        except APIError as e:
            logger.error(f"Error removing Docker network: {e}")
            raise
