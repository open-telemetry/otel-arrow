import os
import pytest
from unittest.mock import MagicMock, patch

from docker.errors import BuildError, APIError
from lib.impl.strategies.hooks.docker.build_docker_image import (
    BuildDockerImage,
    BuildDockerImageConfig,
    BuildDockerImages,
    BuildDockerImagesConfig,
    build_image,
)
from lib.impl.component.managed_component import ManagedComponentConfiguration
from lib.impl.strategies.deployment.docker import (
    DockerBuildConfig,
    DockerDeploymentConfig,
)
from lib.impl.strategies.deployment.process import ProcessDeploymentConfig
from lib.runner.factory import build_managed_component
from lib.runner.wrappers import DeploymentWrapper

BUILD_MODULE = "lib.impl.strategies.hooks.docker.build_docker_image"


def build_component(name, deployment=None):
    wrapper = None
    if deployment is not None:
        strategy = (
            "docker" if isinstance(deployment, DockerDeploymentConfig) else "process"
        )
        wrapper = DeploymentWrapper.model_validate({strategy: deployment.model_dump()})
    return build_managed_component(
        name, ManagedComponentConfiguration(deployment=wrapper)
    )


# Scenario: Docker build hooks inspect a component that needs no Docker build.
# Guarantees: Missing deployment, process deployment, and prebuilt images do not
# contact Docker, even without the global --docker.no-build flag.
@pytest.mark.parametrize(
    "deployment",
    [
        None,
        ProcessDeploymentConfig(command="example"),
        DockerDeploymentConfig(image="x"),
    ],
)
@pytest.mark.parametrize("single_image", [False, True])
def test_build_hooks_skip_components_without_docker_build(deployment, single_image):
    component = build_component("existing-service", deployment)
    ctx = MagicMock()
    ctx.get_suite.return_value.get_runtime.return_value.docker_no_build = False
    ctx.get_components.return_value = {component.name: component}
    ctx.get_component_by_name.return_value = component
    hook = (
        BuildDockerImage(BuildDockerImageConfig(component=component.name))
        if single_image
        else BuildDockerImages(BuildDockerImagesConfig())
    )

    with patch(f"{BUILD_MODULE}.get_or_create_docker_client") as get_client:
        hook.execute(ctx)

    get_client.assert_not_called()


# Scenario: An existing component shares a suite with an image built by the harness.
# Guarantees: Automatic and explicit builds still build the Docker image, while
# --docker.no-build continues to suppress both builds and Docker connections.
@pytest.mark.parametrize("single_image", [False, True])
@pytest.mark.parametrize("no_build", [False, True])
def test_build_hooks_preserve_docker_builds_in_mixed_suites(single_image, no_build):
    deployment = DockerDeploymentConfig(
        image="generator:test", build=DockerBuildConfig(context="./generator")
    )
    generator = build_component("generator", deployment)
    ctx = MagicMock()
    ctx.get_suite.return_value.get_runtime.return_value.docker_no_build = no_build
    ctx.get_components.return_value = {
        "existing-service": build_component("existing-service"),
        "generator": generator,
    }
    ctx.get_component_by_name.return_value = generator
    hook = (
        BuildDockerImage(BuildDockerImageConfig(component=generator.name))
        if single_image
        else BuildDockerImages(BuildDockerImagesConfig())
    )

    with (
        patch(f"{BUILD_MODULE}.get_or_create_docker_client") as get_client,
        patch(f"{BUILD_MODULE}.build_image") as build,
    ):
        hook.execute(ctx)

    if no_build:
        get_client.assert_not_called()
        build.assert_not_called()
    else:
        get_client.assert_called_once_with(ctx)
        build.assert_called_once_with(
            get_client.return_value,
            deployment,
            ctx.get_logger.return_value,
            log_build=False,
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
