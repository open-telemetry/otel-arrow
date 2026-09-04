#!/bin/sh

# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

set -eu

: "${ORACLE_CONNECT_STRING:?ORACLE_CONNECT_STRING is required}"
: "${ORACLE_USERNAME:?ORACLE_USERNAME is required}"
: "${ORACLE_PWD:?ORACLE_PWD is required}"
: "${ORACLE_USERNAME_FILE:?ORACLE_USERNAME_FILE is required}"
: "${ORACLE_PASSWORD_FILE:?ORACLE_PASSWORD_FILE is required}"

mkdir -p "$(dirname "$ORACLE_USERNAME_FILE")" "$(dirname "$ORACLE_PASSWORD_FILE")"
printf '%s' "$ORACLE_USERNAME" >"$ORACLE_USERNAME_FILE"
printf '%s' "$ORACLE_PWD" >"$ORACLE_PASSWORD_FILE"
chmod 600 "$ORACLE_USERNAME_FILE" "$ORACLE_PASSWORD_FILE"

rows="${ORACLE_DEMO_ROWS:-25}"
collision_size="${ORACLE_DEMO_COLLISION_SIZE:-5}"
attempt=1
max_attempts=90

echo "Waiting for Oracle and preparing ${rows} deterministic demo rows..."
until /app/oracle_load_generator \
    --reset \
    --rows "$rows" \
    --collision-size "$collision_size"; do
    if [ "$attempt" -ge "$max_attempts" ]; then
        echo "Oracle did not become ready after ${max_attempts} attempts." >&2
        exit 1
    fi
    attempt=$((attempt + 1))
    sleep 5
done

stabilization_seconds="${ORACLE_DEMO_STABILIZATION_SECONDS:-5}"
echo "Oracle is ready; waiting ${stabilization_seconds}s for new sessions to stabilize."
sleep "$stabilization_seconds"

echo "Starting the Oracle receiver. Composite watermark paging begins at the initial cursor."
exec /app/df_engine \
    --config /app/oracle-oci-console.yaml \
    --num-cores 1
