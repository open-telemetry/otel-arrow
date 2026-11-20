"""Owned, loopback-only access to endpoints on adopted Kubernetes pods."""

from contextlib import ExitStack
from dataclasses import dataclass
from pathlib import Path
import re
import subprocess
from tempfile import TemporaryDirectory
import time

import requests
from pydantic import BaseModel, ConfigDict, Field, PositiveFloat

from .kubernetes import ComponentKubernetesRuntime, nonterminal_pods, pod_identity
from .process import wait_or_terminate_process_tree


class KubernetesPortForwardConfig(BaseModel):
    """Address an HTTP endpoint on every adopted pod without changing the pods."""

    model_config = ConfigDict(extra="forbid", allow_inf_nan=False)

    port: int = Field(gt=0, le=65535)
    path: str = Field(default="/metrics", pattern=r"^/")
    startup_timeout: PositiveFloat = 30


@dataclass
class ForwardedPodEndpoint:
    """A local URL and the adopted identity of its remote pod."""

    url: str
    attributes: dict[str, str]
    process: subprocess.Popen


class KubernetesPortForwards:
    """Bound startup and cleanup of only the kubectl processes created here."""

    def __init__(
        self,
        target: ComponentKubernetesRuntime,
        config: KubernetesPortForwardConfig,
        request_timeout: float,
        logger,
    ):
        self.target = target
        self.config = config
        self.request_timeout = request_timeout
        self.logger = logger
        self.endpoints: list[ForwardedPodEndpoint] = []
        self.cleanup = ExitStack()

    def open(self):
        """Open all tunnels atomically and confirm HTTP readiness and identity."""
        with ExitStack() as cleanup:
            directory = Path(cleanup.enter_context(TemporaryDirectory()))
            for pod in self.target.pods:
                log_path = directory / f"{pod.name}.log"
                log = cleanup.enter_context(log_path.open("w"))
                command = ["kubectl"]
                if self.target.kubeconfig:
                    command.extend(["--kubeconfig", self.target.kubeconfig])
                if self.target.context:
                    command.extend(["--context", self.target.context])
                command.extend(
                    [
                        "-n",
                        self.target.namespace,
                        "port-forward",
                        f"pod/{pod.name}",
                        f":{self.config.port}",
                        "--address",
                        "127.0.0.1",
                    ]
                )
                process = subprocess.Popen(
                    command, stdout=log, stderr=subprocess.STDOUT
                )
                cleanup.callback(self._stop, process, log_path)
                url = self._ready(process, log_path, pod.name)
                self.endpoints.append(
                    ForwardedPodEndpoint(
                        url=url,
                        attributes={
                            "k8s.namespace.name": self.target.namespace,
                            "k8s.pod.name": pod.name,
                            "k8s.pod.uid": pod.uid,
                            "k8s.node.name": pod.node_name,
                        },
                        process=process,
                    )
                )
            self.validate()
            self.cleanup = cleanup.pop_all()
        return self

    def _ready(self, process, log_path: Path, pod: str) -> str:
        deadline = time.monotonic() + self.config.startup_timeout
        last_error = None
        while time.monotonic() < deadline:
            output = log_path.read_text()
            if process.poll() is not None:
                raise RuntimeError(f"Port-forward for {pod!r} exited: {output}")
            match = re.search(
                rf"Forwarding from 127\.0\.0\.1:(\d+) -> {self.config.port}",
                output,
            )
            if match:
                url = f"http://127.0.0.1:{match[1]}{self.config.path}"
                try:
                    response = requests.get(
                        url,
                        timeout=min(
                            self.request_timeout,
                            max(0.001, deadline - time.monotonic()),
                        ),
                    )
                    response.raise_for_status()
                except requests.RequestException as error:
                    last_error = error
                else:
                    return url
            time.sleep(0.05)
        raise TimeoutError(
            f"Pod endpoint for {pod!r} did not become ready: {log_path.read_text()}"
        ) from last_error

    def validate(self):
        """Reject dead tunnels and changed or unready adopted pod identities."""
        from kubernetes import client, config

        if any(endpoint.process.poll() is not None for endpoint in self.endpoints):
            raise RuntimeError("Kubernetes port-forward exited during monitoring")
        timeout = (min(5.0, self.request_timeout), self.request_timeout)
        with config.new_client_from_config(
            config_file=self.target.kubeconfig,
            context=self.target.context,
            persist_config=False,
        ) as api_client:
            current = client.CoreV1Api(api_client).list_namespaced_pod(
                namespace=self.target.namespace,
                label_selector=self.target.label_selector,
                _request_timeout=timeout,
            )
        identities = [pod_identity(pod) for pod in nonterminal_pods(current.items)]
        expected = {pod.name: pod for pod in self.target.pods}
        if (
            len(identities) != len(expected)
            or {pod.name: pod for pod in identities} != expected
        ):
            raise ValueError("Kubernetes pod/container identities changed; adopt again")

    def _stop(self, process, log_path: Path):
        if process.poll() is None:
            wait_or_terminate_process_tree(process.pid, self.logger, normal_timeout=0)
        process.wait(timeout=5)
        self.logger.debug("Kubernetes port-forward output: %s", log_path.read_text())

    def close(self):
        """Release owned tunnels and logs, including after partial startup."""
        self.cleanup.close()
