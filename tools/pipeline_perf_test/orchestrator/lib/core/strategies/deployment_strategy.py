"""
Module: deployment_strategy

This module defines the abstract base class `DeploymentStrategy`, which provides
a common interface for managing the lifecycle of components in the load generator testbed.

The `DeploymentStrategy` interface enables pluggable deployment mechanisms, allowing
different deployment backends (e.g., Docker, Kubernetes, local processes) to be used
interchangeably. This design supports extensibility and modular deployment logic.

Typical concrete implementations of this interface include:
    - DockerDeployment: Deploys and manages components in Docker containers.
    - K8sDeployment: Deploys components as Kubernetes Pods or Deployments.
    - ProcessDeployment: Manages components as local OS processes.

Classes:
    DeploymentStrategyConfig (BaseModel): Base class for strategy configs.
    DeploymentStrategy (ABC): Abstract base class for defining component deployment behavior.
"""

from abc import abstractmethod
from typing import TYPE_CHECKING

from ..context.test_contexts import TestStepContext
from .base import BaseStrategyConfig, BaseStrategy


if TYPE_CHECKING:
    from ..component.component import Component


class DeploymentStrategyConfig(BaseStrategyConfig):
    """Base model for Deployment Strategy config, passed to strategy init."""


class DeploymentStrategy(BaseStrategy):

    @abstractmethod
    def __init__(self, config: DeploymentStrategyConfig) -> None:
        """All deployment strategies must be initialized with a config object."""

    @abstractmethod
    def start(self, component: "Component", ctx: TestStepContext):
        """
        Deploy the component to the target environment.

        Args:
            component: The component instance to deploy.
            ctx: The current execution context for the containing test step.
        """

    @abstractmethod
    def stop(self, component: "Component", ctx: TestStepContext):
        """
        Tear down and remove the deployed component.

        Args:
            component: The component instance to destroy.
            ctx: The current execution context for the containing test step.
        """
