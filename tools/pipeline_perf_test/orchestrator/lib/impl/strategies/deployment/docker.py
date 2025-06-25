"""
Module: strategies.deployment.docker

A Docker-based deployment strategy for component lifecycle management in pipeline-perf environments.

This module defines a `DockerDeployment` strategy used to manage the lifecycle of test components
as Docker containers.

Classes:
    Strategy:
        - DockerDeployment: Main strategy class for deploying / destroying containers.
    Configuration:
        - BuildConfig: Configuration for Docker image builds.
        - PortMapping: Represents a container port mapping (host <-> container).
        - VolumeMapping: Represents a volume mount between host and container.
        - DockerDeploymentConfig: Deployment configuration used to create deployment instances.

Helper Functions:
    - build_volume_bindings(): Converts volume mount specs to Docker API format.
    - build_port_bindings(): Converts port mappings to Docker API format.
"""

import os
from typing import ClassVar, Dict, List, Literal, Optional, Tuple, Union

from docker.errors import DockerException
from pydantic import BaseModel, Field

from ....core.context.framework_element_contexts import StepContext
from ....core.component.component import (
    Component,
    HookableComponentPhase,
)
from ....core.strategies.deployment_strategy import (
    DeploymentStrategy,
    DeploymentStrategyConfig,
)

from ....runner.registry import deployment_registry, PluginMeta

from ..common.docker import (
    stop_and_remove_container,
    sanitize_docker_name,
    get_component_docker_runtime,
    set_component_docker_runtime_data,
    get_or_create_docker_client,
)
from ..hooks.docker.network import (
    CreateDockerNetwork,
    CreateDockerNetworkConfig,
    DeleteDockerNetwork,
    DeleteDockerNetworkConfig,
)
from ..hooks.docker.tidy_existing_container import (
    TidyExistingContainer,
    TidyExistingContainerConfig,
)
from ..hooks.docker.wait_for_status import (
    WaitForDockerStatus,
    WaitForDockerStatusConfig,
)

STRATEGY_NAME = "docker"


class DockerBuildConfig(BaseModel):
    """
    Configuration model for building a Docker image.

    Attributes:
        context (str): Path to the build context directory.
        dockerfile (Optional[str]): Optional path to the Dockerfile relative to the context.
        args (Optional[Dict[str, str]]): Optional build arguments passed to the Docker build.
        target (Optional[str]): Optional build stage target name for multi-stage builds.
    """
    context: str
    dockerfile: Optional[str] = None
    args: Optional[Dict[str, str]] = None
    target: Optional[str] = None


class DockerPortMapping(BaseModel):
    """
    Defines a port mapping between host and container.

    Attributes:
        published (Union[int, str]): Port on the host machine (can be numeric or string).
        target (Union[int, str]): Port inside the container (can be numeric or string).
        protocol (Literal["tcp", "udp"]): Network protocol, defaults to 'tcp'.
        host_ip (Optional[str]): Host IP address to bind the port to, defaults to '0.0.0.0'.
    """
    published: Union[int, str]
    target: Union[int, str]
    protocol: Literal["tcp", "udp"] = "tcp"
    host_ip: Optional[str] = "0.0.0.0"


class DockerVolumeMapping(BaseModel):
    """
    Defines a volume mapping between host and container.

    Attributes:
        source (str): Path on the host machine or named volume.
        target (str): Mount path inside the container.
        read_only (Optional[bool]): Whether the volume is mounted read-only, defaults to False.
    """
    source: str
    target: str
    read_only: Optional[bool] = False


