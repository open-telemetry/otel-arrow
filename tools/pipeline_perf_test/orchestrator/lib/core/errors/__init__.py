"""Initialization for the core.errors package."""

from .error_handler import OnErrorConfig, handle_with_policy

__all__ = ["OnErrorConfig", "handle_with_policy"]
