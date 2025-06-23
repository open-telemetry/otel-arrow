# runtime.py

## Overview

This module defines the `Runtime` class, which provides a flexible and plugin-extensible mechanism for storing runtime information associated with a component or element during its lifecycle.

`Runtime` uses string keys as namespaces (e.g., strategy or plugin names) to isolate different pieces of runtime data, allowing multiple plugins or strategies to attach their own state without interfering with each other.

---

## Key Features

- **Dynamic Namespacing:** Store arbitrary runtime data under string namespaces.
- **Lazy Initialization:** The `get_or_create` method allows creating default values only when needed.
- **Flexible Data Storage:** Supports any kind of user-defined data structures.

---

## Typical Usage

This pattern is useful within lifecycle components that need to track execution-time metadata across multiple strategies or plugins.

## Class Reference

Runtime
* set(namespace: str, data: Any)
Store arbitrary data under the given namespace.

* get(namespace: str) -> Any
Retrieve data stored under the given namespace, or None if not found.

* get_or_create(namespace: str, factory: Callable[[], Any]) -> Any
Retrieve data under the namespace, or create and store a new value using the factory function if missing.

## Usage Context
Runtime is designed for plugin-extensible systems where multiple strategies or plugins may need to attach execution-time state without collisions. It provides a clean and efficient way to manage such data during a component's lifecycle.