@deployment_registry.register_config(STRATEGY_NAME)
class DockerDeploymentConfig(DeploymentStrategyConfig):
    """
    Configuration model for deploying a component using Docker.

    Attributes:
        image (str): Docker image name or tag to use for deployment.
        build (Optional[DockerBuildConfig]): Optional build configuration if the image
            needs to be built locally before deployment.
        environment (Optional[Union[Dict[str, str], List[str]]]): Environment variables to
            set in the container, either as a dictionary of key-value pairs or a list of
            strings in 'KEY=VALUE' format.
        command (Optional[Union[str, List[str]]]): Command or entrypoint override for the
            container, either as a string or a list of command arguments.
        ports (Optional[List[Union[str, DockerPortMapping]]]): List of port mappings to expose,
            either as strings or structured DockerPortMapping objects.
        volumes (Optional[List[Union[str, DockerVolumeMapping]]]): List of volume mounts,
            either as strings or structured DockerVolumeMapping objects.
        network (Optional[str]): Docker network to connect the container to.
    """
    image: str
    build: Optional[DockerBuildConfig] = Field(
        None, description="Build configuration if building locally"
    )
    environment: Optional[Union[Dict[str, str], List[str]]] = None
    command: Optional[Union[str, List[str]]] = None
    ports: Optional[List[Union[str, DockerPortMapping]]] = None
    volumes: Optional[List[Union[str, DockerVolumeMapping]]] = None
    network: Optional[str] = None


@deployment_registry.register_class(STRATEGY_NAME)
class DockerDeployment(DeploymentStrategy):
    """
    Deployment strategy to manage the lifecycle of components using Docker containers.

    This class handles starting and stopping Docker containers based on the given
    deployment configuration. It also registers default lifecycle hooks for Docker
    operations such as network creation, container cleanup, and status checks.

    Methods:
        start(component: Component, ctx: StepContext):
            Starts a Docker container for the specified component using the deployment
            configuration. Handles setting up the container with networking, volumes,
            ports, environment variables, and commands.

        stop(component: Component, ctx: StepContext):
            Stops and removes the Docker container associated with the component, using
            container ID stored in the component runtime. Raises errors if container
            cannot be found or stopped.
    """

    type: ClassVar[Literal["docker"]] = "docker"
    PLUGIN_META = PluginMeta(
        supported_contexts=[StepContext.__name__],
        installs_hooks=[CreateDockerNetwork.__name__, TidyExistingContainer.__name__, WaitForDockerStatus.__name__, DeleteDockerNetwork.__name__],
        yaml_example="""
components:
  otel-collector:
    deployment:
      docker:
        image: otel/opentelemetry-collector:latest
        network: testbed
        volumes:
          - ./system_under_test/otel-collector/collector-config-with-batch-processor.yaml:/etc/otel/collector-config.yaml:ro
        command: ["--config", "/etc/otel/collector-config.yaml"]
        ports:
          - "8888:8888"
"""
    )

    def __init__(self, config: DockerDeploymentConfig):
        """Initialize the strategy and specify default hooks to register."""
        self.config = config
        self.default_component_hooks = {
            HookableComponentPhase.PRE_DEPLOY: [
                # Build images is moved to live statically in the factory setup for now.
                # BuildDockerImage(BuildDockerImageConfig()),
                CreateDockerNetwork(CreateDockerNetworkConfig()),
                TidyExistingContainer(TidyExistingContainerConfig()),
            ],
            HookableComponentPhase.POST_DEPLOY: [
                WaitForDockerStatus(WaitForDockerStatusConfig())
            ],
            HookableComponentPhase.PRE_DESTROY: [],
            HookableComponentPhase.POST_DESTROY: [
                DeleteDockerNetwork(DeleteDockerNetworkConfig())
            ],
        }

    def start(self, component: Component, ctx: StepContext):
        """Start a container based on the provided configuration.

        Args:
            component: the component invoking this strategy.
            context: the current execution context.

        Raises:
            DockerException: on error starting docker container.
            ValueError: on incompatible configuration value.
            TypeError: on incompatilbe configuration type.
        """

        logger = ctx.get_logger(__name__)
        logger.debug(f"Starting Docker container for {component.name}")
        runtime = get_component_docker_runtime(ctx)
        client = get_or_create_docker_client(ctx)

        try:
            container = client.containers.run(
                image=self.config.image,
                name=sanitize_docker_name(component.name),
                detach=True,
                network=sanitize_docker_name(self.config.network),
                ports=build_port_bindings(self.config.ports),
                volumes=build_volume_bindings(self.config.volumes),
                environment=self.config.environment,
                command=self.config.command,
            )
        except DockerException as e:
            logger.error(f"Error launching Docker container: {e}")
            raise

        # Store the contianer runtime info on the component runtime registry.
        runtime.container_id = container.id
        set_component_docker_runtime_data(ctx, runtime)

    def stop(self, component: Component, ctx: StepContext):
        """Stop a container based on the component runtime container id.

        Args:
            component: the component invoking this strategy.
            context: the current execution context.

        Raises:
            RuntimeError: on error reading the container ID from the component runtime registry.
            docker.APIError: on error talking to the docker API
            docker.NotFound: on container id not found (e.g. stopped / removed externally)
        """
        logger = ctx.get_logger(__name__)
        runtime = get_component_docker_runtime(ctx)
        client = get_or_create_docker_client(ctx)
        logger.debug(
            f"Stopping Docker container for {component.name}, with ID: {runtime.container_id}"
        )
        container_id = runtime.container_id
        if not container_id:
            raise RuntimeError(
                f"No container ID found for component '{component.name}' - cannot stop container, it may not have started correctly."
            )
        stop_and_remove_container(ctx, client, container_id)


