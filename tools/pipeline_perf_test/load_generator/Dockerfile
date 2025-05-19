FROM python:3.12-slim@sha256:31a416db24bd8ade7dac5fd5999ba6c234d7fa79d4add8781e95f41b187f4c9a

WORKDIR /app

# Install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy the load generator code
COPY *.py ./

# Set environment variables
ENV PYTHONUNBUFFERED=1
ENV OTLP_ENDPOINT=localhost:4317

# Command to run the load generator
ENTRYPOINT ["python", "loadgen.py"]
CMD ["--duration", "30"]