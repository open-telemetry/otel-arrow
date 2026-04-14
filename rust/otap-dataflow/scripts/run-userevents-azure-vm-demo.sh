#!/usr/bin/env bash
set -euo pipefail

# Runs the working user-events demo on the Azure VM:
# OTEL SDK -> user-events exporter -> kernel tracepoint ->
# df_engine user-events receiver -> debug processor.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OTAP_DF_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

VM_HOST="${VM_HOST:-40.70.242.60}"
VM_USER="${VM_USER:-azureuser}"
VM_PASS_FILE="${VM_PASS_FILE:-/tmp/vmpasswd}"

OTEL_ARROW_REPO_URL="${OTEL_ARROW_REPO_URL:-https://github.com/lalitb/otel-arrow.git}"
OTEL_ARROW_BRANCH="${OTEL_ARROW_BRANCH:-user-events-receiver}"
OTEL_CONTRIB_REPO_URL="${OTEL_CONTRIB_REPO_URL:-https://github.com/lalitb/opentelemetry-rust-contrib.git}"

REMOTE_BASE="${REMOTE_BASE:-/mnt/builddisk/demo-userevents}"
REMOTE_OTEL_ARROW="${REMOTE_BASE}/otel-arrow"
REMOTE_OTAP_DF="${REMOTE_OTEL_ARROW}/rust/otap-dataflow"
REMOTE_CONTRIB="${REMOTE_BASE}/opentelemetry-rust-contrib"
REMOTE_CARGO_HOME="${REMOTE_CARGO_HOME:-/mnt/builddisk/cargo-home}"
REMOTE_CARGO_TARGET_DIR="${REMOTE_CARGO_TARGET_DIR:-/mnt/builddisk/cargo-target}"
REMOTE_TMPDIR="${REMOTE_TMPDIR:-/mnt/builddisk/tmp}"
REMOTE_DEMO_CONFIG="${REMOTE_OTAP_DF}/configs/userevents-debug-demo.yaml"

SSH_OPTS=(
  -o StrictHostKeyChecking=no
  -o ConnectTimeout=15
  -o PreferredAuthentications=password
  -o PubkeyAuthentication=no
)

if [[ ! -f "${VM_PASS_FILE}" ]]; then
  echo "password file not found: ${VM_PASS_FILE}" >&2
  exit 1
fi

VM_PASS="$(<"${VM_PASS_FILE}")"

vm_ssh() {
  sshpass -p "${VM_PASS}" ssh "${SSH_OPTS[@]}" "${VM_USER}@${VM_HOST}" "$@"
}

vm_scp() {
  sshpass -p "${VM_PASS}" scp "${SSH_OPTS[@]}" "$@"
}

vm_sudo_bash() {
  local script="$1"
  vm_ssh "printf '%s\n' '${VM_PASS}' | sudo -S bash -lc $(printf '%q' "${script}")"
}

echo "Preparing VM scratch disk and toolchain..."
vm_sudo_bash "
set -euo pipefail
if ! mountpoint -q /mnt/builddisk; then
  if [[ -b /dev/nvme1n1 ]]; then
    if ! blkid /dev/nvme1n1 >/dev/null 2>&1; then
      mkfs.ext4 -F /dev/nvme1n1
    fi
    mkdir -p /mnt/builddisk
    mount /dev/nvme1n1 /mnt/builddisk || true
  else
    mkdir -p /mnt/builddisk
  fi
fi
chown ${VM_USER}:${VM_USER} /mnt/builddisk
"

vm_ssh "
set -euo pipefail
if ! command -v rustup >/dev/null 2>&1; then
  curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal --default-toolchain stable
fi
. \"\$HOME/.cargo/env\"
mkdir -p '${REMOTE_CARGO_HOME}' '${REMOTE_CARGO_TARGET_DIR}' '${REMOTE_TMPDIR}' '${REMOTE_BASE}'
"

echo "Cloning remote repos if needed..."
vm_ssh "
set -euo pipefail
if [[ ! -d '${REMOTE_OTEL_ARROW}/.git' ]]; then
  rm -rf '${REMOTE_OTEL_ARROW}'
  git clone --branch '${OTEL_ARROW_BRANCH}' --single-branch '${OTEL_ARROW_REPO_URL}' '${REMOTE_OTEL_ARROW}'