# Helpers


def build_volume_bindings(
    volume_mounts: Optional[List[Union[str, DockerVolumeMapping]]],
) -> Dict[str, Dict[str, str]]:
    """Map a list of VolumeMounts to the format expected by docker api"

    Args:
        - volume_mounts: a list of VolumeMapping objects or a string specifying mount configs
    """
    if not volume_mounts:
        return {}

    volume_dict = {}
    for vm in volume_mounts:
        if isinstance(vm, str):
            # Parse string format: /host:/container[:ro|rw]
            parts = vm.split(":")
            if len(parts) < 2 or len(parts) > 3:
                raise ValueError(f"Invalid volume mount string: '{vm}'")

            host_path = os.path.abspath(parts[0])
            container_path = parts[1]
            mode = parts[2] if len(parts) == 3 else "rw"

            if mode not in ("ro", "rw"):
                raise ValueError(f"Invalid mode in volume mount string: '{vm}'")
        elif isinstance(vm, DockerVolumeMapping):
            host_path = os.path.abspath(vm.source)
            container_path = vm.target
            mode = "ro" if vm.read_only else "rw"
        else:
            raise TypeError(f"Invalid type in volume_mounts: {type(vm)}")

        volume_dict[host_path] = {"bind": container_path, "mode": mode}
    return volume_dict


def build_port_bindings(
    bindings: Optional[List[Union[DockerPortMapping, str]]],
) -> Dict[str, Tuple[str, int]]:
    """Map PortBinding objects or strings to Docker SDK format.

    Args:
        bindings: List of PortBinding objects or strings like "8080:80", "127.0.0.1:8080:80/udp"

    Returns:
        dict: Mapping of "container_port/protocol" -> (host_ip, host_port)
    """
    if not bindings:
        return {}

    binding_dict = {}
    for b in bindings:
        if isinstance(b, str):
            # Parse string format: [host_ip:]host_port:container_port[/protocol]
            # Examples: "8080:80", "127.0.0.1:8080:80", "8080:80/udp"
            protocol = "tcp"

            if "/" in b:
                b, protocol = b.rsplit("/", 1)

            parts = b.split(":")

            if len(parts) == 2:
                host_ip = "0.0.0.0"
                host_port, container_port = parts
            elif len(parts) == 3:
                host_ip, host_port, container_port = parts
            else:
                raise ValueError(f"Invalid port mapping string: {b}")

            key = f"{container_port}/{protocol}"
            binding_dict[key] = (host_ip, int(host_port))

        elif isinstance(b, DockerPortMapping):
            key = f"{b.target}/{b.protocol}"
            binding_dict[key] = (b.host_ip or "0.0.0.0", int(b.published))

        else:
            raise TypeError(f"Invalid type in bindings list: {type(b)}")

    return binding_dict
