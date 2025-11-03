import time
from typing import Optional, Union

import requests

from ....core.context.base import BaseContext
from ....core.context import ComponentHookContext, FrameworkElementHookContext
from ....runner.registry import hook_registry, PluginMeta

from .send_http_request import SendHttpRequestConfig, SendHttpRequestHook


HOOK_NAME = "ready_check_http"


@hook_registry.register_config(HOOK_NAME)
class ReadyCheckHttpConfig(SendHttpRequestConfig):
    """
    Configuration class for the 'ready_check_http' hook.

    Extends SendHttpRequestConfig to support readiness polling behavior.
    This configuration allows repeated HTTP requests to a target endpoint
    until specific success criteria are met, or a retry limit is reached.

    Attributes:
        max_retries (int, optional): The maximum number of attempts to check readiness.
            Default is 10.
        retry_interval (float, optional): Delay in seconds between each retry attempt.
            Default is 1.0.
        expected_status (int, optional): Optional expected HTTP status code.
            If set, the response must return this status to be considered ready.
        expected_json_field (str, optional): A top-level JSON field name to check in the response body.
        expected_json_value (str|int|bool|float, optional): The value expected for the JSON field.
            Both `expected_json_field` and `expected_json_value` must be set to enable this check.
        expected_text_substring (str, optional): A plain text substring that must be present
            in the response body for the check to pass.

    Notes:
        If multiple conditions are set (e.g., status code and body checks),
        all must pass for the readiness check to succeed.
    """

    max_retries: Optional[int] = 10
    retry_interval: Optional[float] = 1.0
    expected_status: Optional[int] = None
    # Optional body checks
    expected_json_field: Optional[str] = None
    expected_json_value: Optional[Union[str, int, bool, float]] = None
    expected_text_substring: Optional[str] = None


@hook_registry.register_class(HOOK_NAME)
class ReadyCheckHttpHook(SendHttpRequestHook):
    """
    Hook strategy that performs a readiness check against an HTTP(S) endpoint.

    This hook repeatedly sends an HTTP request to a target endpoint to verify its availability.
    The readiness check passes if all configured conditions are met, such as:
    - Expected HTTP status code
    - Presence of a specific field/value in the JSON body
    - Presence of a text substring in the response body

    It is useful for polling service health endpoints before executing test steps or deployments.

    Configuration is inherited from SendHttpRequestConfig and extended in ReadyCheckHttpConfig.
    """

    PLUGIN_META = PluginMeta(
        supported_contexts=[
            FrameworkElementHookContext.__name__,
            ComponentHookContext.__name__,
        ],
        installs_hooks=[],
        yaml_example="""
tests:
  - name: Wait for service readiness
    steps:
      - name: Ensure API is ready
        hooks:
          run:
            pre:
              - ready_check_http:
                  url: https://api.example.com/health
                  method: GET
                  payload: {}
                  headers: {}
                  expected_status: 200
                  expected_json_field: status
                  expected_json_value: ready
                  max_retries: 5
                  retry_interval: 2
""",
    )

    def execute(self, ctx: BaseContext):
        """
        Send the health check HTTP requests using the requests module.

        Args:
            ctx (BaseContext): The execution context, providing utilities like logging.

        Raises:
            RuntimeError: If requests fail in excess of max retries.
        """

        logger = ctx.get_logger(__name__)
        logger.debug("Starting ready check HTTP hook...")

        for attempt in range(1, self.config.max_retries + 1):
            try:
                response = requests.request(
                    method=self.config.method.upper(),
                    url=self.config.url,
                    headers=self.config.headers,
                    json=self.config.payload,
                    timeout=self.config.timeout,
                )

                logger.debug(
                    f"Attempt {attempt}: Received status {response.status_code}"
                )

                # --- Check expected status ---
                if self.config.expected_status is not None:
                    if response.status_code != self.config.expected_status:
                        logger.debug(
                            f"Expected status {self.config.expected_status}, "
                            f"but got {response.status_code}"
                        )
                        continue

                # --- Check JSON field & value ---
                if (
                    self.config.expected_json_field
                    and self.config.expected_json_value is not None
                ):
                    try:
                        json_data = response.json()
                        actual_value = json_data.get(self.config.expected_json_field)
                        if actual_value != self.config.expected_json_value:
                            logger.debug(
                                f"JSON field '{self.config.expected_json_field}' "
                                f"was '{actual_value}', expected '{self.config.expected_json_value}'"
                            )
                            continue
                    except Exception as e:
                        logger.debug(f"Failed to parse JSON or match field: {e}")
                        continue

                # --- Check text substring ---
                if self.config.expected_text_substring:
                    if self.config.expected_text_substring not in response.text:
                        logger.debug(
                            f"Text '{self.config.expected_text_substring}' not found in response body"
                        )
                        continue

                # If no expected conditions were set, pass on success status or assume success
                logger.info(f"Ready check passed on attempt {attempt}")
                return

            except requests.RequestException as e:
                logger.debug(f"Attempt {attempt} failed with exception: {e}")

            if attempt < self.config.max_retries:
                time.sleep(self.config.retry_interval)

        raise RuntimeError("Ready check failed: conditions not satisfied in time.")
