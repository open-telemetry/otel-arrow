"""Stock CLI coverage for caller-owned targets and harness-owned helpers."""

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import threading
from unittest.mock import MagicMock
from urllib.parse import parse_qs, urlsplit

import psutil
import pytest
import requests
import yaml

from lib.impl.strategies.monitoring.process_component import (
    ProcessComponentMonitoringConfig,
    ProcessComponentMonitoringStrategy,
)
from lib.impl.component.managed_component import ManagedComponentConfiguration
from lib.runner.factory import build_managed_component

ORCHESTRATOR = Path(__file__).resolve().parents[2]
EXAMPLE = ORCHESTRATOR / "examples" / "monitoring-only.yaml"


@pytest.fixture
def existing_service():
    scraped = threading.Event()
    loaded = threading.Event()
    api_requests = []

    class Handler(BaseHTTPRequestHandler):
        def send_json(self, body):
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(body).encode())

        def do_GET(self):
            path = urlsplit(self.path).path
            api_requests.append(("GET", path))
            if path == "/version":
                self.send_json({"ApiVersion": "1.41"})
            elif path == "/v1.41/containers/external/json":
                self.send_json(
                    {
                        "Id": "external-id",
                        "Name": "/external",
                        "State": {"Status": "running", "Running": True},
                    }
                )
            elif path == "/api/v1/namespaces/telemetry/pods":
                selector = parse_qs(urlsplit(self.path).query).get("labelSelector")
                if selector != ["app=external"]:
                    self.send_json({"apiVersion": "v1", "kind": "PodList", "items": []})
                    return
                self.send_json(
                    {
                        "apiVersion": "v1",
                        "kind": "PodList",
                        "items": [
                            {
                                "metadata": {"name": "external", "uid": "pod-uid"},
                                "spec": {
                                    "nodeName": "node-0",
                                    "containers": [{"name": "app"}],
                                },
                                "status": {
                                    "phase": "Running",
                                    "conditions": [{"type": "Ready", "status": "True"}],
                                    "containerStatuses": [
                                        {
                                            "name": "app",
                                            "image": "example",
                                            "imageID": "sha256:x",
                                            "ready": True,
                                            "restartCount": 0,
                                            "containerID": "containerd://app",
                                            "state": {"running": {}},
                                        }
                                    ],
                                },
                            }
                        ],
                    }
                )
            elif path == "/metrics":
                body = b"# TYPE example_value gauge\nexample_value 42\n"
                scraped.set()
                self.send_response(200)
                self.end_headers()
                self.wfile.write(body)
            elif self.path in ("/observed", "/loaded"):
                event = loaded if self.path == "/loaded" else scraped
                self.send_response(200 if event.wait(10) else 503)
                self.end_headers()
            else:
                self.send_error(404)

        def do_POST(self):
            api_requests.append(("POST", self.path))
            if self.path != "/load":
                self.send_error(404)
                return
            self.rfile.read(int(self.headers["Content-Length"]))
            loaded.set()
            self.send_response(204)
            self.end_headers()

        def log_message(self, _format, *_args):
            pass

        def do_DELETE(self):
            api_requests.append(("DELETE", self.path))
            self.send_error(405)

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever)
    thread.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}", scraped, loaded, api_requests
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=10)
        assert not thread.is_alive()


def component_step(name, phase, target):
    return {
        "name": name,
        "action": {"component_action": {"phase": phase, "target": target}},
    }


def wait_for_request(name, url):
    return {
        "name": name,
        "action": {"no_op": {}},
        "hooks": {
            "run": {
                "post": [
                    {
                        "ready_check_http": {
                            "url": url,
                            "method": "GET",
                            "timeout": 15,
                            "max_retries": 1,
                            "expected_status": 200,
                        }
                    }
                ]
            }
        },
    }


def write_kubeconfig(path, endpoint):
    path.write_text(
        yaml.safe_dump(
            {
                "apiVersion": "v1",
                "kind": "Config",
                "clusters": [{"name": "test", "cluster": {"server": endpoint}}],
                "contexts": [
                    {
                        "name": "test",
                        "context": {"cluster": "test", "user": "test"},
                    }
                ],
                "current-context": "test",
                "users": [{"name": "test", "user": {}}],
            }
        )
    )


