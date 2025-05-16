"""
docker.py

Utility functions for managing Docker containers used in OpenTelemetry performance testing.

This module provides helper functions for building images, launching containers,
fetching logs, and cleaning up resources. It is used internally by orchestration
logic to support Docker-based test deployments.

These utilities abstract lower-level Docker CLI operations to simplify higher-level process
orchestration.
"""
import os
from dataclasses import dataclass
from typing import Optional, Dict, List

import docker
from docker.errors import BuildError, APIError, DockerException, NotFound

from ..deployed_process.docker import DockerProcess


@dataclass
class VolumeMount:
    host_path: str
    container_path: str
    mode: str = 'rw'


@dataclass
class PortBinding:
    container_port: int
    host_port: int
    protocol: str = "tcp"
    host_ip: str = "0.0.0.0"


def build_volume_dict(volume_mounts: list[VolumeMount]) -> dict:
    """Map a list of VolumeMounts to the format expected by docker api"

    Args:
        - volume_mounts: a list of VolumeMount objects specifying mount configs
    """
    return {
        vm.host_path: {'bind': vm.container_path, 'mode': vm.mode}
        for vm in volume_mounts
    }


def build_port_bindings(bindings: list[PortBinding]) -> dict:
    """Map a list of PortBindings to the format expected by docker api"

    Args:
        - bindings: a list of PortBindings objects specifying port mapping configs
    """
    result = {}
    for b in bindings:
        key = f"{b.container_port}/{b.protocol}"
        result[key] = (b.host_ip, b.host_port)
    return result


def launch_container(
    image_name: str,
    container_name: str,
    client: docker.DockerClient,
    ports: Optional[List[PortBinding]] = None,
    network: Optional[str] = None,
    command_args: Optional[List[str]] = None,
    volume_mounts: Optional[List[VolumeMount]] = None,
    environment: Optional[Dict[str, str]] = None,
    log_cli: bool = False,
) -> DockerProcess:
    """
    Launch a Docker container

    Args:
        image_name: Docker image name with tag
        container_name: name for the container
        client: docker api client
        ports: Port mappings from host to container
        network: Docker network to connect to
        command_args: Additional command arguments to pass to the container
        volumes: Volume mounts from host to container
        environment: Environment variables to pass to the container
        log_cli: Enable logging the equivalent cli command

    Returns:
        DockerProcess: Object representing the launched container
    """
    print(f"Starting {container_name} service using Docker image: {image_name}...")

    # Prepare port bindings
    port_bindings = build_port_bindings(ports) if ports else None

    # Prepare volume bindings
    volume_bindings = build_volume_dict(volume_mounts) if volume_mounts else {}

    if log_cli:
        port_parts = [
            f"-p {p.host_ip}:{p.host_port}:{p.container_port}/{p.protocol}"
            for p in ports or []
        ]
        volume_parts = [
            f"-v {v.host_path}:{v.container_path}:{v.mode}"
            for v in volume_mounts or []
        ]
        env_parts = [
            f"-e {key}={value}"
            for key, value in (environment or {}).items()
        ]
        args_str = " ".join(command_args) if command_args else ""
        parts = [
            "docker run -d",
            f"--name {container_name}",
            f"--network {network}",
            *port_parts,
            *volume_parts,
            *env_parts,
            image_name,
            args_str
        ]
        cli_command = " ".join(part for part in parts if part.strip())
        print(f"CLI_COMMAND = {cli_command}")

    try:
        container = client.containers.run(
            image=image_name,
            name=container_name,
            detach=True,
            network=network,
            ports=port_bindings,
            volumes=volume_bindings,
            environment=environment,
            command=command_args
        )
        print(f"Docker container started with ID: {container.id}")
        return DockerProcess(container_id=container.id, client=client, log_cli=log_cli)
    except DockerException as e:
        print(f"Error launching Docker container: {e}")
        raise


