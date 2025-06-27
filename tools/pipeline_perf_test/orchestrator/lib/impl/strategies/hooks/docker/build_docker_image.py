"""
Docker Image Build Hook Strategies

This module provides hook strategies to build Docker images locally as part of
a component deployment workflow. It supports building either multiple Docker images
defined in a list of components or a single Docker image associated with a specified component.

Key features:
- BuildDockerImages: Builds Docker images for multiple components with Docker build configs.
- BuildDockerImage: Builds a Docker image for a single component, identified dynamically or explicitly.
- Shared utility function to invoke Docker API image build, handling logs and errors.

The strategies integrate with the execution context to fetch components and deployment
configurations, log build progress, and handle errors via the Docker SDK for Python.

Exceptions:
- Raises docker.errors.BuildError and docker.errors.APIError on build failures.
"""

import os
from typing import Optional, List, TYPE_CHECKING

from docker import DockerClient
from pydantic import Field

from docker.errors import BuildError, APIError
from .....core.context.base import ExecutionStatus, BaseContext
from .....core.context import ComponentHookContext, FrameworkElementHookContext
from .....core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from .....impl.component.managed_component import (
    ManagedComponent,
)
from .....runner.registry import hook_registry, PluginMeta, CliFlag
from ...common.docker import get_or_create_docker_client


if TYPE_CHECKING:
    from ....strategies.deployment.docker import DockerBuildConfig


@hook_registry.register_config("build_docker_images")
class BuildDockerImagesConfig(HookStrategyConfig):
    """
    Configuration for the BuildDockerImages hook strategy.

    Attributes:
        log_build (Optional[bool]): Whether to enable logging of the docker build process. Defaults to False.
        components (Optional[List[str]]): List of component names to build Docker images for.
            If empty or not specified, all applicable components in the context will be considered.
    """

    log_build: Optional[bool] = False
    components: Optional[List[str]] = Field(default_factory=list)


@hook_registry.register_class("build_docker_images")
class BuildDockerImages(HookStrategy):
    """
    Hook strategy to build Docker images for multiple components locally.

    This strategy builds Docker images for specified components or,
    if no components are specified, for all applicable managed components
    with Docker deployment configurations that include a build section.

    Attributes:
        config (BuildDockerImagesConfig): Configuration for this build strategy.
    """

    PLUGIN_META = PluginMeta(
        supported_contexts=[
            ComponentHookContext.__name__,
            FrameworkElementHookContext.__name__,
        ],
        installs_hooks=[],
        cli_flags=[
            CliFlag(
                group="Docker Options",
                flag="--docker.no-build",
                dest="docker_no_build",
                help="Skip build of Docker containers.",
                action="store_true",
                default=False,
            ),
        ],
        yaml_example="""
hooks:
  run:
    pre:
      - build_docker_images:
          # Omit components to build any docker component with a build section.
          components:
            - load-generator
            - backend-service
          log_build: false
""",
    )

    def __init__(self, config: BuildDockerImagesConfig):
        self.config = config

    def execute(self, ctx: BaseContext):
        """
        Build multiple docker images locally

        Args:
            ctx: The current execution context from which the build configuration is fetched.

        Returns:
            Optional[List[str]]: Name of the built image or none if none built.

        Raises:
            docker.errors.BuildError: If the container cannot be found.
            docker.errors.APIError: If there is an error communicating with the docker server.
        """
        from ....strategies.deployment.docker import DockerDeploymentConfig

        logger = ctx.get_logger(__name__)
        client = get_or_create_docker_client(ctx)
        args = ctx.get_suite().get_runtime("args")
        if args.docker_no_build:
            ctx.status = ExecutionStatus.SKIPPED
            return
        buildable_components = self.config.components
        if not buildable_components:
            components = ctx.get_components()
            for _, component in components.items():
                if not isinstance(component, ManagedComponent):
                    continue
                conf = component.get_deployment_config()
                if not conf:
                    continue
                if isinstance(conf, DockerDeploymentConfig) and conf.build:
                    logger.debug(f"Building Docker image '{conf.image}'...")
                    build_image(client, conf, logger, log_build=self.config.log_build)


