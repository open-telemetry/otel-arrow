#!/bin/bash
# Recreate a Multipass Ubuntu VM, verify Linux user_events support, and run the
# userevents exporter -> receiver E2E smoke test there.
#
# Usage:
#   ./scripts/run-userevents-multipass-e2e.sh [vm-name]

set -euo pipefail

VM_NAME="${1:-otel-user-events}"
# Default to an LTS release because interim releases (for example 24.10) can
# go end-of-life quickly and disappear from the primary apt mirrors.
UBUNTU_RELEASE="${UBUNTU_RELEASE:-24.04}"
VM_CPUS="${VM_CPUS:-4}"
VM_MEMORY="${VM_MEMORY:-8G}"
VM_DISK="${VM_DISK:-30G}"
VM_MOUNT_PATH="${VM_MOUNT_PATH:-/workspace/otel-arrow}"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
OTAP_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
REPO_ROOT="$(cd -- "${OTAP_DIR}/../.." && pwd)"

require_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "error: required command not found: $1" >&2
        exit 1
    fi
}

run_in_vm() {
    local cmd="$1"
    multipass exec "${VM_NAME}" -- bash -lc "$cmd"
}

wait_for_vm_exec() {
    local attempts=60
    local delay=2
    local i
    for ((i=1; i<=attempts; i+=1)); do
        if multipass exec "${VM_NAME}" -- true >/dev/null 2>&1; then
            return 0
        fi
        sleep "${delay}"
    done

    echo "error: VM '${VM_NAME}' did not become reachable via multipass exec in time" >&2
    exit 1
}

require_cmd multipass

echo "Recreating Multipass VM '${VM_NAME}'..."
if multipass info "${VM_NAME}" >/dev/null 2>&1; then
    multipass stop "${VM_NAME}" >/dev/null 2>&1 || true
    multipass delete "${VM_NAME}" >/dev/null 2>&1 || true
    multipass purge >/dev/null 2>&1 || true
fi

multipass launch "${UBUNTU_RELEASE}" \
    --name "${VM_NAME}" \
    --cpus "${VM_CPUS}" \
    --memory "${VM_MEMORY}" \
    --disk "${VM_DISK}"

echo "Mounting repo into VM..."
multipass mount "${REPO_ROOT}" "${VM_NAME}:${VM_MOUNT_PATH}"

echo "Waiting for VM command channel to become ready..."
wait_for_vm_exec

echo "Installing VM dependencies..."
run_in_vm "sudo apt update"
run_in_vm "sudo DEBIAN_FRONTEND=noninteractive apt install -y \
    build-essential clang cmake curl git make pkg-config protobuf-compiler \
    python3 linux-tools-common linux-tools-generic"

echo "Ensuring tracefs is mounted..."
run_in_vm "mount | grep -q tracefs || sudo mount -t tracefs tracefs /sys/kernel/tracing"

echo "Checking user_events support..."
run_in_vm "uname -a"
if ! run_in_vm "test -e /sys/kernel/tracing/user_events_status"; then
    echo "error: /sys/kernel/tracing/user_events_status not found in VM '${VM_NAME}'." >&2
    echo "This kernel/environment does not expose Linux user_events, so the E2E test cannot run." >&2
    exit 1
fi

if ! run_in_vm "test -e /sys/kernel/tracing/user_events_data"; then
    echo "error: /sys/kernel/tracing/user_events_data not found in VM '${VM_NAME}'." >&2
    echo "This kernel/environment does not expose Linux user_events, so the E2E test cannot run." >&2
    exit 1
fi

echo "Relaxing user_events permissions for VM-local test execution..."
run_in_vm "sudo chmod a+rw /sys/kernel/tracing/user_events_data"
run_in_vm "sudo chmod a+r /sys/kernel/tracing/user_events_status"
run_in_vm "sudo chmod a+rx /sys/kernel/tracing"
run_in_vm "sudo chmod -R a+rx /sys/kernel/tracing/events"

echo "Installing Rust toolchain in VM if needed..."
run_in_vm "if ! command -v cargo >/dev/null 2>&1; then curl https://sh.rustup.rs -sSf | sh -s -- -y; fi"

echo "Running compile checks..."
run_in_vm "source \"\$HOME/.cargo/env\" && cd \"${VM_MOUNT_PATH}/rust/otap-dataflow\" && cargo check -p otap-df-core-nodes"
run_in_vm "source \"\$HOME/.cargo/env\" && cd \"${VM_MOUNT_PATH}/rust/otap-dataflow\" && cargo test -p otap-df-otap --test userevents_exporter_receiver_e2e --no-run"

echo "Running Linux userevents E2E smoke test..."
run_in_vm "source \"\$HOME/.cargo/env\" && cd \"${VM_MOUNT_PATH}/rust/otap-dataflow\" && cargo test -p otap-df-otap --test userevents_exporter_receiver_e2e -- --ignored --nocapture"

echo
echo "Success. VM '${VM_NAME}' ran the userevents E2E smoke test."