def build_docker_image(
        image_name: str,
        dockerfile_dir: str,
        client: docker.DockerClient,
        log_build: bool=False,
        log_cli: bool = False,
    ) -> str:
    """
    Build a Docker image locally

    Args:
        image_name: Name and tag for the Docker image (e.g. "backend-service:latest")
        dockerfile_dir: Directory containing the Dockerfile
        client: docker api client
        log_cli: Enable logging the equivalent cli command

    Returns:
        str: Name of the built image
    """
    print(f"Building Docker image '{image_name}'...")

    dir_path = os.path.abspath(dockerfile_dir)

    if log_cli:
        print(f"CLI_COMMAND = docker build -t {image_name} {dir_path}")

    try:
        image, build_logs = client.images.build(path=dir_path, tag=image_name, rm=True)

        if log_build:
            for chunk in build_logs:
                if 'stream' in chunk:
                    print(chunk['stream'].strip())

        print(f"Successfully built Docker image: {image.tags}")
        return image_name
    except (BuildError, APIError) as e:
        print(f"Error building Docker image '{image_name}': {e}")
        raise


def get_docker_logs(container_id: str, client: docker.DockerClient, log_cli: bool = False) -> str:
    """
    Get logs from a Docker container

    Args:
        container_id: ID or name of the Docker container
        client: docker api client
        log_cli: Enable logging the equivalent cli command

    Returns:
        str: Container logs or empty string if error
    """
    if log_cli:
        print(f"CLI_COMMAND = docker logs {container_id}")
    try:
        container = client.containers.get(container_id)
        logs = container.logs(stdout=True, stderr=True, stream=False, timestamps=False)
        return logs.decode("utf-8") if isinstance(logs, bytes) else str(logs)
    except (NotFound, APIError) as e:
        print(f"Error getting Docker container logs: {e}")
        return ""


def cleanup_docker_containers(container_names: List[str], client: docker.DockerClient, log_cli: bool = False) -> None:
    """
    Remove any existing Docker containers with the specified names

    Args:
        container_names: List of container names to clean up
        client: docker api client
        log_cli: Enable logging the equivalent cli command

    """
    print(f"Cleaning up any existing Docker containers with names: {container_names}")

    for name in container_names:
        try:
            # Try to get the container by name
            container = client.containers.get(name)
            print(f"Removing existing container: {name} (ID: {container.id})")
            if log_cli:
                print(f"CLI_COMMAND = docker rm -f {name}")
            container.remove(force=True)
        except NotFound:
            print(f"Container '{name}' not found. Skipping.")
        except APIError as e:
            print(f"Error removing container '{name}': {e}. Skipping.")


def create_docker_network(network_name: str, client: docker.DockerClient, log_cli: bool = False) -> bool:
    """
    Creates a Docker network using the Docker SDK. If the network already exists, it will be reused.

    Args:
        network_name: a string representing the name of the docker network to use
        client: docker api client
        log_cli: Enable logging the equivalent cli command

    Returns:
        boolean value indicating if the network is newly created (true) or existing (false)
    """
    try:
        # Check if the network already exists
        existing_networks = client.networks.list(names=[network_name])
        if existing_networks:
            print(f"Using existing Docker network: {network_name}")
            return False

        # Create the network
        if log_cli:
            print(f"CLI_COMMAND = docker network create {network_name}")
        client.networks.create(name=network_name)
        print(f"Created Docker network: {network_name}")
    except APIError as e:
        print(f"Error creating network: {e}")
        raise

    return True


def delete_docker_network(network_name: str, client: docker.DockerClient, log_cli: bool = False) -> None:
    """
    Deletes a Docker network using the Docker SDK.
    Args:
        network_name: a string representing the name of the docker network to use
        client: docker api client
        log_cli: Enable logging the equivalent cli command
    If the network does not exist, it will print a warning.
    """
    try:
        network = client.networks.get(network_name)
        if log_cli:
            print(f"CLI_COMMAND = docker network rm -f {network_name}")
        network.remove()
        print(f"Deleted Docker network: {network_name}")
    except NotFound:
        print(f"Docker network '{network_name}' not found.")
    except APIError as e:
        print(f"Error removing Docker network: {e}")
        raise
