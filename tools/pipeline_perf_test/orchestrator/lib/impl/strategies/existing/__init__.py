"""Read-only strategies for caller-owned resources."""

from . import docker, kubernetes

__all__ = ["docker", "kubernetes"]
