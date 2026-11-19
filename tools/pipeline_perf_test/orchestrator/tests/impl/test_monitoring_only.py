"""Stock CLI coverage for caller-owned targets and harness-owned helpers."""

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from datetime import datetime, timezone
import csv
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import threading
import time
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
def existing_service(request):
    scraped = threading.Event()
    loaded = threading.Event()
    api_requests = []
    resource_timestamp = datetime.now(timezone.utc).isoformat()
    resource_polls = 0
    fresh_resource_timestamp = None

    class Handler(BaseHTTPRequestHandler):
        def send_json(self, body):
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(body).encode())

        def do_GET(self):
            nonlocal resource_polls, fresh_resource_timestamp
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
            elif path == "/apis/metrics.k8s.io/v1beta1/namespaces/telemetry/pods":
                mode = getattr(request, "param", None)
                if mode == "forbidden-resources":
                    self.send_error(403)
                    return
                resource_polls += 1
                fresh = (
                    mode in ("resources", "overlapping-resources")
                    and resource_polls > 1
                )
                if fresh and fresh_resource_timestamp is None:
                    fresh_resource_timestamp = datetime.now(timezone.utc).isoformat()
                timestamp = fresh_resource_timestamp if fresh else resource_timestamp
                cpu = "250m" if fresh else "500m"
                memory = "8Mi" if fresh else "16Mi"
                self.send_json(
                    {
                        "kind": "PodMetricsList",
                        "apiVersion": "metrics.k8s.io/v1beta1",
                        "items": (
                            []
                            if mode == "missing-resources"
                            else [
                                {
                                    "metadata": {
                                        "name": "external",
                                        "namespace": "telemetry",
                                    },
                                    "timestamp": timestamp,
                                    "window": (
                                        "1ms"
                                        if fresh and mode == "resources"
                                        else "1m6.318s"
                                    ),
                                    "containers": [
                                        {
                                            "name": "app",
                                            "usage": {"cpu": cpu, "memory": memory},
                                        }
                                    ],
                                }
                            ]
                        ),
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


# Scenario: The stock CLI consumes the shared observation and resource templates.
# Guarantees: Window eligibility and failures preserve read-only target ownership.
@pytest.mark.parametrize(
    "existing_service,expected_error,expected_cpu_samples,expected_memory_samples",
    [
        ("resources", None, 1, 1),
        ("overlapping-resources", None, 0, 1),
        ("cached-resources", None, 0, 0),
        ("missing-resources", "Missing Kubernetes metrics pods", None, None),
        ("forbidden-resources", "Forbidden", None, None),
    ],
    indirect=["existing_service"],
)
def test_cli_collects_kubernetes_resources_without_engine_endpoint(
    tmp_path,
    existing_service,
    expected_error,
    expected_cpu_samples,
    expected_memory_samples,
):
    endpoint, scraped, _, api_requests = existing_service
    kubeconfig = tmp_path / "kubeconfig.yaml"
    write_kubeconfig(kubeconfig, endpoint)
    config = yaml.safe_load(
        (ORCHESTRATOR / "examples" / "existing-kubernetes.yaml").read_text()
    )
    reference = config["tests"][0]["from_template"]
    reference["path"] = str(ORCHESTRATOR / reference["path"])
    variables = reference["variables"]
    report = variables["report_config"]
    report["from_template"]["path"] = str(
        ORCHESTRATOR / report["from_template"]["path"]
    )
    component = config["components"]["existing-service"]
    del component["monitoring"]["prometheus"]
    component["existing"]["kubernetes"].update(
        label_selector="app=external", kubeconfig=str(kubeconfig)
    )
    component["monitoring"]["kubernetes_component"]["interval"] = 0.05
    # Keep a long wait in failure cases to verify synchronous startup fails fast.
    if expected_error is None:
        variables["duration_seconds"] = 0.2
    report["write_tables"] = {
        table: {"path": str(tmp_path / f"{table}.csv"), "format": "csv"}
        for table in ("resource_samples", "resource_summary")
    }
    config_path = tmp_path / "suite.yaml"
    config_path.write_text(yaml.safe_dump(config))
    result = subprocess.run(
        [
            sys.executable,
            str(ORCHESTRATOR / "run_orchestrator.py"),
            "--config",
            str(config_path),
            "--print-metrics",
        ],
        cwd=tmp_path,
        env={**os.environ, "DOCKER_HOST": f"unix://{tmp_path}/no-docker.sock"},
        capture_output=True,
        text=True,
        timeout=15,
    )
    output = result.stdout + result.stderr
    assert (result.returncode == 0) == (expected_error is None), output
    if expected_error is None:
        assert "container.cpu.usage" in result.stdout, output
        assert "container.memory.working_set" in result.stdout, output
        assert "sample.timestamp" in result.stdout, output
        assert "Kubernetes container resources" in output
        with (tmp_path / "resource_summary.csv").open() as summary_file:
            summary = list(csv.DictReader(summary_file))
        assert len(summary) == 1
        assert int(summary[0]["cpu_samples"]) == expected_cpu_samples
        assert int(summary[0]["memory_samples"]) == expected_memory_samples
        if expected_cpu_samples:
            assert float(summary[0]["cpu_sample_mean_pct_1_core"]) == 25
        else:
            assert summary[0]["cpu_sample_mean_pct_1_core"] == ""
            assert summary[0]["coverage"] == "insufficient CPU windows"
        if expected_memory_samples:
            assert float(summary[0]["working_set_sample_mean_mib"]) == 8
        else:
            assert summary[0]["working_set_sample_mean_mib"] == ""
        with (tmp_path / "resource_samples.csv").open() as samples_file:
            samples = list(csv.DictReader(samples_file))
        assert len(samples) == 1 + expected_memory_samples
        assert float(samples[0]["cpu_pct_1_core"]) == 50
        assert float(samples[0]["working_set_mib"]) == 16
        assert samples[0]["cpu_in_window"] == "false"
        assert samples[0]["memory_in_window"] == "false"
    else:
        assert expected_error in output
        assert "ctx.name=Observe for 60 seconds" not in output
        assert not (tmp_path / "resource_summary.csv").exists()
    assert not scraped.is_set()
    assert all(method == "GET" for method, _ in api_requests)
    assert ("GET", "/api/v1/namespaces/telemetry/pods") in api_requests
    assert (
        "GET",
        "/apis/metrics.k8s.io/v1beta1/namespaces/telemetry/pods",
    ) in api_requests
    surviving = requests.get(
        f"{endpoint}/api/v1/namespaces/telemetry/pods",
        params={"labelSelector": "app=external"},
        timeout=5,
    )
    assert surviving.json()["items"][0]["metadata"]["uid"] == "pod-uid"


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


# Scenario: Adoption fails in a scenario loaded from the observation template.
# Guarantees: Neither fail-fast mode scrapes or takes ownership of the target.
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
    reference = config["tests"][0]["from_template"]
    reference["path"] = str(ORCHESTRATOR / reference["path"])
    report = reference["variables"]["report_config"]["from_template"]
    report["path"] = str(ORCHESTRATOR / report["path"])
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
        reference["variables"]["duration_seconds"] = 0
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


@pytest.mark.parametrize(
    "outcome", ["success", "fail-fast", "scrape-error", "terminate"]
)
def test_cli_owns_pod_access_without_owning_the_pod(
    tmp_path, existing_service, outcome
):
    endpoint, _, _, api_requests = existing_service
    kubeconfig = tmp_path / "kubeconfig.yaml"
    write_kubeconfig(kubeconfig, endpoint)
    tools = tmp_path / "bin"
    tools.mkdir()
    pid_file = tmp_path / "forward.pid"
    scraped_file = tmp_path / "scraped"
    kubectl = tools / "kubectl"
    kubectl.write_text(f"#!{sys.executable}\n" + r"""
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import os
from pathlib import Path
import sys
import urllib.request

Path(os.environ["FORWARD_PID"]).write_text(str(os.getpid()))
calls = 0
class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        global calls
        calls += 1
        if calls > 1:
            Path(os.environ["FORWARD_SCRAPED"]).touch()
        if calls > 1 and os.environ["FORWARD_OUTCOME"] == "scrape-error":
            self.send_error(503)
            return
        with urllib.request.urlopen(
            os.environ["FORWARD_SOURCE"] + "/metrics", timeout=2
        ) as response:
            body = response.read()
        self.send_response(200)
        self.end_headers()
        self.wfile.write(body)
    def log_message(self, _format, *_args):
        pass
port = next(arg[1:] for arg in sys.argv if arg.startswith(":"))
with ThreadingHTTPServer(("127.0.0.1", 0), Handler) as server:
    print(f"Forwarding from 127.0.0.1:{server.server_port} -> {port}", flush=True)
    server.serve_forever()
""")
    kubectl.chmod(0o755)
    steps = [
        component_step("Adopt", "adopt", "service"),
        component_step("Monitor", "start_monitoring", "service"),
        {
            "name": "Wait",
            "action": {
                "wait": {"delay_seconds": 60 if outcome == "terminate" else 0.2}
            },
        },
        component_step("Stop", "stop_monitoring", "service"),
    ]
    if outcome == "fail-fast":
        steps.insert(
            2,
            {
                "name": "Fail",
                "action": {"no_op": {}},
                "hooks": {
                    "run": {"pre": [{"raise_exception": {"message": "case failed"}}]}
                },
            },
        )
    config = {
        "name": "Generic pod Prometheus",
        "components": {
            "service": {
                "existing": {
                    "kubernetes": {
                        "namespace": "telemetry",
                        "label_selector": "app=external",
                        "kubeconfig": str(kubeconfig),
                    }
                },
                "monitoring": {
                    "prometheus": {
                        "kubernetes_port_forward": {"port": 12345},
                        "interval": 0.01,
                    }
                },
            }
        },
        "tests": [{"name": "Observe", "fail_fast": True, "steps": steps}],
    }
    config_path = tmp_path / "suite.yaml"
    config_path.write_text(yaml.safe_dump(config))
    environment = {
        **os.environ,
        "PATH": f"{tools}:{os.environ['PATH']}",
        "FORWARD_PID": str(pid_file),
        "FORWARD_SCRAPED": str(scraped_file),
        "FORWARD_SOURCE": endpoint,
        "FORWARD_OUTCOME": outcome,
        "DOCKER_HOST": f"unix://{tmp_path}/absent.sock",
    }
    process = subprocess.Popen(
        [
            sys.executable,
            str(ORCHESTRATOR / "run_orchestrator.py"),
            "--config",
            str(config_path),
            "--print-metrics",
        ],
        cwd=tmp_path,
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        if outcome == "terminate":
            deadline = time.monotonic() + 10
            while not scraped_file.exists() and process.poll() is None:
                assert time.monotonic() < deadline, "monitor did not become ready"
                time.sleep(0.05)
            assert scraped_file.exists()
            process.terminate()
        stdout, stderr = process.communicate(timeout=15)
        assert (process.returncode == 0) == (outcome == "success"), stdout + stderr
        if outcome == "success":
            assert "example_value" in stdout
            assert "k8s.pod.uid" in stdout
            assert "pod-uid" in stdout
        if outcome == "scrape-error":
            assert "Kubernetes Prometheus monitoring failed" in stderr
        assert pid_file.exists()
        assert not psutil.pid_exists(int(pid_file.read_text()))
        assert all(method == "GET" for method, _ in api_requests)
        assert requests.get(f"{endpoint}/metrics", timeout=2).status_code == 200
    finally:
        if process.poll() is None:
            process.terminate()
            process.communicate(timeout=15)
        if pid_file.exists() and psutil.pid_exists(int(pid_file.read_text())):
            helper = psutil.Process(int(pid_file.read_text()))
            helper.terminate()
            helper.wait(timeout=5)
