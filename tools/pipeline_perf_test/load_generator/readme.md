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
```

## Future Enhancements

- Utilize language-specific OpenTelemetry SDKs.
- Integrate load generation tools like Locust or custom telemetry generators.
- Extend the script to support configurable options such as log size and length.
