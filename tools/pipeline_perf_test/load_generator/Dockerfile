FROM python:3.14-slim@sha256:3955a7dd66ccf92b68d0232f7f86d892eaf75255511dc7e98961bdc990dc6c9b

WORKDIR /app

# Install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

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