fi
if [[ ! -d '${REMOTE_CONTRIB}/.git' ]]; then
  rm -rf '${REMOTE_CONTRIB}'
  git clone --single-branch '${OTEL_CONTRIB_REPO_URL}' '${REMOTE_CONTRIB}'
fi
"

echo "Overlaying local demo files..."
vm_scp \
  "${OTAP_DF_ROOT}/crates/contrib-nodes/src/receivers/userevents_receiver/mod.rs" \
  "${OTAP_DF_ROOT}/crates/contrib-nodes/src/receivers/userevents_receiver/session.rs" \
  "${VM_USER}@${VM_HOST}:${REMOTE_OTAP_DF}/crates/contrib-nodes/src/receivers/userevents_receiver/"

echo "Writing remote demo config..."
vm_ssh "
cat > '${REMOTE_DEMO_CONFIG}' <<'EOF'
version: otel_dataflow/v1

groups:
  default:
    pipelines:
      main:
        policies:
          channel_capacity:
            control:
              node: 100
              pipeline: 100
            pdata: 128

        nodes:
          receiver:
            type: receiver:userevents
            config:
              tracepoint: \"user_events:myprovider_L2K1\"
              format:
                type: common_schema_otel_logs
              session:
                wakeup_watermark: 1
                late_registration:
                  enabled: true
                  poll_interval_ms: 100
              batching:
                max_size: 1
                max_duration: 100ms

          debug:
            type: processor:debug
            config:
              verbosity: normal
              mode: signal
              output: userevents-debug-output.log
              signals:
                - logs

          noop:
            type: exporter:noop
            config: {}

        connections:
          - from: receiver
            to: debug
          - from: debug
            to: noop
EOF
"

echo "Building df_engine and exporter example on the VM..."
vm_ssh "
set -euo pipefail
. \"\$HOME/.cargo/env\"
export CARGO_HOME='${REMOTE_CARGO_HOME}'
export CARGO_TARGET_DIR='${REMOTE_CARGO_TARGET_DIR}'
export TMPDIR='${REMOTE_TMPDIR}'
export CARGO_BUILD_JOBS=8
cd '${REMOTE_OTAP_DF}'
cargo build --no-default-features --features crypto-ring --bin df_engine
cd '${REMOTE_CONTRIB}/opentelemetry-user-events-logs'
cargo build --example basic-logs --all-features
"

echo "Running end-to-end demo..."
vm_sudo_bash "
set -euo pipefail
cd '${REMOTE_OTAP_DF}'
rm -f userevents-debug-output.log /mnt/builddisk/userevents-exporter-demo.log /mnt/builddisk/df-engine-demo.log
timeout 15s '${REMOTE_CARGO_TARGET_DIR}/debug/df_engine' --num-cores 1 --config '${REMOTE_DEMO_CONFIG}' >/mnt/builddisk/df-engine-demo.log 2>&1 &
engine_pid=\$!
sleep 3
timeout 5s taskset -c 0 '${REMOTE_CARGO_TARGET_DIR}/debug/examples/basic-logs' >/mnt/builddisk/userevents-exporter-demo.log 2>&1 || test \$? -eq 124
sleep 5
kill \$engine_pid 2>/dev/null || true
wait \$engine_pid 2>/dev/null || true
"

echo
echo "VM verification summary:"
vm_sudo_bash "
set -euo pipefail
cd '${REMOTE_OTAP_DF}'
if [[ -f userevents-debug-output.log ]]; then
  echo 'DEBUG_FILE_BEGIN'
  cat userevents-debug-output.log
  echo 'DEBUG_FILE_END'
else
  echo 'DEBUG_FILE_MISSING'
fi
echo 'ENGINE_TAIL_BEGIN'
tail -n 80 /mnt/builddisk/df-engine-demo.log || true
echo 'ENGINE_TAIL_END'
echo 'EXPORTER_TAIL_BEGIN'
tail -n 40 /mnt/builddisk/userevents-exporter-demo.log || true
echo 'EXPORTER_TAIL_END'
"
