"""
Docker utilities and runtime management for component deployment strategies.

This module provides common Docker-related abstractions and helper functions used
across deployment and execution strategies that manage Docker containers in the
system.

Key components include:

- Data models representing Docker runtime state at both global (client) and
  component (container) levels (`GlobalDockerRuntime`, `ComponentDockerRuntime`).

- Enumeration of Docker container statuses (`DockerContainerStatus`).

- Utilities to sanitize Docker container names to meet Docker's naming rules.

- Functions to create and manage Docker clients (`create_docker_client`,
  `get_or_create_docker_client`).

- Container lifecycle helpers, such as stopping and removing containers safely
  (`stop_and_remove_container`).

- Context helpers to retrieve or set Docker runtime data within the current
  execution context (`get_global_docker_runtime`, `set_global_docker_runtime_data`,
  `get_component_docker_runtime`, `set_component_docker_runtime_data`).

The module depends on the official Docker Python SDK (`docker`), Pydantic for
runtime data modeling, and integrates tightly with the framework's context and
component abstractions.

Typical usage involves managing container lifecycles, persisting runtime state
across execution steps, and interfacing with Docker for local container orchestration.
"""

import re
from enum import Enum
from typing import ClassVar, Literal, Optional, Union

import docker
from docker.errors import APIError, NotFound
from pydantic import BaseModel, ConfigDict

from ....core.framework.suite import Suite
from ....core.context.base import BaseContext
from ....core.context.framework_element_contexts import StepContext
from ....core.component.component import (
    Component,
    ComponentHookContext,
)


class ComponentDockerRuntime(BaseModel):
    """Base Model for component (container level) docker runtime information."""

    type: ClassVar[Literal["component_docker_runtime"]] = "component_docker_runtime"

    container_id: Optional[str] = None
    container_logs: Optional[list[str]] = None
    network_created: Optional[bool] = False


class GlobalDockerRuntime(BaseModel):
    """Base Model for globbal docker runtime information."""

    type: ClassVar[Literal["global_docker_runtime"]] = "global_docker_runtime"

    client: Optional[docker.DockerClient] = None
    model_config = ConfigDict(arbitrary_types_allowed=True)


class DockerContainerStatus(str, Enum):
    """Represents possible container states."""

    CREATED = "created"
    RUNNING = "running"
    PAUSED = "paused"
    RESTARTING = "restarting"
    REMOVING = "removing"
    EXITED = "exited"
    DEAD = "dead"


def sanitize_docker_name(name: Optional[str]) -> Optional[str]:
    """
    Sanitize a string to be a valid Docker container name.

    - Lowercase
    - Replace invalid characters with hyphens
    - Remove leading/trailing hyphens
    - Enforce Docker name length restriction

    Args:
        name: the name string to sanitize

    Returns: a representation of the original string that satisfies docker naming conventions.
    """
    if not name:
        return None
    name = name.lower()
    name = re.sub(r"[^a-z0-9_.-]", "-", name)
    name = re.sub(r"-{2,}", "-", name)
    name = name.strip("-")
    name = name[:255]
    return name


def create_docker_client() -> docker.DockerClient:
    """Initialize a docker client from the environment"""
    return docker.from_env()


def stop_and_remove_container(
    ctx: Union[StepContext, ComponentHookContext],
    client: docker.DockerClient,
    container_id: str,
):
    """Stop and remove a running container

    Args:
        ctx: The current context
        client: The docker API client to use
        container_id: The id of the running container to stop and remove.
    """
    logger = ctx.get_logger(__name__)
    try:
        container = client.containers.get(container_id)
        container.stop(timeout=10)  # default is 10 seconds
        container.remove(force=True)
    except NotFound:
        logger.debug(
            f"Container {container_id} not found. It may have already been removed."
        )
        raise
    except APIError as e:
        logger.error(f"Error stopping/removing Docker container: {e}")
        raise


def get_container_logs(
    ctx: Union[StepContext, ComponentHookContext],
    client: docker.DockerClient,
    runtime: ComponentDockerRuntime,
):
    """Get docker container logs and save them to the runtime.

    Args:
        ctx: The current context
        client: The docker API client to use
        runtime: ComponentDockerRuntime for the container
    """
    logger = ctx.get_logger(__name__)
    args = ctx.get_suite().get_runtime("args")
    try:
        container = client.containers.get(runtime.container_id)
        logs = container.logs(stdout=True, stderr=True, stream=False, timestamps=False)
        decoded = logs.decode("utf-8") if isinstance(logs, bytes) else str(logs)
        if args.debug:
            logger.debug("Container Logs For %s:\n%s", runtime.container_id, decoded)
        runtime.container_logs = decoded.splitlines()
        set_component_docker_runtime_data(ctx, runtime)
    except (NotFound, APIError) as e:
        logger.error(f"Error getting Docker container logs: {e}")
        raise


def get_global_docker_runtime(ctx: BaseContext) -> GlobalDockerRuntime:
    """Get runtime docker information from the context.

    Args:
        ctx: The current context

    Returns: The existing global docker runtime or a new one"""
    ts = ctx.get_suite()
    assert isinstance(ts, Suite), "Expected TestSuite"
    return ts.get_or_create_runtime(GlobalDockerRuntime.type, GlobalDockerRuntime)


def set_global_docker_runtime_data(ctx: BaseContext, data: GlobalDockerRuntime):
    """Set global runtime docker information to the TestSuite.

    Args:
        ctx: The current context
        data: The GlobalDockerRuntime data to set.
    """
    ts = ctx.get_suite()
    assert isinstance(ts, Suite), "Expected TestSuite"
    return ts.set_runtime_data(GlobalDockerRuntime.type, data)


def get_or_create_docker_client(ctx: BaseContext) -> docker.DockerClient:
    runtime = get_global_docker_runtime(ctx)
    if not runtime.client:
        runtime.client = docker.from_env()
        set_global_docker_runtime_data(ctx, runtime)
    return runtime.client


def get_component_docker_runtime(
    ctx: Union[ComponentHookContext, StepContext],
) -> ComponentDockerRuntime:
    """Get runtime docker information from the context.

    Args:
        ctx: The current context

    Returns: The existing docker runtime or a new one"""
    component = ctx.get_step_component()
    assert isinstance(component, Component), "Expected Component"
    return component.get_or_create_runtime(
        ComponentDockerRuntime.type, ComponentDockerRuntime
    )


def set_component_docker_runtime_data(
    ctx: Union[ComponentHookContext, StepContext], data: ComponentDockerRuntime
):
    """Get runtime docker information from the context.

    Args:
        ctx: The current context

    Returns: The existing docker runtime or a new one"""
    logger = ctx.get_logger(__name__)
    component = ctx.get_step_component()
    if not component:
        logger.warning("No component found in context.")
        return None
    return component.set_runtime_data(ComponentDockerRuntime.type, data)
