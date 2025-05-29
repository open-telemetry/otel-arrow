"""
runtime.py

This module defines the `ComponentRuntime` class, which provides a flexible and
plugin-extensible mechanism for storing runtime information associated with a
component during its lifecycle.

Runtime data is namespaced using string keys (e.g., strategy names), allowing
different strategies or plugins to attach their own execution-specific state
without interference.

Key features:
- Dynamic namespacing of runtime data
- Lazy initialization via `get_or_create`
- Supports arbitrary user-defined data structures

Typical usage:
    runtime.set("monitoring", {"latency_ms": 120})
    state = runtime.get_or_create("deployment", lambda: {})

This is commonly used within lifecycle components to track execution-time metadata
across multiple strategies.
"""

from typing import Callable, Dict, Any


class ComponentRuntime:
    """Holds runtime info for a component in a plugin-extensible way."""

    _data: Dict[str, Any]  # strategy_name -> plugin-defined state

    def __init__(self):
        self._data = {}

    def set(self, namespace: str, data: Any):
        """
        Store data under a given namespace.

        Args:
            namespace (str): A string key (typically a strategy or plugin name) used to namespace the data.
            data (Any): Arbitrary runtime data to associate with the namespace.
        """
        self._data[namespace] = data

    def get(self, namespace: str) -> Any:
        """
        Retrieve data associated with a given namespace.

        Args:
            namespace (str): The namespace under which the data was stored.

        Returns:
            Any: The data associated with the namespace, or None if not found.
        """
        return self._data.get(namespace)

    def get_or_create(self, namespace: str, factory: Callable[[], Any]) -> Any:
        """
        Retrieve existing data for a namespace, or create and store a new value using a factory function.

        Args:
            namespace (str): The namespace to retrieve or create data for.
            factory (Callable[[], Any]): A zero-argument function that returns a default value if the namespace is not yet set.

        Returns:
            Any: The existing or newly created data associated with the namespace.
        """
        if namespace not in self._data:
            self._data[namespace] = factory()
        return self._data[namespace]
