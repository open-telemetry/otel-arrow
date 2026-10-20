#!/usr/bin/env bash
# Diagnose container -> host connectivity for the push-based metrics sink.
#
# Push suites run an OTLP sink on the host (0.0.0.0:14317, see the
# start_otlp_metrics_sink hook) and the engine containers on the 'testbed'
# network export to http://host.docker.internal:14317, which docker maps to the
# host via 'extra_hosts: host.docker.internal: host-gateway'.
#
# This script starts a plain TCP listener on the sink port, probes it from
# containers using several routes, and dumps the host network and firewall
# state so we can tell a firewall drop from a name resolution problem.
#
# Usage: ./scripts/diagnose-push-sink.sh [port]
# Always exits 0; it only reports.

set -u

PORT="${1:-14317}"
NETWORK="testbed"
PROBE_IMAGE="busybox:1.36.1"
CREATED_NETWORK=0
LISTENER_PID=""

section() {
    echo
    echo "=== $* ==="
}

cleanup() {
    if [ -n "$LISTENER_PID" ]; then
        kill "$LISTENER_PID" 2>/dev/null || true
    fi
    if [ "$CREATED_NETWORK" = "1" ]; then
        docker network rm "$NETWORK" >/dev/null 2>&1 || true
    fi
}
trap cleanup EXIT

# Run a TCP connect probe from a container. $1 is a label, $2 the target host,
# the remaining args are extra 'docker run' flags.
probe() {
    local label="$1" target="$2"
    shift 2
    echo "--- probe: $label -> $target:$PORT"
    docker run --rm "$@" "$PROBE_IMAGE" sh -c "
        echo 'hosts entry:' \$(grep host.docker.internal /etc/hosts || echo none)
        if nc -w 3 $target $PORT </dev/null; then
            echo 'RESULT: CONNECTED'
        else
            echo 'RESULT: FAILED (exit '\$?')'
        fi
    " 2>&1
}

section "Docker"
docker version --format 'server {{.Server.Version}} client {{.Client.Version}}' 2>&1
docker info --format 'driver={{.Driver}} cgroup={{.CgroupDriver}} rootless={{.SecurityOptions}}' 2>&1
grep -H . /etc/docker/daemon.json 2>/dev/null || echo "no /etc/docker/daemon.json"

section "Start TCP listener on 0.0.0.0:$PORT"
python3 -c "
import socket
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(('0.0.0.0', $PORT))
s.listen(16)
while True:
    c, addr = s.accept()
    print('listener: accepted connection from', addr[0], flush=True)
    c.close()
" &
LISTENER_PID=$!
sleep 1
if ! kill -0 "$LISTENER_PID" 2>/dev/null; then
    echo "listener failed to start (port in use?)"
    LISTENER_PID=""
fi
ss -ltnp 2>/dev/null | grep ":$PORT " || echo "nothing listening on :$PORT"

section "Network '$NETWORK'"
if ! docker network inspect "$NETWORK" >/dev/null 2>&1; then
    docker network create "$NETWORK" >/dev/null && CREATED_NETWORK=1
    echo "created network (will be removed on exit)"
fi
TESTBED_GW=$(docker network inspect "$NETWORK" --format '{{range .IPAM.Config}}{{.Gateway}}{{end}}')
TESTBED_BRIDGE="br-$(docker network inspect "$NETWORK" --format '{{.Id}}' | cut -c1-12)"
DOCKER0_IP=$(ip -4 -o addr show docker0 2>/dev/null | awk '{print $4}' | cut -d/ -f1)
echo "testbed gateway: ${TESTBED_GW:-unknown} (bridge $TESTBED_BRIDGE)"
echo "docker0 ip:      ${DOCKER0_IP:-unknown}"

section "Probes"
docker pull -q "$PROBE_IMAGE" >/dev/null 2>&1 || echo "failed to pull $PROBE_IMAGE"
# Sanity: host network namespace, no bridge or firewall in the path.
probe "host network" 127.0.0.1 --network host
# Exactly what the push suites do.
probe "testbed + host-gateway" host.docker.internal \
    --network "$NETWORK" --add-host host.docker.internal:host-gateway
# Same network, skipping host-gateway resolution.
if [ -n "$TESTBED_GW" ]; then
    probe "testbed -> testbed gateway" "$TESTBED_GW" --network "$NETWORK"
fi
# Default bridge, where host-gateway usually points.
probe "default bridge + host-gateway" host.docker.internal \
    --add-host host.docker.internal:host-gateway

section "Host addresses and routes"
ip -4 -br addr 2>&1
ip -4 route 2>&1

section "Firewall: ufw"
sudo -n ufw status verbose 2>&1 || echo "ufw not available"

section "Firewall: iptables filter INPUT / DOCKER-USER"
sudo -n iptables -S INPUT 2>&1 || true
sudo -n iptables -S DOCKER-USER 2>&1 || true

section "Firewall: nftables (input hooks only)"
sudo -n nft list ruleset 2>/dev/null | grep -n -B2 -A20 "hook input" | head -120 \
    || echo "nft not available"

section "Kernel"
sysctl net.ipv4.ip_forward net.bridge.bridge-nf-call-iptables 2>&1

exit 0
