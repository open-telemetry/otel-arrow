"""Resolve an existing Docker container without modifying it."""

from pydantic import Field

from ....core.component.component import Component
from ....core.context.framework_element_contexts import StepContext
from ....core.strategies.existing_strategy import (
    ExistingStrategy,
    ExistingStrategyConfig,
)
from ....runner.registry import existing_registry, PluginMeta
from ..common.docker import ComponentDockerRuntime, get_or_create_docker_client


@existing_registry.register_config("docker")
class ExistingDockerConfig(ExistingStrategyConfig):
    """Select a running container by name or ID."""

    container: str = Field(min_length=1)


@existing_registry.register_class("docker")
class ExistingDocker(ExistingStrategy):
    """Resolve a running Docker container for observation without owning it."""

    PLUGIN_META = PluginMeta(
        supported_contexts=[StepContext.__name__],
        installs_hooks=[],
        yaml_example="""
components:
  existing-service:
    existing:
      docker:
        container: my-service
    monitoring:
      docker_component:
        interval: 1
""",
    )

    def __init__(self, config: ExistingDockerConfig):
        self.config = config

    def resolve(self, component: Component, ctx: StepContext):
        client = get_or_create_docker_client(ctx)
        container = client.containers.get(self.config.container)
        if container.status != "running":
            raise ValueError(
                f"Existing container {self.config.container!r} is not running: "
                f"{container.status}"
            )
        if not container.id:
            raise ValueError(f"Existing container {self.config.container!r} has no ID")
        component.set_runtime_data(
            ComponentDockerRuntime.type,
            ComponentDockerRuntime(container_id=container.id),
        )
        ctx.get_logger(__name__).info(
            "Observing existing Docker container %s (%s)",
            self.config.container,
            container.id,
        )
