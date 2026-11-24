"""Adoption-owned localhost tunnels to Kubernetes pod HTTP endpoints."""

from collections import deque
from contextlib import ExitStack
from dataclasses import dataclass
from logging import Logger, LoggerAdapter
import re
import subprocess
import threading
from typing import Literal, Optional, TextIO

from pydantic import BaseModel, ConfigDict, Field
import requests

from .process import wait_or_terminate_process_tree


class KubernetesPortForwardConfig(BaseModel):
    """Address an HTTP(S) listener, including one bound only to pod localhost."""

    model_config = ConfigDict(extra="forbid")

    port: int = Field(gt=0, le=65535)
    path: str = Field(default="/metrics", pattern=r"^/")
    scheme: Literal["http", "https"] = "http"


class KubernetesPortForward:
    """Own one kubectl tunnel and its bounded diagnostic output."""

    def __init__(
        self,
        namespace: str,
        pod: str,
        port: int,
        kubeconfig: Optional[str],
        context: Optional[str],
        logger: Logger | LoggerAdapter,
        startup_timeout: float = 10.0,
    ):
        self.namespace = namespace
        self.pod = pod
        self.port = port
        self.kubeconfig = kubeconfig
        self.context = context
        self.logger = logger
        self.startup_timeout = startup_timeout
        self._process: Optional[subprocess.Popen[str]] = None
        self._reader: Optional[threading.Thread] = None
        self._local_port: Optional[int] = None
        self._reader_error: Optional[Exception] = None
        self._ready = threading.Event()
        self._output: deque[str] = deque(maxlen=32)
        self._output_lock = threading.Lock()

    def _diagnostics(self) -> str:
        with self._output_lock:
            return "\n".join(self._output)

    def _read_output(self, output: TextIO) -> None:
        try:
            for line in output:
                with self._output_lock:
                    self._output.append(line.rstrip())
                match = re.fullmatch(
                    rf"Forwarding from 127\.0\.0\.1:(\d+) -> {self.port}",
                    line.strip(),
                )
                if match:
                    self._local_port = int(match[1])
                    self._ready.set()
        except (OSError, UnicodeError) as error:
            self._reader_error = error
        finally:
            self._ready.set()

    def _check_running(self) -> None:
        if self._process is None:
            raise RuntimeError("Kubernetes port-forward has been released; adopt again")
        if self._reader_error is not None:
            raise RuntimeError(
                f"Could not read kubectl port-forward output: {self._reader_error}"
            ) from self._reader_error
        if self._process.poll() is not None or self._local_port is None:
            raise RuntimeError(
                f"kubectl port-forward did not remain running: {self._diagnostics()}"
            )
        if self._reader is not None and not self._reader.is_alive():
            raise RuntimeError(
                f"kubectl port-forward output closed: {self._diagnostics()}"
            )

    def start(self) -> None:
        if self._process is not None:
            raise RuntimeError("Kubernetes port-forward is already started")
        command = ["kubectl", "--namespace", self.namespace]
        if self.kubeconfig is not None:
            command.extend(["--kubeconfig", self.kubeconfig])
        if self.context is not None:
            command.extend(["--context", self.context])
        command.extend(
            ["port-forward", "--address=127.0.0.1", f"pod/{self.pod}", f"0:{self.port}"]
        )
        with ExitStack() as cleanup:
            cleanup.callback(self.close)
            self._process = subprocess.Popen(
                command,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                encoding="utf-8",
            )
            output = self._process.stdout
            if output is None:
                raise RuntimeError("kubectl port-forward output was not captured")
            self._reader = threading.Thread(
                target=self._read_output, args=(output,), daemon=True
            )
            self._reader.start()
            if not self._ready.wait(self.startup_timeout):
                raise TimeoutError(
                    f"kubectl port-forward startup timed out: {self._diagnostics()}"
                )
            self._check_running()
            cleanup.pop_all()

    def url(self, scheme: str, path: str) -> str:
        self._check_running()
        return f"{scheme}://127.0.0.1:{self._local_port}{path}"

    def close(self) -> None:
        process = self._process
        if process is None:
            return
        if process.poll() is None:
            wait_or_terminate_process_tree(process.pid, self.logger, normal_timeout=0)
        if process.poll() is None:
            raise RuntimeError("kubectl port-forward could not be stopped")
        reader = self._reader
        if reader is not None and reader.ident is not None:
            reader.join(timeout=3)
            if reader.is_alive():
                raise RuntimeError("kubectl port-forward output reader did not stop")
        if process.stdout is not None:
            process.stdout.close()
        self._process = None
        self._reader = None
        self._local_port = None


@dataclass(frozen=True)
class KubernetesPortForwardEndpoint:
    """Fetch text through a provider-owned tunnel without widening pod access."""

    forward: KubernetesPortForward
    config: KubernetesPortForwardConfig

    def fetch(self, timeout: float) -> str:
        url = self.forward.url(self.config.scheme, self.config.path)
        try:
            with requests.get(
                url, timeout=timeout, proxies={"http": "", "https": ""}
            ) as response:
                response.raise_for_status()
                return response.content.decode("utf-8")
        except requests.exceptions.Timeout as error:
            raise TimeoutError(f"Port-forward request timed out: {error}") from error
        except requests.exceptions.SSLError as error:
            raise RuntimeError(f"Port-forward request failed: {error}") from error
        except requests.exceptions.ConnectionError as error:
            raise ConnectionError(f"Port-forward connection failed: {error}") from error
        except requests.exceptions.RequestException as error:
            raise RuntimeError(f"Port-forward request failed: {error}") from error
