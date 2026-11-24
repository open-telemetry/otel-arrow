"""Actual process coverage for bounded forwarding startup and teardown."""

import subprocess
import sys
from types import SimpleNamespace
from unittest.mock import MagicMock

import pytest

from lib.runner.factory import build_managed_component  # noqa: F401
from lib.impl.strategies.common import kubernetes_port_forward as forwarding


@pytest.fixture
def helper(tmp_path, monkeypatch):
    command = tmp_path / "kubectl"
    processes = []
    original = subprocess.Popen

    def spawn(*args, **kwargs):
        process = original(*args, **kwargs)
        processes.append(process)
        return process

    monkeypatch.setattr(
        forwarding,
        "subprocess",
        SimpleNamespace(Popen=spawn, PIPE=subprocess.PIPE, STDOUT=subprocess.STDOUT),
    )
    monkeypatch.setenv("PATH", str(tmp_path))
    forward = forwarding.KubernetesPortForward(
        namespace="telemetry",
        pod="external",
        port=12345,
        kubeconfig="/caller/config",
        context="caller-context",
        logger=MagicMock(),
        startup_timeout=10,
    )
    try:
        yield SimpleNamespace(command=command, forward=forward, processes=processes)
    finally:
        forward.close()
        assert all(process.poll() is not None for process in processes)


def write_command(path, source):
    path.write_text(f"#!{sys.executable}\n" + source)
    path.chmod(0o755)


# Scenario: kubectl exits with a denial or never publishes its listening address.
# Guarantees: Startup fails within its bound and closes/reaps the partial helper.
@pytest.mark.parametrize("mode", ["denied", "timeout"])
def test_failed_startup_closes_partial_forward(helper, mode):
    if mode == "denied":
        write_command(
            helper.command,
            "print('Error from server (Forbidden): portforward denied', flush=True)\n",
        )
        expected, message = RuntimeError, "portforward denied"
    else:
        write_command(helper.command, "import threading\nthreading.Event().wait()\n")
        helper.forward.startup_timeout = 0.05
        expected, message = TimeoutError, "startup timed out"
    with pytest.raises(expected, match=message):
        helper.forward.start()
    assert all(process.poll() is not None for process in helper.processes)
    assert helper.forward._process is None


# Scenario: A local scrape bypasses outbound proxies, then its helper exits.
# Guarantees: Scrapes reach localhost; exit surfaces without restart or leaked readers.
def test_exited_forward_is_not_restarted(helper, monkeypatch):
    write_command(
        helper.command,
        "from http.server import BaseHTTPRequestHandler, HTTPServer\n"
        "class Handler(BaseHTTPRequestHandler):\n"
        "    def do_GET(self):\n"
        "        self.send_response(200)\n"
        "        self.end_headers()\n"
        "        self.wfile.write(b'example_value 7\\n')\n"
        "server = HTTPServer(('127.0.0.1', 0), Handler)\n"
        "print(f'Forwarding from 127.0.0.1:{server.server_port} -> 12345', flush=True)\n"
        "server.serve_forever()\n",
    )
    for name in ("HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY"):
        monkeypatch.setenv(name, "http://127.0.0.1:1")
        monkeypatch.setenv(name.lower(), "http://127.0.0.1:1")
    monkeypatch.setenv("NO_PROXY", "")
    monkeypatch.setenv("no_proxy", "")
    helper.forward.start()
    endpoint = forwarding.KubernetesPortForwardEndpoint(
        helper.forward, forwarding.KubernetesPortForwardConfig(port=12345)
    )
    assert endpoint.fetch(3) == "example_value 7\n"
    process = helper.processes[0]
    reader = helper.forward._reader
    process.terminate()
    process.wait(timeout=5)
    with pytest.raises(RuntimeError, match="did not remain running"):
        endpoint.fetch(3)
    assert len(helper.processes) == 1
    helper.forward.close()
    helper.forward.close()
    assert reader is not None and not reader.is_alive()


# Scenario: A missing kubectl executable prevents endpoint adoption.
# Guarantees: The failure is explicit and no process or output thread is retained.
def test_missing_kubectl_leaves_no_partial_forward(helper):
    with pytest.raises(FileNotFoundError):
        helper.forward.start()
    assert not helper.processes
    assert helper.forward._process is None
