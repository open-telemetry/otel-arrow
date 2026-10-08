import os
import pytest
from unittest.mock import MagicMock, patch
from docker.errors import DockerException, APIError

from lib.impl.strategies.deployment.docker import (
    DockerDeployment,
    DockerDeploymentConfig,
    DockerUlimit,
    DockerVolumeMapping,
    DockerPortMapping,
    _split_volume_mount_string,
    build_port_bindings,
    build_ulimits,
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


# Scenario: A docker deployment config specifies extra_hosts (e.g. mapping
#   host.docker.internal to host-gateway).
# Guarantees: The mapping is forwarded unchanged to containers.run as the
#   extra_hosts kwarg so containers can reach services on the docker host.
@patch("lib.impl.strategies.deployment.docker.get_or_create_docker_client")
@patch("lib.impl.strategies.deployment.docker.get_component_docker_runtime")
@patch("lib.impl.strategies.deployment.docker.set_component_docker_runtime_data")
@patch(
    "lib.impl.strategies.deployment.docker.sanitize_docker_name",
    side_effect=lambda x: x,
)
def test_start_passes_extra_hosts(
    mock_sanitize,
    mock_set_runtime,
    mock_get_runtime,
    mock_docker_client,
    mock_component,
    mock_context,
):
    mock_client = MagicMock()
    mock_docker_client.return_value = mock_client
    config = DockerDeploymentConfig(
        image="my-image:latest",
        network="test-network",
        extra_hosts={"host.docker.internal": "host-gateway"},
    )

    DockerDeployment(config=config).start(mock_component, mock_context)

    kwargs = mock_client.containers.run.call_args.kwargs
    assert kwargs["extra_hosts"] == {"host.docker.internal": "host-gateway"}


# Scenario: A docker deployment config specifies a ulimit as a bare int
#   (e.g. {"nofile": 65536}).
# Guarantees: The value is forwarded to containers.run as a single Ulimit with
#   matching soft and hard limits, so the container's file-descriptor ceiling
#   is raised as requested.
@patch("lib.impl.strategies.deployment.docker.get_or_create_docker_client")
@patch("lib.impl.strategies.deployment.docker.get_component_docker_runtime")
@patch("lib.impl.strategies.deployment.docker.set_component_docker_runtime_data")
@patch(
    "lib.impl.strategies.deployment.docker.sanitize_docker_name",
    side_effect=lambda x: x,
)
def test_start_passes_ulimit_int(
    mock_sanitize,
    mock_set_runtime,
    mock_get_runtime,
    mock_docker_client,
    mock_component,
    mock_context,
):
    mock_client = MagicMock()
    mock_docker_client.return_value = mock_client
    config = DockerDeploymentConfig(
        image="my-image:latest",
        network="test-network",
        ulimits={"nofile": 65536},
    )

    DockerDeployment(config=config).start(mock_component, mock_context)

    ulimits = mock_client.containers.run.call_args.kwargs["ulimits"]
    assert len(ulimits) == 1
    assert ulimits[0]["Name"] == "nofile"
    assert ulimits[0]["Soft"] == 65536
    assert ulimits[0]["Hard"] == 65536


# Scenario: A docker deployment config specifies a ulimit with explicit soft
#   and hard values (e.g. {"nofile": {"soft": 4096, "hard": 65536}}).
# Guarantees: The soft and hard limits are forwarded independently to
#   containers.run so a lower soft limit can coexist with a higher hard ceiling.
@patch("lib.impl.strategies.deployment.docker.get_or_create_docker_client")
@patch("lib.impl.strategies.deployment.docker.get_component_docker_runtime")
@patch("lib.impl.strategies.deployment.docker.set_component_docker_runtime_data")
@patch(
    "lib.impl.strategies.deployment.docker.sanitize_docker_name",
    side_effect=lambda x: x,
)
def test_start_passes_ulimit_soft_hard(
    mock_sanitize,
    mock_set_runtime,
    mock_get_runtime,
    mock_docker_client,
    mock_component,
    mock_context,
):
    mock_client = MagicMock()
    mock_docker_client.return_value = mock_client
    config = DockerDeploymentConfig(
        image="my-image:latest",
        network="test-network",
        ulimits={"nofile": {"soft": 4096, "hard": 65536}},
    )

    DockerDeployment(config=config).start(mock_component, mock_context)

    ulimits = mock_client.containers.run.call_args.kwargs["ulimits"]
    assert len(ulimits) == 1
    assert ulimits[0]["Name"] == "nofile"
    assert ulimits[0]["Soft"] == 4096
    assert ulimits[0]["Hard"] == 65536


# Scenario: A docker deployment config leaves ulimits unset.
# Guarantees: No ulimits kwarg is passed to containers.run, preserving the
#   docker daemon's default limits for components that do not opt in.
@patch("lib.impl.strategies.deployment.docker.get_or_create_docker_client")
@patch("lib.impl.strategies.deployment.docker.get_component_docker_runtime")
@patch("lib.impl.strategies.deployment.docker.set_component_docker_runtime_data")
@patch(
    "lib.impl.strategies.deployment.docker.sanitize_docker_name",
    side_effect=lambda x: x,
)
def test_start_omits_ulimits_when_unset(
    mock_sanitize,
    mock_set_runtime,
    mock_get_runtime,
    mock_docker_client,
    mock_component,
    mock_context,
):
    mock_client = MagicMock()
    mock_docker_client.return_value = mock_client
    config = DockerDeploymentConfig(
        image="my-image:latest",
        network="test-network",
    )

    DockerDeployment(config=config).start(mock_component, mock_context)

    assert "ulimits" not in mock_client.containers.run.call_args.kwargs


# Scenario: build_ulimits is given a mix of bare-int and explicit soft/hard
#   ulimit specs.
# Guarantees: Each entry becomes one docker.types.Ulimit with the correct
#   name, soft, and hard values; bare ints apply to both soft and hard.
def test_build_ulimits_int_and_object():
    result = build_ulimits(
        {"nofile": 65536, "nproc": DockerUlimit(soft=1024, hard=2048)}
    )
    by_name = {u["Name"]: u for u in result}
    assert by_name["nofile"]["Soft"] == 65536
    assert by_name["nofile"]["Hard"] == 65536
    assert by_name["nproc"]["Soft"] == 1024
    assert by_name["nproc"]["Hard"] == 2048


# Scenario: build_ulimits is given None or an empty mapping.
# Guarantees: It returns an empty list so callers skip the ulimits kwarg
#   entirely rather than passing an empty/invalid value to the docker SDK.
@pytest.mark.parametrize("value", [None, {}])
def test_build_ulimits_empty(value):
    assert build_ulimits(value) == []


# Scenario: build_ulimits receives a boolean, which is a subclass of int.
# Guarantees: It raises TypeError instead of silently coercing True/False into
#   a file-descriptor limit.
def test_build_ulimits_rejects_bool():
    with pytest.raises(TypeError):
        build_ulimits({"nofile": True})


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


@pytest.mark.parametrize(
    "mount,expected_bind,expected_mode",
    [
        ("./data:C:/app/data", "C:/app/data", "rw"),
        ("./data:C:\\app\\data", "C:\\app\\data", "rw"),
        ("./config:C:/app/config:ro", "C:/app/config", "ro"),
        ("./config:C:\\app\\config:ro", "C:\\app\\config", "ro"),
    ],
)
# Scenario: a volume mount string whose container target is a Windows
# drive-letter path, with and without an explicit ':ro' mode suffix.
# Guarantees: the drive letter stays attached to its path instead of being
# parsed as a separate ':'-delimited field, so the container bind target and
# the mode are both recovered correctly.
def test_build_volume_bindings_windows_container_target(
    mount, expected_bind, expected_mode
):
    host_path = os.path.abspath(mount.split(":")[0])

    result = build_volume_bindings([mount])

    assert result == {host_path: {"bind": expected_bind, "mode": expected_mode}}


@pytest.mark.parametrize(
    "mount,expected_source,expected_target,expected_mode",
    [
        ("./host:C:/container", "./host", "C:/container", "rw"),
        ("./host:C:\\container", "./host", "C:\\container", "rw"),
        ("./host:C:/container:ro", "./host", "C:/container", "ro"),
        ("C:/host:D:/container", "C:/host", "D:/container", "rw"),
        (
            "C:\\host:D:\\container:ro",
            "C:\\host",
            "D:\\container",
            "ro",
        ),
        ("/host:/container", "/host", "/container", "rw"),
        ("/host:/container:ro", "/host", "/container", "ro"),
        ("./host:/container:rw", "./host", "/container", "rw"),
        ("x:/container", "x", "/container", "rw"),
        ("x:/container:ro", "x", "/container", "ro"),
    ],
)
# Scenario: volume mount strings cover Windows drive letters, Linux-style
# paths, explicit modes, and single-character relative host paths.
# Guarantees: only field-separator colons split SOURCE from TARGET and mode,
# while drive-letter colons remain inside their respective paths.
def test_split_volume_mount_string(
    mount, expected_source, expected_target, expected_mode
):
    assert _split_volume_mount_string(mount) == (
        expected_source,
        expected_target,
        expected_mode,
    )


@pytest.mark.parametrize(
    "mount,expected_mode",
    [
        ("x:/container", "rw"),
        ("x:/container:ro", "ro"),
    ],
)
# Scenario: a valid volume mount uses a single-character relative source path.
# Guarantees: the parser preserves pre-existing behavior by treating `x` as
# the host source, not as a Windows drive-qualified source path.
def test_build_volume_bindings_single_character_relative_host(
    mount, expected_mode
):
    result = build_volume_bindings([mount])

    assert result == {
        os.path.abspath("x"): {"bind": "/container", "mode": expected_mode}
    }


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
