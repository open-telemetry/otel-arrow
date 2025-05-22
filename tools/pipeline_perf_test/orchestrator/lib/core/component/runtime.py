from typing import Callable, Dict, Any


class ComponentRuntime:
    """Holds runtime info for a component in a plugin-extensible way."""

    _data: Dict[str, Any]  # strategy_name -> plugin-defined state

    def __init__(self):
        self._data = {}

    def set(self, namespace: str, data: Any):
        self._data[namespace] = data

    def get(self, namespace: str) -> Any:
        return self._data.get(namespace)

    def get_or_create(self, namespace: str, factory: Callable[[], Any]) -> Any:
        if namespace not in self._data:
            self._data[namespace] = factory()
        return self._data[namespace]
