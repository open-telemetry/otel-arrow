#!/usr/bin/env bash
set -euo pipefail

kafka-configs --bootstrap-server kafka:29092 --alter \
  --entity-type users \
  --entity-name scram256 \
  --add-config 'SCRAM-SHA-256=[iterations=8192,password=scram256-secret]'

kafka-configs --bootstrap-server kafka:29092 --alter \
  --entity-type users \
  --entity-name scram512 \
  --add-config 'SCRAM-SHA-512=[iterations=8192,password=scram512-secret]'

for topic in otlp-logs-plain otlp-logs-scram-256 otlp-logs-scram-512; do
  kafka-topics --bootstrap-server kafka:29092 --create --if-not-exists \
    --topic "${topic}" \
    --partitions 1 \
    --replication-factor 1
done

echo "Created SCRAM users and OTLP log topics."
