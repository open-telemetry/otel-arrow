"""
Error handling utilities for test components and plugin execution.

This module defines a configuration model and a helper function for managing
error handling behavior in test or plugin execution contexts. It supports
retry policies, optional continuation after failure, and integration with
logging and tracing systems.

Classes:
    OnErrorConfig: Defines retry and continuation settings for error handling.

Functions:
    handle_with_policy: Wraps a callable with retry logic and error handling
    based on a provided OnErrorConfig.
"""

import time
from typing import Optional
from pydantic import BaseModel, Field
from opentelemetry.trace import StatusCode, Status
from ..context.base import BaseContext, ExecutionStatus


class OnErrorConfig(BaseModel):
    """Base configuration model for handling error state in test components / plugins."""

    retries: Optional[int] = 0
    retry_delay_seconds: Optional[int] = 10
    continue_: Optional[bool] = Field(default=False, alias="continue")


def handle_with_policy(ctx: BaseContext, func, on_error: OnErrorConfig):
    """
    Executes a function with retry and error-handling policy defined by `on_error`.

    This utility function wraps a callable in retry logic, retrying it upon exceptions
    based on the provided configuration. If retries are exhausted, the function either
    raises the exception or logs it and continues execution, depending on the `continue_`
    flag in the `on_error` config.

    Args:
        ctx (BaseContext): Execution context providing logger, status, and span tracing.
        func (Callable): The function to execute with retry and error handling.
        on_error (OnErrorConfig): Configuration for retry attempts, delay, and continue behavior.

    Returns:
        The return value of `func()` if successful or if `on_error.continue_` is True after failure.
        Otherwise, re-raises the final exception after exhausting retries.

    Raises:
        Exception: The last raised exception if retries are exhausted and `continue_` is False.
    """

    logger = ctx.get_logger(__name__)
    retries = on_error.retries
    should_continue = on_error.continue_

    for attempt in range(retries + 1):
        try:
            return func()
        except Exception as e:
            if attempt < retries:
                logger.warning(f"[Attempt {attempt+1}] Retrying after error: {e}")
                time.sleep(on_error.retry_delay_seconds)
                continue
            if should_continue:
                logger.warning(f"Continuing after failure: {e}")
                ctx.status = ExecutionStatus.ERROR
                ctx.span.set_status(
                    Status(
                        StatusCode.ERROR,
                        f"Failed after {retries+1} attempts, continuing per config.",
                    )
                )
                return
            logger.error(f"Fatal error after {retries+1} attempts: {e}")
            raise
