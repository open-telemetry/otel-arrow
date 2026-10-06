# Load Generator

A simple Python script that continuously sends OTLP logs or syslog messages for
a specified duration. At the end of the run, it outputs the total count of logs
sent to stdout, which can be parsed to determine the number of logs sent.

## Setup

### Prerequisites

- Python 3.10+

### Create a virtual environment and install dependencies

```bash
cd tools/pipeline_perf_test/load_generator
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

## Usage

### Standalone OTLP load generation

```bash
python loadgen.py --load-type otlp --duration 30 --threads 4 --batch-size 1000
```

### Standalone syslog UDP load generation

```bash
python loadgen.py --load-type syslog --syslog-server 127.0.0.1 --syslog-port 5140 --duration 30
```

### Standalone syslog TCP load generation

```bash
python loadgen.py --load-type syslog --syslog-server 127.0.0.1 --syslog-port 5140 --syslog-transport tcp --duration 30
```

### Standalone syslog CEF load generation

```bash
python loadgen.py --load-type syslog --syslog-content-type cef --syslog-server 127.0.0.1 --syslog-port 5140 --duration 30
```

### Raw Syslog over Kafka

Kafka mode uses `confluent-kafka`/librdkafka to send one raw Syslog value per
record, not OTLP protobuf. It defaults to RFC 5424, 1024 UTF-8 bytes including
the header, a plain random body, and no trailing newline, key, or compression.
TCP/UDP retain their newline framing and mixed random-message pools.

```bash
python loadgen.py --load-type syslog --syslog-transport kafka \
  --kafka-brokers localhost:19094 --kafka-topic otel-syslog \
  --syslog-format rfc5424 --message-size 1024 --body-size 1024 \
  --threads 1 --batch-size 100 --target-rate 100000 --duration 30
```

Create the topic before starting. `target_rate` is aggregate across workers;
`batch_size` is a scheduling chunk, not multiple logs inside one Kafka record.
Each worker has its own producer. Queue bounds default to 10,000 messages and
16 MiB per producer, with a 5s enqueue deadline, 10s delivery lifetime and 10s
flush deadline. Queue pressure retries without counting false sends; only broker
callbacks advance `logs_produced`, `bytes_sent`, and `logs_bytes_produced`.
`kafka_pending`, delivery/enqueue failures and flush timeouts remain observable.
These acknowledgements (`acks=1` by default) do not prove consumer delivery.

#### Explicit topic/partition routing

To balance accepted records deterministically across a pre-created topology, set
both `kafka_topics` (an ordered list of unique topic names) and
`kafka_partitions_per_topic` (a positive integer). The product of topic count and
partitions per topic must not exceed 256. Names must be 1-249 ASCII letters,
digits, `.`, `_`, or `-`, except `.` and `..`. Explicit routing requires Syslog
Kafka mode and cannot be combined with an explicitly supplied `kafka_topic`,
even its default value.

```bash
python loadgen.py --load-type syslog --syslog-transport kafka \
  --kafka-brokers localhost:19094 --kafka-topics syslog-a syslog-b \
  --kafka-partitions-per-topic 4 --threads 4 --target-rate 100000 \
  --message-size 1024 --duration 30
```

The equivalent `/start` fields are:

```json
{
  "load_type": "syslog",
  "syslog_transport": "kafka",
  "kafka_topics": ["syslog-a", "syslog-b"],
  "kafka_partitions_per_topic": 4,
  "threads": 4,
  "target_rate": 100000,
  "message_size": 1024
}
```

One shared round-robin cursor advances only after a successful enqueue. It
visits topics in list order and each topic's partitions from zero upward,
wrapping after the last target. Workers share this cursor; queue-full retries
do not consume a slot, and polling/backpressure waits do not hold its lock.
`target_rate` remains the **total** requested records per second across all
workers and targets, not a per-topic or per-partition rate. Each worker receives
an equal share regardless of topology. This is a pacing target, not a throughput
guarantee.

`/metrics` exposes broker-confirmed counters for every configured target,
including targets with zero deliveries:

```text
kafka_delivered_records{topic="syslog-a",partition="0"} 0
kafka_delivered_bytes{topic="syslog-a",partition="0"} 0
```

In explicit mode their sums match `logs_produced` and `logs_bytes_produced`
respectively in each atomic snapshot, also returned by `/status` and `/stop`.
Pending or failed records never count as delivered. A new run resets the cursor
and all target series. Omitting both topology fields preserves the singular
`kafka_topic` behavior: librdkafka chooses the partition and no per-target series
are emitted. Topics and partitions must already exist; the generator does not
create them or discover additional broker partitions.

Kafka options are available as CLI flags (`--kafka-send-timeout`, for example)
and corresponding underscore-named JSON fields. `/status` returns lifecycle,
bounded error details and metrics, with HTTP 500 on failure. `/stop` stops and
flushes accepted work and returns HTTP 500 if unsuccessful; it blocks a new run
until an old controller exits. CLI failures exit nonzero. `/metrics` remains
readable for diagnostics even after failure.

The [receiver-only dashboard suite](../../comparison_dashboard/README.md#syslog-kafka-receiver-only-benchmark)
manages the broker, generator, consumer, and final delivery evidence.

### Container build and offline wheelhouse

From the repository root:

```bash
docker build -t load_generator:kafka-syslog tools/pipeline_perf_test/load_generator
```

The Dockerfile installs `requirements.lock.txt` with `--require-hashes`. If the
build cannot reach the package index, download the locked wheels on a
network-enabled Linux host matching the image's Python 3.14 and architecture:

```bash
python -m pip download --only-binary=:all: --require-hashes \
  -r tools/pipeline_perf_test/load_generator/requirements.lock.txt \
  --dest /tmp/loadgen-wheels
docker build --build-context wheelhouse=/tmp/loadgen-wheels \
  --build-arg PIP_NO_INDEX=1 -t load_generator:kafka-syslog \
  tools/pipeline_perf_test/load_generator
```

Use a trusted package index; do not disable TLS verification. The offline build
uses BuildKit's named `wheelhouse` context without copying wheels into the image.

### Server mode (HTTP API control)

Start the load generator as a long-running server, then control it via HTTP:

```bash
python loadgen.py --serve
```

```bash
# Start load generation
curl -X POST http://localhost:5001/start \
  -H "Content-Type: application/json" \
  -d '{"load_type": "syslog", "batch_size": 1000, "threads": 2, "syslog_server": "127.0.0.1", "syslog_port": 5140}'

# Stop load generation
curl -X POST http://localhost:5001/stop

# Get metrics
curl http://localhost:5001/metrics

# Check lifecycle/error state
curl --fail http://localhost:5001/status
```

## Future Enhancements

- Utilize language-specific OpenTelemetry SDKs.
- Integrate load generation tools like Locust or custom telemetry generators.
- Extend the script to support configurable options such as log size and length.
