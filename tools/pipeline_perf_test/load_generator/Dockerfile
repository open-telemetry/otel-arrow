FROM python:3.14-slim@sha256:b877e50bd90de10af8d82c57a022fc2e0dc731c5320d762a27986facfc3355c1

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