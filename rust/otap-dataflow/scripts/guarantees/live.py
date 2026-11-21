"""Exercise an operator's add/change/remove workflow against one real engine."""

from contextlib import ExitStack
from copy import deepcopy
import json
from pathlib import Path
import socket
import tempfile
import time
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

from engine import from_args


def request(url, method="GET", body=None):
    data = None if body is None else json.dumps(body).encode()
    req = Request(url, method=method, data=data, headers={"Content-Type": "application/json"})
    with urlopen(req, timeout=15) as response:
        return json.load(response)


def until(description, probe, process):
    deadline = time.monotonic() + 12
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"engine exited with {process.returncode} while waiting for {description}")
        if probe():
            return
        time.sleep(0.03)
    raise TimeoutError(f"waiting for {description}")


def syslog(port):
    return {"type": "receiver:syslog_cef", "config": {
        "batch": {"max_size": 1}, "protocol": {"tcp": {"listening_addr": f"127.0.0.1:{port}"}}}}


def send(port, marker):
    with socket.create_connection(("127.0.0.1", port), timeout=3) as client:
        client.sendall(f"<14>1 2025-01-01T00:00:00Z host app - - - {marker}\n".encode())


def listener_closed(port):
    try:
        with socket.create_connection(("127.0.0.1", port), timeout=2):
            return False
    except ConnectionRefusedError:
        return True


def pipeline(port, path):
    return {
        "nodes": {
            "input": syslog(port),
            "out": {"type": "exporter:file", "config": {
                "path": str(path) + "-{signal}-{core_id}-{generation}.jsonl"}},
        },
        "connections": [{"from": "input", "to": "out"}],
    }


def delivered(directory, name, marker, phase=None):
    for path in directory.glob(f"{name}-logs-*.jsonl"):
        for line in path.read_bytes().splitlines(keepends=True):
            if not line.endswith(b"\n"):
                continue
            for resource in json.loads(line).get("resourceLogs", []):
                for scope in resource.get("scopeLogs", []):
                    for record in scope.get("logRecords", []):
                        attrs = {a["key"]: a["value"].get("stringValue")
                                 for a in record.get("attributes", [])}
                        if attrs.get("syslog.message") == marker:
                            return attrs.get("guarantee.phase") == phase
    return False


def exercise(engine, directory, ports):
    admin, main, observer, extra, changed = ports
    host = "127.0.0.1"
    root = f"http://{host}:{admin}/api/v1"
    pipelines = root + "/groups/default/pipelines"
    config = {
        "version": "otel_dataflow/v1",
        "engine": {"http_admin": {"bind_address": f"{host}:{admin}"}},
        "policies": {"resources": {"core_allocation": {"type": "core_count", "count": 1}}},
        "groups": {"default": {"pipelines": {
            "main": pipeline(main, directory / "main"),
            "observer": pipeline(observer, directory / "observer"),
        }}},
    }
    startup = directory / "startup.json"
    startup.write_text(json.dumps(config))
    with (directory / "engine.log").open("wb") as log, engine.running(startup, log) as process:
        def ready():
            try:
                return all(request(f"{pipelines}/{p}/status")["runningCores"] == 1
                           for p in ("main", "observer"))
            except (URLError, ConnectionError):
                return False

        until("both running pipelines", ready, process)
        observer_generation = request(f"{pipelines}/observer")["activeGeneration"]

        def probe(name, marker):
            port = main if name == "main" else observer
            send(port, marker)
            until(marker, lambda: delivered(directory, name, marker), process)
            if name == "observer":
                assert request(f"{pipelines}/observer")["activeGeneration"] == observer_generation

        def rollout(candidate, step):
            url = f"{pipelines}/main"
            result = request(url + "?wait=false", "PUT", {"pipeline": candidate})
            probe("observer", "observer-during-" + step)
            def completed():
                status = request(url + "/rollouts/" + result["rolloutId"])
                if status["state"] == "failed":
                    raise AssertionError(f"{step}: {status}")
                return status["state"] == "succeeded"
            until(step + " rollout", completed, process)
            probe("observer", "observer-after-" + step)

        probe("main", "original-main")
        probe("observer", "original-observer")
        original = request(f"{pipelines}/main")["pipeline"]
        added = deepcopy(original)
        added["connections"].append({"from": "syslog", "to": "out"})

        for step, port in (("add", extra), ("change", changed)):
            added["nodes"]["syslog"] = syslog(port)
            rollout(added, step)
            marker = "syslog-guarantee-" + step
            send(port, marker)
            until(marker, lambda: delivered(directory, "main", marker), process)

        assert listener_closed(extra), "old listening address still accepts connections"
        rollout(original, "remove")
        assert listener_closed(changed), "removed receiver still accepts connections"
        # Change a real processor and destination, checking their effect in the exported record.
        for phase in ("introduced", "modified"):
            candidate = deepcopy(original)
            output = candidate["nodes"].pop("out")
            output["config"]["path"] = str(directory / (phase + "-{signal}-{core_id}-{generation}.jsonl"))
            candidate["nodes"]["destination"] = output
            candidate["nodes"]["annotate"] = {"type": "processor:attribute", "config": {
                "actions": [{"action": "upsert", "key": "guarantee.phase", "value": phase}]}}
            candidate["connections"] = [{"from": "input", "to": "annotate"},
                                        {"from": "annotate", "to": "destination"}]
            rollout(candidate, phase)
            marker = "processed-" + phase
            send(main, marker)
            until(marker, lambda: delivered(directory, phase, marker, phase), process)
        rollout(original, "restore-original-route")
        probe("main", "original-route-without-annotation")
        before = request(f"{pipelines}/main")
        assert before["pipeline"]["nodes"] == original["nodes"]
        invalid = deepcopy(before["pipeline"])
        invalid["connections"].append({"from": "missing", "to": "out"})
        try:
            request(f"{pipelines}/main?wait=true", "PUT", {"pipeline": invalid})
        except HTTPError as error:
            assert error.code == 422, (error.code, error.read())
        else:
            raise AssertionError("invalid topology was accepted")
        committed = request(root + "/config")
        desired = deepcopy(committed)
        desired["engine"].setdefault("telemetry", {})["reporting_interval"] = "47s"
        try:
            request(root + "/config/reconcile", "POST", {"config": desired})
        except HTTPError as error:
            assert error.code == 422 and json.load(error)["kind"] == "unsupported_mutation"
        else:
            raise AssertionError("startup-owned telemetry interval was accepted")
        assert request(root + "/config") == committed
        after = request(f"{pipelines}/main")
        assert (before["activeGeneration"], before["pipeline"]) == (
            after["activeGeneration"], after["pipeline"])
        probe("main", "still-serving-after-rejection")
        probe("observer", "observer-after-rejection")


# Scenario: add, update and remove a context-declaring receiver, then submit an invalid graph.
# Guarantees: the engine keeps both pipelines serving and commits only successful updates.
def run(engine):
    with tempfile.TemporaryDirectory() as tmp:
        directory = Path(tmp)
        with ExitStack() as stack:
            sockets = [stack.enter_context(socket.socket()) for _ in range(5)]
            for sock in sockets:
                sock.bind(("127.0.0.1", 0))
            ports = [sock.getsockname()[1] for sock in sockets]
        try:
            exercise(engine, directory, ports)
        except Exception as error:
            if isinstance(error, HTTPError):
                print(f"HTTP {error.code}: {error.read().decode(errors='replace')}", flush=True)
            print((directory / "engine.log").read_text(errors="replace")[-5000:], flush=True)
            raise


if __name__ == "__main__":
    run(from_args(__doc__))
