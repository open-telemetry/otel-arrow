#!/usr/bin/env bash
set -euo pipefail

sed \
  -e "s/KAFKA_SASL_TLS_HOST/${KAFKA_SASL_TLS_HOST}/g" \
  -e "s/KAFKA_SASL_TLS_PORT/${KAFKA_SASL_TLS_PORT}/g" \
  /workspace/server.properties >/tmp/server.properties

/opt/kafka/bin/kafka-storage.sh format \
  --ignore-formatted \
  --cluster-id "4L6g3nShT-eMCtK--X86sw" \
  --config /tmp/server.properties \
  --add-scram 'SCRAM-SHA-256=[name=scram256,password=scram256-secret]' \
  --add-scram 'SCRAM-SHA-512=[name=scram512,password=scram512-secret]'

exec /opt/kafka/bin/kafka-server-start.sh /tmp/server.properties
