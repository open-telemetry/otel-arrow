# `errors`

Error handling utilities for test components and plugin execution.

## Overview

The `lib.core.errors` module provides a lightweight, configurable framework for error handling in test and plugin execution contexts. It supports flexible retry mechanisms, optional continuation after failure, and seamless integration with logging and tracing systems. This utility is designed to improve the reliability and observability of automated testing workflows.

## Structure

This directory contains the following file:

- `error_handler.py` - Implements configurable error handling behavior with support for retries, delayed retries, and optional continuation.

## Contents

### Classes

#### `OnErrorConfig`

A Pydantic-based model that defines how errors should be handled when executing test components or plugins.
**Fields:**

- `retries` (`Optional[int]`): Number of retry attempts after a failure. Default is `0`.
- `retry_delay_seconds` (`Optional[int]`): Delay in seconds between retry attempts. Default is `10`.
- `continue_` (`Optional[bool]`): If `True`, execution continues even after all retries fail. Default is `False`.

### Functions

#### `handle_with_policy(ctx, func, on_error)`

Wraps a callable function with retry logic and error handling, based on the provided `OnErrorConfig`.

**Parameters:**

- `ctx` (`BaseContext`): Context object providing logger, execution status, and tracing span.
- `func` (`Callable`): The function to execute.
- `on_error` (`OnErrorConfig`): Configuration object specifying retry and continuation behavior.

**Behavior:**

- Retries execution based on the configured retry count and delay.
- If retries fail, and `continue_` is set to `True`, logs the error and continues.
- If `continue_` is `False`, re-raises the last exception after retries are exhausted.

## Example Usage

```python
from lib.core.errors.error_handler import handle_with_policy, OnErrorConfig
from my_context import ctx

def risky_operation():
    # some operation that might fail
    ...

config = OnErrorConfig(retries=3, retry_delay_seconds=5, continue_=True)
result = handle_with_policy(ctx, risky_operation, config)
```
