"""
kubernetes.py

Utility functions for managing Kubernetes resources used in OpenTelemetry pipeline performance
testing.

This module provides helper functions to deploy resources, retrieve logs, manage namespaces,
handle port forwarding, and run Kubernetes-based load generators. It abstracts the underlying
`kubectl` operations to streamline orchestration workflows within a Kubernetes environment.

These functions are intended to support orchestration of performance testing scenarios on Kubernetes
clusters.
"""
import os
import subprocess
import time
from typing import Tuple

from ..deployed_process.kubernetes import K8sDeployedResource
from ...report.report import parse_logs_for_sent_count

def deploy_kubernetes_resources(manifest_path: str, deployment_name: str, namespace: str = "default") -> K8sDeployedResource:
    """
    Deploy resources to Kubernetes using kubectl apply

    Args:
        manifest_path: Path to the Kubernetes YAML manifest file
        deployment_name: Name of the deployment to be created
        namespace: Kubernetes namespace to deploy to

    Returns:
        K8sDeployedResource: Object representing the deployed resources
    """
    print(f"Deploying '{deployment_name}' to Kubernetes using manifest: {manifest_path}...")

    try:
        # Apply the manifest to create the resources
        cmd = ["kubectl", "apply", "-f", manifest_path, "-n", namespace]
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print(f"Successfully deployed '{deployment_name}' resources to Kubernetes")

        # Return a K8sDeployedResource object
        return K8sDeployedResource(
            deployment_name=deployment_name,
            manifest_path=manifest_path,
            namespace=namespace
        )
    except subprocess.CalledProcessError as e:
        print(f"Error deploying Kubernetes resources: {e}")
        print(f"Error output: {e.stderr}")
        raise

def get_k8s_logs(pod_selector: str, namespace: str = "default") -> str:
    """
    Get logs from a Kubernetes pod

    Args:
        pod_selector: Label selector to identify the pod (e.g., 'app=loadgen')
        namespace: Kubernetes namespace

    Returns:
        str: Pod logs or empty string if error
    """
    try:
        # First, get the pod name matching the selector
        cmd = ["kubectl", "get", "pods", "-l", pod_selector, "-n", namespace,
               "-o", "jsonpath={.items[0].metadata.name}"]
        pod_name = subprocess.check_output(cmd, text=True).strip()

        if not pod_name:
            print(f"No pod found with selector: {pod_selector}")
            return ""

        # Get logs from the pod
        cmd = ["kubectl", "logs", pod_name, "-n", namespace]
        logs = subprocess.check_output(cmd, text=True)
        return logs
    except subprocess.CalledProcessError as e:
        print(f"Error getting pod logs: {e}")
        print(f"Error output: {e.stderr if hasattr(e, 'stderr') else 'No error output'}")
        return ""


def setup_k8s_port_forwarding(service_name: str, namespace: str, local_port: int, remote_port: int) -> subprocess.Popen:
    """
    Set up port forwarding from a local port to a service in Kubernetes.

    Args:
        service_name: Name of the Kubernetes service to forward to
        namespace: Kubernetes namespace
        local_port: Local port to forward from
        remote_port: Remote port in the service to forward to

    Returns:
        subprocess.Popen: The process object for the port forwarding command
    """
    print(f"Setting up port forwarding from localhost:{local_port} to {service_name}:{remote_port} in namespace {namespace}...")

    # Build the kubectl port-forward command
    cmd = [
        "kubectl", "port-forward",
        f"service/{service_name}",
        f"{local_port}:{remote_port}",
        "-n", namespace
    ]

    # Start the port forwarding in a subprocess
    process = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    # Give it a moment to establish the connection
    time.sleep(2)

    # Check if the process is still running
    if process.poll() is not None:
        # Process terminated prematurely
        stdout, stderr = process.communicate()
        print(f"Port forwarding failed to start: {stderr}")
        return None

    print(f"Port forwarding established successfully")
    return process

