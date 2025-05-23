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
    DeploymentStrategy (ABC): Abstract base class for defining component deployment behavior.
"""

from abc import ABC, abstractmethod


class DeploymentStrategy(ABC):
    @abstractmethod
    def start(self, component):
        """
        Deploy the component to the target environment.

        Args:
            component: The component instance to deploy.
        """

    @abstractmethod
    def stop(self, component):
        """
        Tear down and remove the deployed component.

        Args:
            component: The component instance to destroy.
        """
