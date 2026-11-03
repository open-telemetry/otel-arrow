#!/usr/bin/env bash
# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

tls_dir=/var/lib/postgresql/tls
mkdir -p "${tls_dir}" /tls-export

openssl req -x509 -newkey rsa:2048 -nodes -days 2 \
  -subj '/CN=OTel PostgreSQL Test CA' \
  -keyout "${tls_dir}/ca.key" \
  -out "${tls_dir}/ca.pem"
openssl req -newkey rsa:2048 -nodes \
  -subj '/CN=postgres' \
  -addext 'subjectAltName=DNS:postgres,DNS:localhost,IP:127.0.0.1' \
  -keyout "${tls_dir}/server.key" \
  -out "${tls_dir}/server.csr"
openssl x509 -req \
  -in "${tls_dir}/server.csr" \
  -CA "${tls_dir}/ca.pem" \
  -CAkey "${tls_dir}/ca.key" \
  -CAcreateserial \
  -days 2 \
  -copy_extensions copyall \
  -out "${tls_dir}/server.crt"

cp "${tls_dir}/ca.pem" /tls-export/ca.pem
printf '%s\n' 'otel_reader' > /tls-export/username
printf '%s\n' 'otel-reader-password' > /tls-export/password
chmod 0644 /tls-export/ca.pem /tls-export/username /tls-export/password
chown -R postgres:postgres "${tls_dir}"
chmod 0600 "${tls_dir}/server.key"

exec /usr/local/bin/docker-entrypoint.sh postgres \
  -c ssl=on \
  -c ssl_cert_file="${tls_dir}/server.crt" \
  -c ssl_key_file="${tls_dir}/server.key"
