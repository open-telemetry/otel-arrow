#!/usr/bin/env bash

set -euo pipefail

artifact_dir="${OTEL_ARROW_INTEGRATION_ARTIFACT_DIR:?artifact directory is required}"
runner_temp="${RUNNER_TEMP:-$artifact_dir/tmp}"
mkdir -p "$artifact_dir" "$runner_temp"

oracle_username="OTAP$(openssl rand -hex 4 | tr '[:lower:]' '[:upper:]')"
oracle_password="CiA1_$(openssl rand -hex 10)"
echo "::add-mask::$oracle_password"

export ORACLE_CONNECT_STRING="//127.0.0.1:1521/FREEPDB1"
export ORACLE_USERNAME="$oracle_username"
export ORACLE_PWD="$oracle_password"

docker run --detach \
  --name oracle-receiver-ci \
  --publish 127.0.0.1:1521:1521 \
  --env ORACLE_RANDOM_PASSWORD=true \
  --env APP_USER="$oracle_username" \
  --env APP_USER_PASSWORD="$oracle_password" \
  --health-cmd healthcheck.sh \
  --health-interval 10s \
  --health-timeout 5s \
  --health-retries 60 \
  gvenzl/oracle-free@sha256:0489e0c1f20b2ca632075653c66f284234689ccff62c9a39809d9a5b3e7c1642

sudo apt-get update
sudo apt-get install --yes libaio1t64 unzip

libaio_path="$(dpkg-query -L libaio1t64 | grep -E '/libaio\.so\.1t64$' || true)"
if [[ ! -f "$libaio_path" ]]; then
  echo "libaio1t64 did not provide libaio.so.1t64." >&2
  exit 1
fi
sudo ln -sf "$libaio_path" "$(dirname "$libaio_path")/libaio.so.1"

curl --fail --location --silent --show-error \
  https://download.oracle.com/otn_software/linux/instantclient/2326000/instantclient-basiclite-linux.x64-23.26.0.0.0.zip \
  --output "$runner_temp/instantclient.zip"
echo "94a458f43873a420e5be6fdf18e9a85df0393b4a1e2999ed406d8ec50f235e71  $runner_temp/instantclient.zip" \
  | sha256sum --check
unzip -q "$runner_temp/instantclient.zip" -d "$runner_temp"

client_dir="$runner_temp/instantclient_23_26"
export LD_LIBRARY_PATH="$client_dir${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export ORACLE_INSTANT_CLIENT_DIR="$client_dir"
ldd "$client_dir/libclntsh.so" | tee "$artifact_dir/instant-client-ldd.txt"
if grep -F 'not found' "$artifact_dir/instant-client-ldd.txt" > /dev/null; then
  echo "Oracle Instant Client has unresolved dependencies." >&2
  exit 1
fi

umask 077
printf '%s' "$oracle_password" > "$runner_temp/oracle-password"
chmod 600 "$runner_temp/oracle-password"
export ORACLE_PASSWORD_FILE="$runner_temp/oracle-password"

for _ in {1..60}; do
  state="$(docker inspect --format '{{.State.Status}}' oracle-receiver-ci)"
  health="$(docker inspect --format '{{.State.Health.Status}}' oracle-receiver-ci)"
  if [[ "$health" == "healthy" ]]; then
    break
  fi
  if [[ "$state" != "running" ]]; then
    echo "Oracle container stopped before becoming healthy." >&2
    exit 1
  fi
  sleep 10
done
if [[ "$(docker inspect --format '{{.State.Health.Status}}' oracle-receiver-ci)" != "healthy" ]]; then
  echo "Oracle did not become healthy." >&2
  exit 1
fi

cd rust/otap-dataflow
cargo run --locked \
  -p otel-arrow-dfe-contrib-nodes \
  --features oracle \
  --example oracle_load_generator \
  -- \
  --rows 10 \
  --collision-size 2 \
  --reset

OTAP_ORACLE_RECEIVER_E2E=1 cargo test --locked \
  -p otel-arrow-dfe-contrib-nodes \
  --features oracle \
  --lib \
  emits_oracle_rows_when_live_test_is_enabled \
  -- \
  --nocapture \
  --test-threads=1 \
  2>&1 | tee "$artifact_dir/oracle-receiver-test.log"
