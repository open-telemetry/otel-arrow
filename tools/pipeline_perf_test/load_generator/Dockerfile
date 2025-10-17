FROM python:3.13-slim@sha256:079601253d5d25ae095110937ea8cfd7403917b53b077870bccd8b026dc7c42f

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