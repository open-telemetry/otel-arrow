# Kafka Receiver SASL/TLS Validation

This fixture runs a real single-node Kafka broker with one `SASL_SSL` listener
and validates the Kafka receiver with three separate consumer groups:

- SASL/PLAIN
- SCRAM-SHA-256
- SCRAM-SHA-512

The ignored Rust E2E test produces one OTLP traces request per mechanism, starts
the real `KafkaReceiver`, acknowledges the decoded request, and confirms that
the corresponding consumer group's committed offset reaches `1`.

## Prerequisites

- Docker Engine with Docker Compose

The fixture uses `apache/kafka:4.1.1`. Certificates and test credentials are
generated locally and removed when the Compose project is torn down. Kafka is
exposed on `localhost:39093` by default; set `KAFKA_SASL_TLS_PORT` to override
the host port. The Rust test runs in a pinned builder image, with Cargo caches
kept in Docker volumes between runs.

## Run

From PowerShell:

```powershell
.\rust\otap-dataflow\crates\contrib-nodes\src\common\kafka\test\sasl_tls\run.ps1
```

From Bash:

```bash
./rust/otap-dataflow/crates/contrib-nodes/src/common/kafka/test/sasl_tls/run.sh
```

The scripts bring up Kafka, wait for an authenticated TLS health check, run the
single ignored E2E test, print broker logs on failure, and remove the containers
and volume.