def create_k8s_namespace(namespace: str) -> bool:
    """
    Create a Kubernetes namespace if it doesn't exist

    Args:
        namespace: Name of the namespace to create

    Returns:
        bool: True if successful, False otherwise
    """
    print(f"Ensuring Kubernetes namespace exists: {namespace}")

    try:
        # Check if the namespace already exists
        check_cmd = ["kubectl", "get", "namespace", namespace, "--no-headers", "--ignore-not-found"]
        result = subprocess.run(check_cmd, check=True, capture_output=True, text=True)

        if namespace in result.stdout:
            print(f"Namespace {namespace} already exists")
            return True

        # Create the namespace
        create_cmd = ["kubectl", "create", "namespace", namespace]
        result = subprocess.run(create_cmd, check=True, capture_output=True, text=True)
        print(f"Created Kubernetes namespace: {namespace}")
        return True

    except subprocess.CalledProcessError as e:
        print(f"Error creating Kubernetes namespace: {e}")
        print(f"Error output: {e.stderr}")
        return False

def run_k8s_loadgen(loadgen_manifest: str, namespace: str, duration: int, k8s_collector_resource: K8sDeployedResource, loadgen_image: str) -> Tuple[int, int, float]:
    """
    Deploy and run the load generator in Kubernetes and return the counts of logs and duration

    Args:
        loadgen_manifest: Path to the load generator Kubernetes manifest
        namespace: Kubernetes namespace
        duration: Test duration in seconds
        k8s_collector_resource: The collector resource to monitor
        loadgen_image: The loadgen Docker image name/tag to use

    Returns:
        Tuple[int, int, float]: Count of logs sent, logs failed, and actual duration
    """
    print("Starting load generator in Kubernetes...")
    print(f"Using loadgen image: {loadgen_image}")

    # Modify the manifest to set the correct duration and image
    # Read the manifest
    with open(loadgen_manifest, 'r') as f:
        manifest_content = f.read()

    # Replace the {{DURATION}} placeholder with the actual duration
    # and the image name with the provided loadgen_image
    temp_manifest = f"{loadgen_manifest}.tmp"
    updated_manifest = manifest_content.replace("{{DURATION}}", str(duration))
    updated_manifest = updated_manifest.replace("otel-loadgen:latest", loadgen_image)

    print(f"Setting loadgen duration to {duration}s")

    # Write to a temporary file
    with open(temp_manifest, 'w') as f:
        f.write(updated_manifest)

    start_time = time.time()

    # Deploy load generator
    loadgen_resource = deploy_kubernetes_resources(temp_manifest, "otel-loadgen", namespace)

    # Wait for the load generator job to complete
    print(f"Waiting for loadgen job to complete (expected duration: {duration}s)...")

    # Job might take time to get created, so we'll wait a bit before polling
    time.sleep(5)

    # Start monitoring the collector resource, this should all get refactored to avoid
    # relying on sleep in favor of explicit ready / start / stop / error signals.
    k8s_collector_resource.start_monitoring(duration / 10)

    # Poll for job completion
    completed = False
    max_wait = duration + 30  # Add buffer time
    wait_start = time.time()

    while not completed and (time.time() - wait_start) < max_wait:
        try:
            cmd = ["kubectl", "get", "job", "otel-loadgen", "-n", namespace,
                  "-o", "jsonpath={.status.succeeded}"]
            result = subprocess.run(cmd, check=True, capture_output=True, text=True)
            if result.stdout.strip() == "1":
                completed = True
                print("Load generator job completed successfully")
                break
            time.sleep(5)  # Check every 5 seconds
        except subprocess.CalledProcessError:
            time.sleep(5)  # Continue checking

    # High likelyhood that this will have accumulated samples from the idle collector after the job finished.
    # Better handling for component start/stop + test start/stop will be important in future PRs.
    k8s_collector_resource.stop_monitoring()

    if not completed:
        print("Warning: Load generator job didn't complete in the expected time, getting logs anyway")

    # Get logs from the loadgen pod
    logs = get_k8s_logs("app=loadgen", namespace)

    # Use the expected duration since the actual might be hard to determine precisely
    actual_duration = duration

    # Parse the output to extract logs sent count
    logs_sent, logs_failed = parse_logs_for_sent_count(logs)

    if logs_sent == 0:
        print(f"Could not find LOADGEN_LOGS_SENT in logs")
    else:
        print(f"Load generator completed. Sent {logs_sent} logs, Failed {logs_failed} logs in {actual_duration:.2f}s")

    # Clean up the temporary manifest
    try:
        os.remove(temp_manifest)
    except:
        pass

    return logs_sent, logs_failed, actual_duration

