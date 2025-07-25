import os
import pytest
from unittest.mock import MagicMock

from docker.errors import BuildError, APIError
from lib.impl.strategies.hooks.docker.build_docker_image import build_image
from lib.impl.strategies.deployment.docker import (
    DockerBuildConfig,
    DockerDeploymentConfig,
)


def test_build_image_success_with_logs():
    # Setup
    mock_client = MagicMock()
    mock_logger = MagicMock()
    config = DockerDeploymentConfig(
        image="my-image:latest",
        build=DockerBuildConfig(
            context="./docker", args={"ARG1": "value"}, target="prod"
        ),
    )

    abs_path = os.path.abspath(config.build.context)

    mock_image = MagicMock()
    mock_image.tags = [config.image]
    mock_logs = [{"stream": "Step 1/2"}, {"stream": "Step 2/2"}]

    mock_client.images.build.return_value = (mock_image, mock_logs)

    # Execute
    result = build_image(mock_client, config, mock_logger, log_build=True)

    # Assert
    mock_client.images.build.assert_called_once_with(
        path=abs_path,
        tag=config.image,
        rm=True,
        buildargs=config.build.args,
        target=config.build.target,
    )

    mock_logger.debug.assert_any_call("Step 1/2")
    mock_logger.debug.assert_any_call("Step 2/2")
    mock_logger.debug.assert_any_call(
        f"Successfully built Docker image: {mock_image.tags}"
    )
    assert result == config.image


def test_build_image_success_no_logs():
    mock_client = MagicMock()
    mock_logger = MagicMock()
    config = DockerDeploymentConfig(
        image="test:no-log", build=DockerBuildConfig(context="./build")
    )
    mock_image = MagicMock()
    mock_image.tags = [config.image]
    mock_logs = [{"stream": "ignored"}]

    mock_client.images.build.return_value = (mock_image, mock_logs)

    result = build_image(mock_client, config, mock_logger, log_build=False)

    # Logs from stream should NOT be called
    mock_logger.debug.assert_called_with(
        f"Successfully built Docker image: {mock_image.tags}"
    )
    assert result == config.image


def test_build_image_raises_build_error():
    mock_client = MagicMock()
    mock_logger = MagicMock()
    config = DockerDeploymentConfig(
        image="fail:build", build=DockerBuildConfig(context="./broken")
    )

    mock_client.images.build.side_effect = BuildError("build failed", build_log="")

    with pytest.raises(BuildError):
        build_image(mock_client, config, mock_logger, log_build=True)

    mock_logger.debug.assert_called_with(
        f"Error building Docker image '{config.image}': build failed"
    )


def test_build_image_raises_api_error():
    mock_client = MagicMock()
    mock_logger = MagicMock()
    config = DockerDeploymentConfig(
        image="fail:api", build=DockerBuildConfig(context="./api-error")
    )

    mock_client.images.build.side_effect = APIError("api error")

    with pytest.raises(APIError):
        build_image(mock_client, config, mock_logger, log_build=False)

    mock_logger.debug.assert_called_with(
        f"Error building Docker image '{config.image}': api error"
    )


def test_build_image_logs_without_stream_key():
    mock_client = MagicMock()
    mock_logger = MagicMock()
    config = DockerDeploymentConfig(
        image="image:no-stream", build=DockerBuildConfig(context="./no-stream")
    )

    mock_image = MagicMock()
    mock_image.tags = [config.image]
    mock_logs = [{"something": "unexpected"}]

    mock_client.images.build.return_value = (mock_image, mock_logs)

    build_image(mock_client, config, mock_logger, log_build=True)

    mock_logger.debug.assert_called_with(
        f"Successfully built Docker image: {mock_image.tags}"
    )
