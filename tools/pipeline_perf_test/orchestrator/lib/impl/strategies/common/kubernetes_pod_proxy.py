"""Authenticated pod-proxy text endpoints with component-owned API clients."""

from dataclasses import dataclass
from typing import Callable, Literal, TYPE_CHECKING

from pydantic import BaseModel, ConfigDict, Field

if TYPE_CHECKING:
    from kubernetes.client import ApiClient


class KubernetesPodProxyConfig(BaseModel):
    """Address an HTTP(S) listener reachable through the pod's network interface."""

    model_config = ConfigDict(extra="forbid")

    port: int = Field(gt=0, le=65535)
    path: str = Field(default="/metrics", pattern=r"^/")
    scheme: Literal["http", "https"] = "http"


@dataclass(frozen=True)
class KubernetesPodProxyEndpoint:
    """Fetch text using the adoption-owned authenticated Kubernetes client."""

    get_client: Callable[[], "ApiClient"]
    namespace: str
    pod: str
    config: KubernetesPodProxyConfig

    def fetch(self, timeout: float) -> str:
        from kubernetes import client
        from kubernetes.client.exceptions import ApiException
        from urllib3.exceptions import (
            ConnectTimeoutError,
            HTTPError,
            MaxRetryError,
            NewConnectionError,
            ProtocolError,
            ReadTimeoutError,
        )

        name = f"{self.config.scheme}:{self.pod}:{self.config.port}"
        try:
            response = client.CoreV1Api(
                self.get_client()
            ).connect_get_namespaced_pod_proxy_with_path(
                name=name,
                namespace=self.namespace,
                path=self.config.path.lstrip("/"),
                _request_timeout=timeout,
                _preload_content=False,
            )
            try:
                return response.data.decode("utf-8")
            finally:
                response.close()
                response.release_conn()
        except (NewConnectionError, ProtocolError) as error:
            raise ConnectionError(f"Pod proxy connection failed: {error}") from error
        except (ConnectTimeoutError, ReadTimeoutError) as error:
            raise TimeoutError(f"Pod proxy request timed out: {error}") from error
        except MaxRetryError as error:
            if isinstance(error.reason, (NewConnectionError, ProtocolError)):
                raise ConnectionError(
                    f"Pod proxy connection failed: {error}"
                ) from error
            if isinstance(error.reason, (ConnectTimeoutError, ReadTimeoutError)):
                raise TimeoutError(f"Pod proxy request timed out: {error}") from error
            raise RuntimeError(f"Pod proxy request failed: {error}") from error
        except (ApiException, HTTPError) as error:
            raise RuntimeError(f"Pod proxy request failed: {error}") from error
