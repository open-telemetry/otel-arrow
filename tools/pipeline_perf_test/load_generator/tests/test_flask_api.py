import sys
import os
import time
import pytest

# Add root dir to sys.path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from loadgen import app  # noqa: E402


@pytest.fixture
def client():
    app.config["TESTING"] = True
    with app.test_client() as client:
        yield client


def test_start_with_valid_config(client):
    config = {
        "body_size": 10,
        "num_attributes": 2,
        "attribute_value_size": 10,
        "batch_size": 2,
        "threads": 1,
        "target_rate": 100,
    }

    resp = client.post("/start", json=config)
    assert resp.status_code == 200
    assert resp.get_json()["status"] == "started"

    # Let it run a little
    time.sleep(0.2)
    client.post("/stop")


def test_start_with_invalid_config(client):
    invalid_config = {
        "body_size": -1,  # Invalid
        "num_attributes": 2,
        "attribute_value_size": 10,
        "batch_size": 2,
        "threads": 1,
        "target_rate": 100,
    }

    resp = client.post("/start", json=invalid_config)
    assert resp.status_code == 400
    assert "error" in resp.get_json()


def test_double_start_is_rejected(client):
    config = {
        "body_size": 10,
        "num_attributes": 1,
        "attribute_value_size": 10,
        "batch_size": 2,
        "threads": 1,
    }

    resp1 = client.post("/start", json=config)
    assert resp1.status_code == 200

    resp2 = client.post("/start", json=config)
    assert resp2.status_code == 400
    assert "error" in resp2.get_json()

    client.post("/stop")


def test_stop_endpoint(client):
    config = {
        "body_size": 10,
        "num_attributes": 1,
        "attribute_value_size": 10,
        "batch_size": 1,
        "threads": 1,
    }

    client.post("/start", json=config)
    time.sleep(0.1)
    resp = client.post("/stop")
    assert resp.status_code == 200
    assert resp.get_json()["status"] == "stopped"


def test_metrics_endpoint(client):
    config = {
        "body_size": 10,
        "num_attributes": 1,
        "attribute_value_size": 10,
        "batch_size": 2,
        "threads": 1,
    }

    client.post("/start", json=config)
    time.sleep(0.2)
    client.post("/stop")

    resp = client.get("/metrics")
    assert resp.status_code == 200

    # Metrics returned as plain text
    body = resp.data.decode("utf-8")
    lines = body.splitlines()
    keys = [line.split()[0] for line in lines]

    assert "sent" in keys
    assert "failed" in keys
    assert "bytes_sent" in keys
