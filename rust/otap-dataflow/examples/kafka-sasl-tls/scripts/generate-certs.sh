#!/bin/sh
set -eu

if [ -f /certs/ca.crt ] && [ -f /certs/kafka.keystore.jks ] && \
  [ -f /certs/kafka.truststore.jks ] && [ -f /certs/kafka_server_jaas.conf ]; then
  echo "Kafka TLS certificates already exist."
  exit 0
fi

rm -f /certs/*

openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout /certs/ca.key \
  -out /certs/ca.crt \
  -days 3650 \
  -subj "/CN=OTAP Kafka Local CA"

openssl req -newkey rsa:2048 -nodes \
  -keyout /certs/kafka.key \
  -out /certs/kafka.csr \
  -subj "/CN=localhost"

cat >/certs/kafka.ext <<'EOF'
subjectAltName=DNS:localhost,DNS:kafka,IP:127.0.0.1
extendedKeyUsage=serverAuth
EOF

openssl x509 -req \
  -in /certs/kafka.csr \
  -CA /certs/ca.crt \
  -CAkey /certs/ca.key \
  -CAcreateserial \
  -out /certs/kafka.crt \
  -days 3650 \
  -extfile /certs/kafka.ext

openssl pkcs12 -export \
  -in /certs/kafka.crt \
  -inkey /certs/kafka.key \
  -certfile /certs/ca.crt \
  -name kafka \
  -out /certs/kafka.p12 \
  -passout pass:changeit

keytool -importkeystore -noprompt \
  -srckeystore /certs/kafka.p12 \
  -srcstoretype PKCS12 \
  -srcstorepass changeit \
  -destkeystore /certs/kafka.keystore.jks \
  -deststorepass changeit \
  -destkeypass changeit

keytool -importcert -noprompt \
  -alias local-ca \
  -file /certs/ca.crt \
  -keystore /certs/kafka.truststore.jks \
  -storepass changeit

printf 'changeit' >/certs/kafka_keystore_creds
printf 'changeit' >/certs/kafka_sslkey_creds
printf 'changeit' >/certs/kafka_truststore_creds
cat >/certs/kafka_server_jaas.conf <<'EOF'
KafkaServer {
  org.apache.kafka.common.security.plain.PlainLoginModule required
  username="plain"
  password="plain-secret"
  user_plain="plain-secret";
};
EOF
chmod 644 /certs/*

echo "Generated Kafka TLS certificates for localhost and kafka."
