from typing import Dict, List, Literal, Optional, Union

import docker
from pydantic import BaseModel, Field

from ....core.strategies.deployment_strategy import DeploymentStrategy
from ....core.context.test_contexts import TestStepContext
from ....core.component.lifecycle_component import (
    LifecycleHookContext,
    HookableLifecyclePhase,
)


class DockerRuntime(BaseModel):
    container_id: Optional[str] = None


class BuildConfig(BaseModel):
    """Base Model for docker container build settings."""

    context: str
    dockerfile: Optional[str] = None
    args: Optional[Dict[str, str]] = None
    target: Optional[str] = None


class PortMapping(BaseModel):
    """Base Model for docker container port settings."""

    published: Union[int, str]
    target: Union[int, str]


class VolumeMapping(BaseModel):
    """Base Model for docker container volume settings."""

    source: str
    target: str
    read_only: Optional[bool] = False


class DockerDeploymentConfig(BaseModel):
    """Base Model for docker docker deployment settings."""

    type: Literal["docker"]
    build: Optional[BuildConfig] = Field(
        None, description="Build configuration if building locally"
    )
    image: Optional[str] = Field(None, description="Docker image to use")
    environment: Optional[Dict[str, str]] = None
    ports: Optional[List[Union[str, PortMapping]]] = None
    volumes: Optional[List[Union[str, VolumeMapping]]] = None
    network: Optional[str] = None

    # TODO: move this to use a factory / registry.
    def create(self) -> DeploymentStrategy:
        return DockerDeployment(config=self)


class DockerDeployment(DeploymentStrategy):
    """Manages component container lifecycle using docker on the local host."""

    def __init__(self, config: DockerDeploymentConfig):
        self.config = config
        self.default_hooks = {
            HookableLifecyclePhase.PRE_DEPLOY: [
                build_docker_image,
                create_docker_network,
            ],
            HookableLifecyclePhase.PRE_DESTROY: [get_docker_logs],
            HookableLifecyclePhase.POST_DESTROY: [delete_docker_network],
        }

    def start(self, component, context: TestStepContext):
        # dummy impl
        context.log(f"Starting Docker container for {component.name}")
        runtime = get_docker_runtime(context)
        runtime.container_id = "123456789"
        set_docker_runtime_data(context, runtime)

    def stop(self, component, context: TestStepContext):
        # dummy impl
        runtime = get_docker_runtime(context)
        context.log(
            f"Stopping Docker container for {component.name}, with ID: {runtime.container_id}"
        )


def create_docker_client() -> object:
    """Initialize a docker client from the environment"""
    return docker.from_env()


def get_docker_config(ctx: LifecycleHookContext) -> Optional[DockerDeploymentConfig]:
    """Get the deployment strategy config for the component and ensure it's a docker deployment.

    Args:
        ctx: The context for the hook that's currently firing.

    Returns: The DockerDeploymentConfig instance if valid, else None.
    """
    component = ctx.get_step_component()
    if not component:
        ctx.log("No component found in context.")
        return None
    deployment = getattr(component.component_config, "deployment", None)
    if not isinstance(deployment, DockerDeploymentConfig):
        ctx.log(
            f"Deployment config is not DockerDeploymentConfig: {type(deployment).__name__}"
        )
        return None

    return deployment


def get_docker_runtime(
    ctx: Union[LifecycleHookContext, TestStepContext],
) -> DockerRuntime:
    """Get runtime docker information from the context.

    Args:
        ctx: The current context

    Returns: The existing docker runtime or a new one"""
    component = ctx.get_step_component()
    if not component:
        ctx.log("No component found in context.")
        return None
    return component.get_or_create_runtime("docker", DockerRuntime)


def set_docker_runtime_data(
    ctx: Union[LifecycleHookContext, TestStepContext], data: DockerRuntime
):
    """Get runtime docker information from the context.

    Args:
        ctx: The current context

    Returns: The existing docker runtime or a new one"""
    component = ctx.get_step_component()
    if not component:
        ctx.log("No component found in context.")
        return None
    return component.set_runtime_data("docker", data)


def build_docker_image(ctx: LifecycleHookContext):
    _client = ctx.get_client("docker", create_docker_client)
    conf = get_docker_config(ctx)
    if not conf:
        return
    # dummy impl
    ctx.log(f"docker build -t {conf.image} (other args available in conf.build) )")


def create_docker_network(ctx: LifecycleHookContext):
    _client = ctx.get_client("docker", create_docker_client)
    conf = get_docker_config(ctx)
    if not conf:
        return
    # dummy impl
    ctx.log(f"docker network create: {conf.deployment.network}")


def delete_docker_network(ctx: LifecycleHookContext):
    _client = ctx.get_client("docker", create_docker_client)
    conf = get_docker_config(ctx)
    if not conf:
        return
    if not conf.network:
        ctx.log("Default network in use, skip removal")
        return
    # dummy impl
    ctx.log(f"docker network rm: {conf.network}")


def get_docker_logs(ctx: LifecycleHookContext):
    # dummy impl
    runtime = get_docker_runtime(ctx)
    ctx.log(f"docker logs {runtime.container_id}")
