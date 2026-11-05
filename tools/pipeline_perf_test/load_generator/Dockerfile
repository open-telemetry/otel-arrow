FROM python:3.15-slim@sha256:e97b88e416c3c01eb427b21fd8cd3ebf006e6ee38afcfcc3cae1c54ee2ab0a75

WORKDIR /app

# Install dependencies
COPY requirements.lock.txt .
RUN python -m pip install --no-cache-dir --require-hashes -r requirements.lock.txt

# Copy the load generator code
COPY *.py ./

# Set environment variables
ENV PYTHONUNBUFFERED=1
ENV OTLP_ENDPOINT=localhost:4317
ENV SYSLOG_SERVER=localhost
ENV SYSLOG_PORT=514

# Command to run the load generator
ENTRYPOINT ["python", "loadgen.py"]
CMD ["--duration", "30"]