import os
import pytest
from unittest.mock import MagicMock, patch
from docker.errors import DockerException, APIError

from lib.impl.strategies.deployment.docker import (
    DockerDeployment,
    DockerDeploymentConfig,
    DockerVolumeMapping,
    DockerPortMapping,
    build_port_bindings,
    build_volume_bindings,
)
from lib.core.component import Component
from lib.core.context.framework_element_contexts import StepContext


@pytest.fixture
def sample_config():
    return DockerDeploymentConfig(
        image="my-image:latest",
        network="test-network",
        ports=["8080:80"],
        volumes=["./host:/container"],
        environment={"ENV_VAR": "value"},
        command=["python", "app.py"],
    )


@pytest.fixture
def mock_component():
    mock = MagicMock(spec=Component)
    mock.name = "test-component"
    return mock


@pytest.fixture
def mock_context():
    ctx = MagicMock(spec=StepContext)
    ctx.get_logger.return_value = MagicMock()
    return ctx


@patch("lib.impl.strategies.deployment.docker.get_or_create_docker_client")
@patch("lib.impl.strategies.deployment.docker.get_component_docker_runtime")
@patch("lib.impl.strategies.deployment.docker.set_component_docker_runtime_data")
@patch(
    "lib.impl.strategies.deployment.docker.build_port_bindings",
    return_value={"8080/tcp": 8080},
)
@patch(
    "lib.impl.strategies.deployment.docker.build_volume_bindings",
    return_value={"/host": {"bind": "/container", "mode": "rw"}},
)
@patch(
    "lib.impl.strategies.deployment.docker.sanitize_docker_name",
    side_effect=lambda x: x,
)
def test_start_successful(
    mock_sanitize,
    mock_volumes,
    mock_ports,
    mock_set_runtime,
    mock_get_runtime,
    mock_docker_client,
    sample_config,
    mock_component,
    mock_context,
):
    # Setup
    mock_container = MagicMock()
    mock_container.id = "container123"
    mock_client = MagicMock()
    mock_client.containers.run.return_value = mock_container
    mock_docker_client.return_value = mock_client
    mock_runtime = MagicMock()
    mock_get_runtime.return_value = mock_runtime

    deployment = DockerDeployment(config=sample_config)

    # Execute
    deployment.start(mock_component, mock_context)

    # Assertions
    mock_client.containers.run.assert_called_once_with(
        image=sample_config.image,
        name="test-component",
        detach=True,
        network="test-network",
        ports={"8080/tcp": 8080},
        volumes={"/host": {"bind": "/container", "mode": "rw"}},
        environment=sample_config.environment,
        command=sample_config.command,
    )
    assert mock_runtime.container_id == "container123"
    mock_set_runtime.assert_called_once_with(mock_context, mock_runtime)


@patch("lib.impl.strategies.deployment.docker.get_or_create_docker_client")
@patch("lib.impl.strategies.deployment.docker.get_component_docker_runtime")
@patch(
    "lib.impl.strategies.deployment.docker.build_port_bindings",
    return_value={"8080/tcp": 8080},
)
@patch(
    "lib.impl.strategies.deployment.docker.build_volume_bindings",
    return_value={"/host": {"bind": "/container", "mode": "rw"}},
)
@patch(
    "lib.impl.strategies.deployment.docker.sanitize_docker_name",
    side_effect=lambda x: x,
)
def test_start_raises_docker_exception(
    mock_volumes,
    mock_ports,
    mock_sanitize,
    mock_get_runtime,
    mock_docker_client,
    sample_config,
    mock_component,
    mock_context,
):
    mock_client = MagicMock()
    mock_client.containers.run.side_effect = DockerException("Docker error")
    mock_docker_client.return_value = mock_client

    deployment = DockerDeployment(config=sample_config)

    with pytest.raises(DockerException):
        deployment.start(mock_component, mock_context)

    logger = mock_context.get_logger.return_value
    logger.error.assert_called_once_with(
        "Error launching Docker container: Docker error"
    )


@patch("lib.impl.strategies.deployment.docker.stop_and_remove_container")
@patch("lib.impl.strategies.deployment.docker.get_component_docker_runtime")
@patch("lib.impl.strategies.deployment.docker.get_or_create_docker_client")
def test_stop_successful(
    mock_get_client,
    mock_get_runtime,
    mock_stop_container,
    mock_component,
    mock_context,
):
    runtime = MagicMock()
    runtime.container_id = "abc123"
    mock_get_runtime.return_value = runtime

    deployment = DockerDeployment(config=MagicMock())

    deployment.stop(mock_component, mock_context)

    mock_stop_container.assert_called_once_with(
        mock_context, mock_get_client.return_value, "abc123"
    )

    logger = mock_context.get_logger.return_value
    logger.debug.assert_any_call(
        f"Stopping Docker container for {mock_component.name}, with ID: abc123"
    )


@patch("lib.impl.strategies.deployment.docker.get_component_docker_runtime")
@patch("lib.impl.strategies.deployment.docker.get_or_create_docker_client")
def test_stop_raises_runtime_error_if_no_container_id(
    mock_get_client,
    mock_get_runtime,
    mock_component,
    mock_context,
):
    runtime = MagicMock()
    runtime.container_id = None
    mock_get_runtime.return_value = runtime

    deployment = DockerDeployment(config=MagicMock())

    with pytest.raises(RuntimeError) as exc_info:
        deployment.stop(mock_component, mock_context)

    assert f"No container ID found for component '{mock_component.name}'" in str(
        exc_info.value
    )


