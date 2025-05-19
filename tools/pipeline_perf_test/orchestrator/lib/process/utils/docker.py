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
import subprocess
from typing import Optional, Dict, List

from ..deployed_process.docker import DockerProcess

def launch_container(
    image_name: str,
    container_name: str,
    ports: Optional[Dict[str, str]] = None,
    network: Optional[str] = None,
    command_args: Optional[List[str]] = None,
    volumes: Optional[Dict[str, str]] = None,
    environment: Optional[Dict[str, str]] = None
) -> DockerProcess:
    """
    Launch a Docker container

    Args:
        image_name: Docker image name with tag
        container_name: name for the container
        ports: Port mappings from host to container
        network: Docker network to connect to
        command_args: Additional command arguments to pass to the container
        volumes: Volume mounts from host to container
        environment: Environment variables to pass to the container

    Returns:
        DockerProcess: Object representing the launched container
    """
    print(f"Starting {container_name} service using Docker image: {image_name}...")

    # Construct the Docker command
    cmd = ["docker", "run", "-d"]

    # Add container name
    cmd.extend(["--name", container_name])

    # Add network if specified
    if network:
        cmd.extend(["--network", network])

    # Add port mappings
    if ports:
        for host_port, container_port in ports.items():
            cmd.extend(["-p", f"{host_port}:{container_port}"])

    # Add volume mounts if provided
    if volumes:
        for host_path, container_path in volumes.items():
            cmd.extend(["-v", f"{host_path}:{container_path}"])

    # Add environment variables if provided
    if environment:
        for var_name, var_value in environment.items():
            cmd.extend(["-e", f"{var_name}={var_value}"])

    # Add the image name
    cmd.append(image_name)

    # Add any additional command arguments
    if command_args:
        cmd.extend(command_args)

    # Run the Docker container
    try:
        # Start the container and get its ID
        print(f"Running command: {' '.join(cmd)}")
        container_id = subprocess.check_output(cmd, text=True).strip()
        print(f"Docker container started with ID: {container_id}")

        # Return a TargetProcess object
        return DockerProcess(
            container_id=container_id
        )
    except subprocess.CalledProcessError as e:
        print(f"Error launching Docker container: {e}")
        print(f"Error output: {e.stderr if hasattr(e, 'stderr') else 'No error output'}")
        raise



def build_docker_image(image_name: str, dockerfile_dir: str) -> str:
    """
    Build a Docker image locally

    Args:
        image_name: Name and tag for the Docker image (e.g. "backend-service:latest")
        dockerfile_dir: Directory containing the Dockerfile

    Returns:
        str: Name of the built image
    """
    print(f"Building Docker image '{image_name}'...")

    # Get the absolute path to the directory
    dir_path = os.path.abspath(dockerfile_dir)

    # Build the Docker image
    try:
        cmd = ["docker", "build", "-t", image_name, dir_path]
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print(f"Successfully built Docker image: {image_name}")
        return image_name
    except subprocess.CalledProcessError as e:
        print(f"Error building Docker image '{image_name}': {e}")
        print(f"Error output: {e.stderr}")
        raise


def get_docker_logs(container_id: str) -> str:
    """
    Get logs from a Docker container

    Args:
        container_id: ID or name of the Docker container

    Returns:
        str: Container logs or empty string if error
    """
    try:
        cmd = ["docker", "logs", container_id]
        logs = subprocess.check_output(cmd, text=True)
        return logs
    except subprocess.CalledProcessError as e:
        print(f"Error getting Docker container logs: {e}")
        print(f"Error output: {e.stderr if hasattr(e, 'stderr') else 'No error output'}")
        return ""


def cleanup_docker_containers(container_names: List[str]) -> None:
    """
    Remove any existing Docker containers with the specified names

    Args:
        container_names: List of container names to clean up
    """
    print(f"Cleaning up any existing Docker containers with names: {container_names}")

    for name in container_names:
        try:
            # Check if the container exists
            inspect_cmd = ["docker", "ps", "-a", "-q", "-f", f"name={name}"]
            container_id = subprocess.run(inspect_cmd, check=True, capture_output=True, text=True).stdout.strip()

            if container_id:
                # Container exists, remove it forcefully
                print(f"Removing existing container: {name} (ID: {container_id})")
                rm_cmd = ["docker", "rm", "-f", name]
                subprocess.run(rm_cmd, check=True, capture_output=True, text=True)
        except subprocess.CalledProcessError as e:
            print(f"Error checking/removing container '{name}': {e}")
            # Continue with other containers
