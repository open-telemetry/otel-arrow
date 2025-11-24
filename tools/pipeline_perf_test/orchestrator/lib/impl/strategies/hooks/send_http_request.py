"""
Hook strategy module for sending HTTP(S) requests.

This module defines the `send_http_request` hook, which enables HTTP requests
to be triggered during a test's lifecycle. It can be used to notify external
systems, trigger APIs, or integrate with web services at specific hook phases.

Classes:
    - SendHttpRequestConfig: Configuration schema specifying HTTP method, URL, headers, and payload.
    - SendHttpRequestHook: Hook strategy that sends the configured HTTP request when invoked.

Use case:
    Useful for sending notifications, logging to external systems, or triggering RESTful actions
    as part of a test or deployment pipeline.

Warnings:
    This hook performs external network operations. Ensure that endpoint URLs and request
    content are trusted and secure, particularly in public or production environments.
"""

from typing import Optional

import requests

from ....core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from ....core.context.base import BaseContext
from ....core.context import ComponentHookContext, FrameworkElementHookContext
from ....runner.registry import hook_registry, PluginMeta


HOOK_NAME = "send_http_request"


@hook_registry.register_config(HOOK_NAME)
class SendHttpRequestConfig(HookStrategyConfig):
    """
    Configuration class for the 'send_http_request' hook.

    Attributes:
        url (str): The full URL to send the HTTP request to.
        method (str): HTTP method (GET, POST, PUT, DELETE, etc.).
        headers (dict, optional): Optional headers to include in the request.
        payload (dict, optional): Optional JSON payload for methods like POST or PUT.
        timeout (int, optional): Request timeout in seconds (default: 10).
        raise_for_status (bool, optional): Raise an exception if status is not ok (default: true).
    """

    url: str
    method: Optional[str]
    headers: Optional[dict] = None
    payload: Optional[dict] = None
    timeout: Optional[int] = 10
    raise_for_status: Optional[bool] = True


@hook_registry.register_class(HOOK_NAME)
class SendHttpRequestHook(HookStrategy):
    """
    Hook strategy that sends an HTTP request to a configured endpoint.

    This class performs an HTTP operation using the provided configuration when invoked.
    """

    PLUGIN_META = PluginMeta(
        supported_contexts=[
            FrameworkElementHookContext.__name__,
            ComponentHookContext.__name__,
        ],
        installs_hooks=[],
        yaml_example="""
tests:
  - name: Notify external system
    steps:
      - name: Post to webhook before step
        action:
          wait:
            delay_seconds: 5
        hooks:
          run:
            pre:
              - send_http_request:
                  url: https://example.com/webhook
                  method: POST
                  headers:
                    Content-Type: application/json
                    Authorization: Bearer abc123
                  payload:
                    event: start
                    step: initialize
""",
    )

    def __init__(self, config: SendHttpRequestConfig):
        """
        Initialize the hook with its configuration.

        Args:
            config (SendHttpRequestConfig): HTTP request configuration.
        """
        self.config = config

    def execute(self, ctx: BaseContext):
        """
        Send the HTTP request using the requests module.

        Args:
            ctx (BaseContext): The execution context, providing utilities like logging.

        Raises:
            requests.RequestException: If the HTTP request fails.
        """

        logger = ctx.get_logger(__name__)
        logger.debug(
            f"Preparing HTTP {self.config.method} request to {self.config.url}"
        )

        try:
            response = requests.request(
                method=self.config.method.upper(),
                url=self.config.url,
                headers=self.config.headers,
                json=self.config.payload,
                timeout=self.config.timeout,
            )
            logger.debug(f"HTTP response {response.text}")
            if self.config.raise_for_status:
                response.raise_for_status()
            logger.debug(f"HTTP request complete: {response.status_code}")
        except requests.RequestException as e:
            logger.error(f"HTTP request failed: {e}")
            raise