@patch("lib.impl.strategies.deployment.docker.stop_and_remove_container")
@patch("lib.impl.strategies.deployment.docker.get_component_docker_runtime")
@patch("lib.impl.strategies.deployment.docker.get_or_create_docker_client")
def test_stop_propagates_docker_exceptions(
    mock_get_client,
    mock_get_runtime,
    mock_stop_container,
    mock_component,
    mock_context,
):
    runtime = MagicMock()
    runtime.container_id = "abc123"
    mock_get_runtime.return_value = runtime
    mock_stop_container.side_effect = APIError("Docker API failed")

    deployment = DockerDeployment(config=MagicMock())

    with pytest.raises(APIError, match="Docker API failed"):
        deployment.stop(mock_component, mock_context)


def test_build_volume_bindings_valid_string_default_mode():
    volume = f"./data:/app/data"
    expected_host_path = os.path.abspath("./data")

    result = build_volume_bindings([volume])

    assert result == {expected_host_path: {"bind": "/app/data", "mode": "rw"}}


def test_build_volume_bindings_valid_string_readonly():
    volume = f"./config:/app/config:ro"
    expected_host_path = os.path.abspath("./config")

    result = build_volume_bindings([volume])

    assert result == {expected_host_path: {"bind": "/app/config", "mode": "ro"}}


def test_build_volume_bindings_invalid_string_format():
    with pytest.raises(ValueError, match="Invalid volume mount string"):
        build_volume_bindings(["invalidstring"])


def test_build_volume_bindings_invalid_string_mode():
    with pytest.raises(ValueError, match="Invalid volume mount string"):
        build_volume_bindings(["./data:/app:data:bad"])


def test_build_volume_bindings_valid_object():
    vm = DockerVolumeMapping(source="./src", target="/dest", read_only=True)
    expected_host_path = os.path.abspath("./src")

    result = build_volume_bindings([vm])

    assert result == {expected_host_path: {"bind": "/dest", "mode": "ro"}}


def test_build_volume_bindings_invalid_type():
    with pytest.raises(TypeError, match="Invalid type in volume_mounts"):
        build_volume_bindings([123])  # Not a string or DockerVolumeMapping


def test_build_volume_bindings_none_or_empty():
    assert build_volume_bindings(None) == {}
    assert build_volume_bindings([]) == {}


@pytest.mark.parametrize(
    "mount,expected_mode",
    [
        ("./ro-path:/container:ro", "ro"),
        ("./rw-path:/container", "rw"),
    ],
)
def test_build_volume_bindings_param_string_modes(mount, expected_mode):
    result = build_volume_bindings([mount])
    host_path = os.path.abspath(mount.split(":")[0])
    assert result == {host_path: {"bind": "/container", "mode": expected_mode}}


def test_build_port_bindings_simple_string():
    result = build_port_bindings(["8080:80"])
    assert result == {"80/tcp": ("0.0.0.0", 8080)}


def test_build_port_bindings_with_host_ip():
    result = build_port_bindings(["127.0.0.1:8080:80"])
    assert result == {"80/tcp": ("127.0.0.1", 8080)}


def test_build_port_bindings_with_protocol():
    result = build_port_bindings(["8080:80/udp"])
    assert result == {"80/udp": ("0.0.0.0", 8080)}


def test_build_port_bindings_ip_and_protocol():
    result = build_port_bindings(["127.0.0.1:8080:80/udp"])
    assert result == {"80/udp": ("127.0.0.1", 8080)}


def test_build_port_bindings_with_object():
    mapping = DockerPortMapping(
        host_ip="0.0.0.0", published=8080, target=80, protocol="tcp"
    )
    result = build_port_bindings([mapping])
    assert result == {"80/tcp": ("0.0.0.0", 8080)}


def test_build_port_bindings_invalid_string_format():
    with pytest.raises(ValueError, match="Invalid port mapping string"):
        build_port_bindings(["8080"])


def test_build_port_bindings_too_many_parts():
    with pytest.raises(ValueError, match="Invalid port mapping string"):
        build_port_bindings(["a:b:c:d"])


def test_build_port_bindings_invalid_type():
    with pytest.raises(TypeError, match="Invalid type in bindings list"):
        build_port_bindings([42])


def test_build_port_bindings_empty_or_none():
    assert build_port_bindings(None) == {}
    assert build_port_bindings([]) == {}


@pytest.mark.parametrize(
    "input_str, expected",
    [
        ("8080:80", {"80/tcp": ("0.0.0.0", 8080)}),
        ("127.0.0.1:8080:80", {"80/tcp": ("127.0.0.1", 8080)}),
        ("8080:80/udp", {"80/udp": ("0.0.0.0", 8080)}),
        ("127.0.0.1:8080:80/udp", {"80/udp": ("127.0.0.1", 8080)}),
    ],
)
def test_build_port_bindings_param(input_str, expected):
    result = build_port_bindings([input_str])
    assert result == expected
