#!/usr/bin/env bash
set -euo pipefail

rm -rf /out/*

keytool -genkeypair \
  -alias broker \
  -keyalg RSA \
  -keysize 2048 \
  -validity 3650 \
  -dname "CN=localhost" \
  -ext "SAN=DNS:localhost,DNS:broker,DNS:host.docker.internal,IP:127.0.0.1" \
  -storetype PKCS12 \
  -keystore /out/broker.p12 \
  -storepass changeit \
  -keypass changeit \
  -noprompt

keytool -exportcert \
  -alias broker \
  -keystore /out/broker.p12 \
  -storepass changeit \
  -rfc \
  -file /out/ca.pem

keytool -importcert \
  -alias broker \
  -file /out/ca.pem \
  -storetype PKCS12 \
  -keystore /out/client.truststore.p12 \
  -storepass changeit \
  -noprompt

cat >/out/plain-client.properties <<'EOF'
security.protocol=SASL_SSL
sasl.mechanism=PLAIN
sasl.jaas.config=org.apache.kafka.common.security.plain.PlainLoginModule required username="plain" password="plain-secret";
ssl.truststore.location=/etc/kafka/secrets/client.truststore.p12
ssl.truststore.password=changeit
ssl.truststore.type=PKCS12
EOF

chmod 0644 /out/*