# Scenario: The stock CLI observes an endpoint or adopts Docker/Kubernetes
# resources, optionally with a process generator, on success and failure.
# Guarantees: Only Docker adoption needs Docker API reads; all modes scrape
# metrics without an opt-out flag, preserve external resources, and clean up
# harness-owned generators.
@pytest.mark.parametrize("with_generator", [False, True])
@pytest.mark.parametrize("fail", [False, True])
@pytest.mark.parametrize("backend", [None, "docker", "kubernetes"])
def test_cli_preserves_existing_service(
    tmp_path, existing_service, with_generator, fail, backend
):
    endpoint, scraped, loaded, api_requests = existing_service
    config = yaml.safe_load(EXAMPLE.read_text())
    config["components"]["existing-service"]["monitoring"]["prometheus"].update(
        endpoint=f"{endpoint}/metrics", interval=0.01
    )
    steps = config["tests"][0]["steps"]
    steps[1] = wait_for_request("Wait for a scrape", f"{endpoint}/observed")
    environment = {
        **os.environ,
        "DOCKER_HOST": f"unix://{tmp_path}/no-docker.sock",
    }
    if backend is not None:
        if backend == "docker":
            existing = {"docker": {"container": "external"}}
            environment["DOCKER_HOST"] = endpoint.replace("http://", "tcp://")
        else:
            kubeconfig = tmp_path / "kubeconfig.yaml"
            write_kubeconfig(kubeconfig, endpoint)
            existing = {
                "kubernetes": {
                    "namespace": "telemetry",
                    "label_selector": "app=external",
                    "kubeconfig": str(kubeconfig),
                }
            }
        config["components"]["existing-service"]["existing"] = existing
        steps.insert(
            0, component_step("Adopt existing service", "adopt", "existing-service")
        )
    pid_file = tmp_path / "generator.pid"
    if with_generator:
        script = (
            "import os, pathlib, time, urllib.request; "
            f"pathlib.Path({str(pid_file)!r}).write_text(str(os.getpid())); "
            f"urllib.request.urlopen({endpoint + '/load'!r}, "
            "data=b'test', timeout=10).close(); "
            "time.sleep(3600)"
        )
        config["components"]["generator"] = {
            "deployment": {
                "process": {"command": shlex.join([sys.executable, "-c", script])}
            }
        }
        steps.insert(0, component_step("Deploy generator", "deploy", "generator"))
        steps.insert(2, wait_for_request("Wait for load", f"{endpoint}/loaded"))
        steps.append(component_step("Remove generator", "destroy", "generator"))
    if fail:
        steps.insert(
            next(
                i for i, step in enumerate(steps) if step["name"] == "Stop monitoring"
            ),
            {
                "name": "Fail the observation",
                "action": {"no_op": {}},
                "hooks": {
                    "run": {
                        "pre": [{"raise_exception": {"message": "failed observation"}}]
                    }
                },
            },
        )
    config_path = tmp_path / "suite.yaml"
    config_path.write_text(yaml.safe_dump(config))

    try:
        result = subprocess.run(
            [
                sys.executable,
                str(ORCHESTRATOR / "run_orchestrator.py"),
                "--config",
                str(config_path),
            ],
            cwd=tmp_path,
            env=environment,
            capture_output=True,
            text=True,
            timeout=60,
        )
        assert (result.returncode != 0) == fail, result.stdout + result.stderr
        if fail:
            assert "failed observation" in result.stderr
        assert scraped.is_set(), result.stdout + result.stderr
        assert [(method, path) for method, path in api_requests if method != "GET"] == (
            [("POST", "/load")] if with_generator else []
        )
        if backend == "docker":
            assert ("GET", "/v1.41/containers/external/json") in api_requests
        elif backend == "kubernetes":
            assert ("GET", "/api/v1/namespaces/telemetry/pods") in api_requests
        if with_generator:
            assert loaded.is_set()
            assert pid_file.exists()
            assert not psutil.pid_exists(int(pid_file.read_text()))
        assert requests.get(f"{endpoint}/metrics", timeout=5).status_code == 200
    finally:
        if pid_file.exists():
            pid = int(pid_file.read_text())
            if psutil.pid_exists(pid):
                process = psutil.Process(pid)
                assert process.cmdline() == [sys.executable, "-c", script]
                process.terminate()
                process.wait(timeout=10)


@pytest.mark.parametrize("fail_fast", [False, True])
def test_cli_failed_adoption_does_not_scrape_reachable_endpoint(
    tmp_path, existing_service, fail_fast
):
    endpoint, scraped, _, api_requests = existing_service
    kubeconfig = tmp_path / "kubeconfig.yaml"
    write_kubeconfig(kubeconfig, endpoint)
    config = yaml.safe_load(
        (ORCHESTRATOR / "examples" / "existing-kubernetes.yaml").read_text()
    )
    component = config["components"]["existing-service"]
    component["existing"]["kubernetes"].update(
        label_selector="app=does-not-exist", kubeconfig=str(kubeconfig)
    )
    component["monitoring"]["prometheus"].update(
        endpoint=f"{endpoint}/metrics", interval=0.01
    )
    scenario = config["tests"][0]
    scenario["fail_fast"] = fail_fast
    if not fail_fast:
        scenario["steps"][2]["action"]["wait"]["delay_seconds"] = 0
    config_path = tmp_path / "suite.yaml"
    config_path.write_text(yaml.safe_dump(config))

    result = subprocess.run(
        [
            sys.executable,
            str(ORCHESTRATOR / "run_orchestrator.py"),
            "--config",
            str(config_path),
        ],
        cwd=tmp_path,
        env={**os.environ, "DOCKER_HOST": f"unix://{tmp_path}/no-docker.sock"},
        capture_output=True,
        text=True,
        timeout=10,
    )
    output = result.stdout + result.stderr
    assert result.returncode != 0, output
    assert "No existing pods match 'app=does-not-exist'" in output
    if fail_fast:
        assert "Skipping remaining steps after failure" in output
        for name in ("Start monitoring", "Observe for 60 seconds", "Stop monitoring"):
            assert f"ctx.name={name}" not in output
    else:
        assert "requires a successful 'adopt' phase before monitoring" in output
        assert "Continuing remaining steps for diagnostics" in output
        assert "ctx.name=Stop monitoring" in output
    assert not scraped.is_set(), output
    assert api_requests == [("GET", "/api/v1/namespaces/telemetry/pods")]
    assert requests.get(f"{endpoint}/metrics", timeout=5).status_code == 200


# Scenario: Process resource monitoring is attached without process deployment.
# Guarantees: Monitoring-only support does not invent a PID or silently monitor
# the harness; the existing deployment-runtime requirement is reported.
def test_process_monitoring_still_requires_a_deployment_pid():
    component = build_managed_component(
        "existing-service", ManagedComponentConfiguration()
    )
    strategy = ProcessComponentMonitoringStrategy(ProcessComponentMonitoringConfig())
    with pytest.raises(RuntimeError, match="can't find process id"):
        strategy.start(component, MagicMock())