@hook_registry.register_config("build_docker_image")
class BuildDockerImageConfig(HookStrategyConfig):
    """
    Configuration schema for the 'build_docker_image' hook strategy.

    This configuration is used to control the behavior of building a single Docker image
    for a specified component.

    Attributes:
        log_build (Optional[bool]): If True, logs the output of the Docker build process.
            Defaults to False.
        component (Optional[str]): The name of the component whose Docker image should be built.
            If not provided, the component may be inferred from context (e.g., in a step or hook).
    """

    log_build: Optional[bool] = False
    component: Optional[str] = None


@hook_registry.register_class("build_docker_image")
class BuildDockerImage(HookStrategy):
    """
    Hook strategy implementation for building a single Docker image.

    This class is responsible for executing the Docker build process for a specified
    component. It can be used in test steps or hook phases to ensure that a Docker image
    is built prior to component execution.

    Behavior:

    - Uses the component name from the configuration if specified.
    - Falls back to resolving the component from the execution context if not explicitly provided.
    - Skips execution if the component is missing, not a ManagedComponent, or lacks a Docker build config.

    Args:
        config (BuildDockerImageConfig): The configuration object containing build options.
    """

    PLUGIN_META = PluginMeta(
        supported_contexts=[
            ComponentHookContext.__name__,
            FrameworkElementHookContext.__name__,
        ],
        installs_hooks=[],
        yaml_example="""
hooks:
  run:
    pre:
      - build_docker_image:
          # Omit components to build any docker component with a build section.
          component: load-generator
          log_build: false
""",
    )

    def __init__(self, config: BuildDockerImageConfig):
        self.config = config

    def execute(self, ctx: BaseContext):
        """
        Build a Docker image locally

        Args:
            ctx: The current execution context from which the build configuration is fetched.

        Returns:
            str: Name of the built image or none if none built.

        Raises:
            docker.errors.BuildError: If the container cannot be found.
            docker.errors.APIError: If there is an error communicating with the docker server.
        """

        logger = ctx.get_logger(__name__)
        args = ctx.get_suite().get_runtime("args")
        if args.docker_no_build:
            ctx.status = ExecutionStatus.SKIPPED
            return
        client = get_or_create_docker_client(ctx)
        component_name = None
        if self.config.component:
            component_name = self.config.component

        # If it's a TestStep or ComponentHook, can fetch dynamically.
        if not component_name and hasattr(ctx, "get_step_component"):
            component = ctx.get_step_component()
            component_name = component.name

        if not component_name:
            logger.warning(
                "Failed to determine component name: %s", component.__class__.__name__
            )
            ctx.status = ExecutionStatus.SKIPPED
            return

        component = ctx.get_component_by_name(component_name)
        if not isinstance(component, ManagedComponent):
            logger.warning(
                "Component type not supported for build docker image: %s",
                component.__class__.__name__,
            )
            ctx.status = ExecutionStatus.SKIPPED
            return
        conf = component.get_deployment_config()
        if not conf:
            ctx.status = ExecutionStatus.SKIPPED
            return

        if not conf.build:
            ctx.status = ExecutionStatus.SKIPPED
            return
        logger.debug(f"Building Docker image '{conf.image}'...")
        return build_image(client, conf, logger, log_build=self.config.log_build)


def build_image(
    client: DockerClient, config: "DockerBuildConfig", logger, log_build: bool
):
    """
    Build a Docker image using the specified build configuration.

    Args:
        client (DockerClient): The Docker client instance used to perform the build.
        config (DockerBuildConfig): Configuration details for the Docker build,
            including context path, Dockerfile, build arguments, and target stage.
        logger: Logger instance to output build logs and messages.
        log_build (bool): Whether to log detailed build output.

    Returns:
        str: The tag/name of the successfully built Docker image.

    Raises:
        docker.errors.BuildError: If there is an error during the Docker build process.
        docker.errors.APIError: If there is an API error communicating with the Docker daemon.
    """
    dir_path = os.path.abspath(config.build.context)
    try:
        image, build_logs = client.images.build(
            path=dir_path,
            tag=config.image,
            rm=True,
            buildargs=config.build.args,
            target=config.build.target,
        )

        if log_build:
            for chunk in build_logs:
                if "stream" in chunk:
                    logger.debug(chunk["stream"].strip())

        logger.debug(f"Successfully built Docker image: {image.tags}")
        return config.image
    except (BuildError, APIError) as e:
        logger.debug(f"Error building Docker image '{config.image}': {e}")
        raise
