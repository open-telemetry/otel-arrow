FROM python:3.13-slim@sha256:f2fdaec50160418e0c2867ba3e254755edd067171725886d5d303fd7057bbf81

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