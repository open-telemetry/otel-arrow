"""
Module: config_strategy

This module defines the abstract base class `ConfigStrategy`, which is used to implement
pluggable configuration behaviors for components in the load generator testbed.

The `ConfigStrategy` interface allows components to be configured in a flexible and
extensible way. Implementations of this class should define how configuration is
applied to a given component, enabling support for various configuration mechanisms
(e.g., file-based config, environment variables, remote config services, etc.).

Typical concrete implementations of this interface might include:
    - FileConfig: Writes configs to a file on disk
    - ManifestConfig: Generates k8s manifests for application to a cluster
    - RemoteConfig: Reads configs from a remote location and applies them

Classes:
    ConfigStrategy (ABC): Abstract base class that declares the `start` method,
                          which must be implemented by all concrete configuration strategies.
"""

from abc import ABC, abstractmethod


class ConfigurationStrategy(ABC):
    """
    Abstract base class for component configuration strategies.

    Subclasses should implement the `start` method to define how configuration
    is applied to a given component.

    Methods:
        start(component): Apply configuration to the specified component.
    """

    @abstractmethod
    def start(self, component):
        """
        Start configuration for the given component.

        Args:
            component: The component instance to configure.
        """
